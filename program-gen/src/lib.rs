mod arbitrary;
mod randombits;

mod classes;

use std::io;

use risico::memory::PlacedBytes;
use risico::repr::Addr;
use rvhwfuzzer_encoding::{FRegIdent, Instruction, RoundingMode, XRegIdent};

use crate::arbitrary::ArbitraryGenerationContext;

use self::arbitrary::{ArbitraryInstruction, ArbitraryParameterProvider};
use self::classes::alu::AluInstruction;
use self::classes::csr::{CsrImmWrite, CsrRead, CsrWrite};
use self::classes::fpu32::FPU32Instruction;
use self::classes::nonhopping_branches::NonHoppingBranch;
use self::randombits::RandomBits;

const NUM_REGISTERS: usize = 32;
const FPU_NUM_REGISTERS: usize = 32;

const MIN_BASIC_BLOCKS: usize = 1;
const MAX_BASIC_BLOCKS: usize = 100;
const MAX_INSTRUCTIONS_PER_BB: usize = 100;

const MIN_PADDING: usize = 16;
const MAX_PADDING: usize = 64;

const MAX_INSTRUCTIONS_BYTESIZE: usize = 4;

const PROGRAM_SIZE_UPPERBOUND: usize = 32 * MAX_INSTRUCTIONS_BYTESIZE * 2
    + MAX_PADDING * MAX_BASIC_BLOCKS
    + MAX_INSTRUCTIONS_BYTESIZE * MAX_INSTRUCTIONS_PER_BB * MAX_INSTRUCTIONS_PER_BB;

impl ArbitraryParameterProvider for RegisterRecencyList {
    fn take_register_src(&mut self) -> XRegIdent {
        self.take_source()
    }

    fn take_register_dest(&mut self) -> XRegIdent {
        self.take_destination()
    }

    fn take_fpu_register_src(&mut self) -> FRegIdent {
        self.take_fpu_source()
    }

    fn take_fpu_register_dest(&mut self) -> FRegIdent {
        self.take_fpu_destination()
    }

    fn take_static_rounding_mode(&mut self) -> RoundingMode {
        let rm = self.entropy.take(3);

        use RoundingMode as R;
        match rm {
            0b001 => R::ToZero,
            0b010 => R::Down,
            0b011 => R::Up,
            0b100 => R::TiesToMaxMagnitude,

            // @NOTE: This is biased towards the TiesToEven operation
            _ => R::TiesToEven,
        }
    }

    fn take_rounding_mode(&mut self) -> RoundingMode {
        let rm = self.entropy.take(3);

        use RoundingMode as R;
        match rm {
            0b001 => R::ToZero,
            0b010 => R::Down,
            0b011 => R::Up,
            0b100 => R::TiesToMaxMagnitude,
            0b111 => R::Dynamic,

            // @NOTE: This is biased towards the TiesToEven operation
            _ => R::TiesToEven,
        }
    }

    fn take_immediate(&mut self, bitsize: u32) -> u64 {
        debug_assert!(bitsize > 0 && bitsize <= 64);

        self.entropy.take(bitsize)
    }
}

#[derive(Clone, Copy, Debug)]
enum HopClass {
    Jump,
    Branch,
    FPU32,
}

#[derive(Clone, Copy, Debug)]
enum StillClass {
    // RegFSM,
    // FPUFSM,
    Alu,

    // @Temporary
    Branch,
    // ALU64,
    // MulDiv,
    // MulDiv64,
    // AMO,
    // AMO64,
    // Jal,
    // Jalr,
    // Branch,
    // Mem,
    // Mem64,
    // MemFPU,
    FPU32,
    // FPU64,
    // MemFPUD,
    // FPUD,
    // FPUD64,
    // TVECFSM,
    // PPFSM,
    // EPCFSM,
    // Medeleg,
    // Exception,
    // RandomCSR,
    // DescendPrivilege,
    // Special,
    ReadCsr,
    WriteCsr,
    WriteCsrImmediate,
}

#[rustfmt::skip]
enum ExceptionCauseValue {
    InstructionAddrMisaligned =  0,
    InstructionAccessFault    =  1,
    IllegalInstruction        =  2,
    Breakpoint                =  3,
    LoadAddrMisaligned        =  4,
    LoadAccessFault           =  5,
    StoreAMOAddrMisaligned    =  6,
    StoreAMOAccessFault       =  7,
    EnvironmentCallFromUMode  =  8,
    EnvironmentCallFromSMode  =  9,
    EnvironmentCallFromMMode  = 11,
    InstructionPageFault      = 12,
    LoadPageFault             = 13,
    StoreAMOPageFault         = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BasicBlockId(usize);

struct BasicBlock {
    classes: Box<[StillClass]>,
    hop: HopClass,
}

pub struct ControlFlowGraph {
    inner: Box<[BasicBlock]>,
}

struct RegisterRecencyList {
    entropy: RandomBits,
    inner: [XRegIdent; NUM_REGISTERS - 1],
    fpu: [FRegIdent; FPU_NUM_REGISTERS],
}

impl RegisterRecencyList {
    pub fn new() -> Self {
        // @Hack. This should probably be more random.
        Self {
            entropy: RandomBits::new(),
            inner: std::array::from_fn(|i| XRegIdent::take_masked((i + 1) as u32)),
            fpu: std::array::from_fn(|i| FRegIdent::take_masked(i as u32)),
        }
    }

    pub fn take_source(&mut self) -> XRegIdent {
        // This works as follows.
        // We take a weighted random register from `register_recency`.
        //
        //   RegId : Weight
        //  1 -  3 : 128
        //       0 :  64
        //  4 -  7 :  32
        //  8 - 11 :  16
        // 12 - 15 :   8
        // 16 - 19 :   4
        // 20 - 23 :   2
        // 24 - 31 :   2

        let offset: u8 = self.entropy.take_u8(4);
        let weight: u8 = self.entropy.take_u8(8);

        match weight {
            0..=127 => self.inner[u8::min(offset & 0x3, 2) as usize],
            128..=191 => XRegIdent::Zero,
            192..=223 => self.inner[4 + (offset & 0x3) as usize],
            224..=239 => self.inner[8 + (offset & 0x3) as usize],
            240..=247 => self.inner[12 + (offset & 0x3) as usize],
            248..=251 => self.inner[16 + (offset & 0x3) as usize],
            252..=253 => self.inner[20 + (offset & 0x3) as usize],
            254..=255 => self.inner[24 + u8::min(offset & 0x7, 6) as usize],
        }
    }

    pub fn take_fpu_source(&mut self) -> FRegIdent {
        // This works as follows.
        // We take a weighted random register from `register_recency`.
        //
        //   RegId : Weight
        //  0 -  3 : 128
        //  4 -  7 :  64
        //  8 - 11 :  32
        // 12 - 15 :  16
        // 16 - 19 :   8
        // 20 - 23 :   4
        // 24 - 31 :   4

        let offset: u8 = self.entropy.take_u8(4);
        let weight: u8 = self.entropy.take_u8(8);

        match weight {
            0..=127 => self.fpu[(offset & 0x3) as usize],
            128..=191 => self.fpu[4 + (offset & 0x3) as usize],
            192..=223 => self.fpu[8 + (offset & 0x3) as usize],
            224..=239 => self.fpu[12 + (offset & 0x3) as usize],
            240..=247 => self.fpu[16 + (offset & 0x3) as usize],
            248..=251 => self.fpu[20 + (offset & 0x3) as usize],
            252..=255 => self.fpu[24 + (offset & 0x7) as usize],
        }
    }

    pub fn take_destination(&mut self) -> XRegIdent {
        // @Hack. This should not be hard coded
        let value = self.entropy.take_u32(5);
        let register = XRegIdent::take_masked(value);

        if register == XRegIdent::Zero {
            return register;
        }

        let mut prev = register;
        for recent_register in self.inner.iter_mut() {
            std::mem::swap(recent_register, &mut prev);

            if prev == register {
                break;
            }
        }

        register
    }

    pub fn take_fpu_destination(&mut self) -> FRegIdent {
        // @Hack. This should not be hard coded
        let value = self.entropy.take_u32(5);
        let register = FRegIdent::take_masked(value);

        let mut prev = register;
        for recent_register in self.fpu.iter_mut() {
            std::mem::swap(recent_register, &mut prev);

            if prev == register {
                break;
            }
        }

        register
    }
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        fn get_instruction_class() -> StillClass {
            let t = rand::random::<u32>() % 32;

            match t {
                0..=8 => StillClass::Alu,
                0..=15 => StillClass::WriteCsrImmediate,
                16..=23 => StillClass::FPU32,
                24..=27 => StillClass::ReadCsr,
                28..=30 => StillClass::WriteCsr,
                31 => StillClass::Branch,
                _ => unreachable!(),
            }
        }

        fn get_edge() -> HopClass {
            let t = rand::random::<u32>() % 2;

            match t {
                0 => HopClass::Jump,
                1 => HopClass::Branch,
                2 => HopClass::FPU32,
                _ => unreachable!(),
            }
        }

        let num_basic_blocks =
            MIN_BASIC_BLOCKS + rand::random::<usize>() % (MAX_BASIC_BLOCKS - MIN_BASIC_BLOCKS);

        let mut basic_blocks = Vec::with_capacity(num_basic_blocks);

        for _ in 0..num_basic_blocks {
            let num_instructions = rand::random::<usize>() % MAX_INSTRUCTIONS_PER_BB;

            let mut classes = Vec::with_capacity(num_instructions);
            for _ in 0..num_instructions {
                classes.push(get_instruction_class());
            }

            basic_blocks.push(BasicBlock {
                classes: classes.into_boxed_slice(),
                hop: get_edge(),
            });
        }

        Self {
            inner: basic_blocks.into_boxed_slice(),
        }
    }
}

impl StillClass {
    fn instantiate<P>(self, ctx: &mut ArbitraryGenerationContext<P>) -> Instruction
    where
        P: ArbitraryParameterProvider,
    {
        match self {
            StillClass::Alu => AluInstruction::take(ctx),
            StillClass::FPU32 => FPU32Instruction::take(ctx),
            StillClass::Branch => NonHoppingBranch::take(ctx),
            StillClass::ReadCsr => CsrRead::take(ctx),
            StillClass::WriteCsr => CsrWrite::take(ctx),
            StillClass::WriteCsrImmediate => CsrImmWrite::take(ctx),
        }
    }
}

fn add_instruction<P>(ctx: &mut ArbitraryGenerationContext<P>, instruction: Instruction) -> io::Result<()>
where
    P: ArbitraryParameterProvider
{
    debug_assert_eq!(ctx.state().pc(), ctx.state().memory().end());

    // eprintln!("{instruction}");

    instruction.encode(ctx.state_mut().memory_mut().bytes_mut())?;
    ctx.state_mut().instruction_execute(instruction);

    Ok(())
}

pub fn generate_binary(cfg: ControlFlowGraph, entry: u32) -> io::Result<Box<[u8]>> {
    let recency_list = RegisterRecencyList::new();

    let state = risico::State::new(
        risico::device_config::Isa::Rv32I,
        risico::syscall::SystemCallBehavior::Abort,
        entry,
        PlacedBytes::new(entry, Vec::new()),
    );

    let mut ctx = ArbitraryGenerationContext {
        state,
        parameter_provider: recency_list,
    };

    for i in 1..NUM_REGISTERS {
        let value = rand::random::<u32>();
        let r = XRegIdent::take_masked(i as u32);

        let lui = rvhwfuzzer_encoding::Lui::new(r, value);
        let addi = rvhwfuzzer_encoding::Addi::new(r, r, (((value & 0xFFF) << 4) as i16) >> 4);

        add_instruction(&mut ctx, lui.into())?;
        add_instruction(&mut ctx, addi.into())?;
    }

    for bb in cfg.inner.iter() {
        for class in bb.classes.iter() {
            let instruction = class.instantiate(&mut ctx);
            add_instruction(&mut ctx, instruction)?;
        }

        let padding = rand::random::<usize>() % (MAX_PADDING - MIN_PADDING) + MIN_PADDING;
        let padding = padding & !0x3;
        let offset = (padding + 4) as i16;

        match bb.hop {
            HopClass::Jump => {
                let instruction = rvhwfuzzer_encoding::Jal::new(XRegIdent::Zero, offset as _);
                add_instruction(&mut ctx, instruction.into())?;
            }
            HopClass::Branch => {
                let rs1 = ctx.params_mut().take_register_src();
                let rs2 = ctx.params_mut().take_register_src();

                let rs1_value = ctx.state().registers().get(rs1);
                let rs2_value = ctx.state().registers().get(rs2);

                let instruction = if rs1_value == rs2_value {
                    rvhwfuzzer_encoding::Beq::new(rs1, rs2, offset).into()
                } else {
                    rvhwfuzzer_encoding::Bne::new(rs1, rs2, offset).into()
                };

                add_instruction(&mut ctx, instruction)?;
            }
            HopClass::FPU32 => {
                unimplemented!()
            }
        }

        ctx.state_mut().memory_mut().bytes_mut().extend(std::iter::repeat(0).take(padding));
    }

    let binary = std::mem::take(ctx.state_mut().memory_mut().bytes_mut());

    Ok(binary.into_boxed_slice())
}

#[no_mangle]
pub extern "C" fn rv_generate_instructions(entry: u32, length: *mut u32) -> *mut u8 {
    let binary = ::std::panic::catch_unwind(|| {
        let cfg = ControlFlowGraph::new();
        generate_binary(cfg, entry).unwrap()
    })
    .unwrap_or_else(|err| {
        eprintln!("Panic occurred: {err:?}");
        ::std::process::exit(1);
    });

    if let Some(length) = unsafe { length.as_mut() } {
        *length = binary.len() as u32;
    }

    Box::leak(binary).as_mut_ptr().cast()
}

#[no_mangle]
pub extern "C" fn rv_free_instructions(ptr: *mut u8, length: u32) {
    unsafe {
        let slice = std::slice::from_raw_parts_mut(ptr, length as usize);
        let _: Box<[u8]> = <Box<[u8]>>::from_raw(slice);
    }
}

#[test]
fn show_concrete() -> std::io::Result<()> {
    // use std::io::Write;

    // let mut stdout = std::io::stdout().lock();
    // let stdout = &mut stdout;

    let mut num_instructions = 0u64;

    // for i in 0..100 {
    let cfg = ControlFlowGraph::new();

    let binary = generate_binary(cfg, 0)?;

    // for i in (0..binary.len()).step_by(4) {
    //     if &binary[i..i+4] == &[0,0,0,0] {
    //         continue;
    //     }
    //
    //     let decoded = ::rvhwfuzzer_encoding::Instruction::decode(&binary[i..]).unwrap();
    //     writeln!(stdout, "{}", ::rvhwfuzzer_encoding::asm::AsmDisplay {
    //         ctx: &Default::default(),
    //         instr: &decoded,
    //     })?;
    // }

    // for (i, b) in binary.iter().enumerate() {
    //     if i != 0 && i % 8 == 0 {
    //         writeln!(stdout)?;
    //     }
    //
    //     write!(stdout, "{b:02X} ")?;
    // }
    //
    // writeln!(stdout)?;
    //
    std::fs::write("test.bin", &binary)?;
    //
    // writeln!(stdout)?;

    num_instructions += (binary.len() / 4) as u64;

    // writeln!(stdout, "Bytes: {}", binary.len())?;
    // writeln!(stdout, "Instructions: ~{}", binary.len() / 4)?;
    //
    // writeln!(stdout)?;
    // }
    //
    // drop(stdout);

    println!("# of instructions: {num_instructions}");

    assert!(false);

    Ok(())
}
