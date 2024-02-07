mod arbitrary;
mod randombits;

mod classes;

use std::io;

use risico::memory::PlacedBytes;
use risico::repr::Addr;
use risico::RegIdent;

use crate::arbitrary::ArbitraryGenerationContext;

use self::arbitrary::{ArbitraryInstruction, ArbitraryParameterProvider};
use self::classes::alu::AluInstruction;
use self::classes::csr::{CsrImmWrite, CsrRead, CsrWrite};
use self::classes::fpu32::FPU32Instruction;
use self::classes::nonhopping_branches::NonHoppingBranch;
use self::randombits::RandomBits;

#[derive(Debug)]
struct HoppingEqualityBranch {
    rs1: RegisterId,
    rs2: RegisterId,
    is_equal: bool,
}

impl ArbitraryInstruction for HoppingEqualityBranch {
    fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self {
        let rs1 = ctx.params_mut().take_register_src();
        let rs2 = ctx.params_mut().take_register_src();

        let rs1_value = ctx.state().registers().get(RegIdent::from(rs1.0));
        let rs2_value = ctx.state().registers().get(RegIdent::from(rs2.0));

        let is_equal = rs1_value == rs2_value;

        Self { rs1, rs2, is_equal }
    }
}

impl HoppingEqualityBranch {
    pub fn encode(&self, imm12_1: u16, writer: &mut impl io::Write) -> io::Result<()> {
        let imm4_1 = imm12_1 & 0xF;
        let imm10_5 = (imm12_1 >> 4) & 0x3F;
        let imm11 = (imm12_1 >> 10) & 1;
        let imm12 = (imm12_1 >> 11) & 1;

        let imm4_1 = imm4_1 as u8;
        let imm10_5 = imm10_5 as u8;
        let imm11 = imm11 as u8;
        let imm12 = imm12 as u8;

        let rs1 = self.rs1.0;
        let rs2 = self.rs2.0;

        if self.is_equal {
            rvhwfuzzer_encoding::BeqArgs {
                imm10_5,
                imm4_1,
                imm11,
                rs1,
                imm12,
                rs2,
            }
            .encode(writer)
        } else {
            rvhwfuzzer_encoding::BneArgs {
                imm10_5,
                imm4_1,
                imm11,
                rs1,
                imm12,
                rs2,
            }
            .encode(writer)
        }
    }
}

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
    fn take_register_src(&mut self) -> RegisterId {
        self.take_source()
    }

    fn take_register_dest(&mut self) -> RegisterId {
        self.take_destination()
    }

    fn take_fpu_register_src(&mut self) -> FPURegisterId {
        self.take_fpu_source()
    }

    fn take_fpu_register_dest(&mut self) -> FPURegisterId {
        self.take_fpu_destination()
    }

    fn take_static_rounding_mode(&mut self) -> arbitrary::RoundingMode {
        let rm = self.entropy.take(3);

        use arbitrary::RoundingMode as R;
        match rm {
            0b001 => R::ToZero,
            0b010 => R::Down,
            0b011 => R::Up,
            0b100 => R::ToMaxMagnitude,

            // @NOTE: This is biased towards the TiesToEven operation
            _ => R::TiesToEven,
        }
    }

    fn take_rounding_mode(&mut self) -> arbitrary::RoundingMode {
        let rm = self.entropy.take(3);

        use arbitrary::RoundingMode as R;
        match rm {
            0b001 => R::ToZero,
            0b010 => R::Down,
            0b011 => R::Up,
            0b100 => R::ToMaxMagnitude,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegisterId(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FPURegisterId(u8);

struct BasicBlock {
    classes: Box<[StillClass]>,
    hop: HopClass,
}

pub struct ControlFlowGraph {
    inner: Box<[BasicBlock]>,
}

struct RegisterRecencyList {
    entropy: RandomBits,
    inner: [RegisterId; NUM_REGISTERS - 1],
    fpu: [FPURegisterId; FPU_NUM_REGISTERS],
}

impl RegisterRecencyList {
    pub fn new() -> Self {
        // @Hack. This should probably be more random.
        Self {
            entropy: RandomBits::new(),
            inner: std::array::from_fn(|i| RegisterId(i as u8)),
            fpu: std::array::from_fn(|i| FPURegisterId(i as u8)),
        }
    }

    pub fn take_source(&mut self) -> RegisterId {
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
            128..=191 => RegisterId(0),
            192..=223 => self.inner[4 + (offset & 0x3) as usize],
            224..=239 => self.inner[8 + (offset & 0x3) as usize],
            240..=247 => self.inner[12 + (offset & 0x3) as usize],
            248..=251 => self.inner[16 + (offset & 0x3) as usize],
            252..=253 => self.inner[20 + (offset & 0x3) as usize],
            254..=255 => self.inner[24 + u8::min(offset & 0x7, 6) as usize],
        }
    }

    pub fn take_fpu_source(&mut self) -> FPURegisterId {
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

    pub fn take_destination(&mut self) -> RegisterId {
        // @Hack. This should not be hard coded
        let value = self.entropy.take_u8(5);
        let register = RegisterId(value);

        if register == RegisterId(0) {
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

    pub fn take_fpu_destination(&mut self) -> FPURegisterId {
        // @Hack. This should not be hard coded
        let value = self.entropy.take_u8(5);
        let register = FPURegisterId(value);

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
    fn instantiate<W, P>(
        self,
        writer: &mut W,
        ctx: &mut ArbitraryGenerationContext<P>,
    ) -> io::Result<()>
    where
        W: io::Write,
        P: ArbitraryParameterProvider,
    {
        match self {
            StillClass::Alu => {
                let alu_instruction = AluInstruction::take(ctx);
                alu_instruction.encode(writer)
            }
            StillClass::FPU32 => {
                let fpu32_instruction = FPU32Instruction::take(ctx);
                fpu32_instruction.encode(writer)
            }
            StillClass::Branch => {
                let non_hopping_branch = NonHoppingBranch::take(ctx);
                non_hopping_branch.encode(writer)
            }
            StillClass::ReadCsr => {
                let readcsr = CsrRead::take(ctx);
                readcsr.encode(writer)
            }
            StillClass::WriteCsr => {
                let writecsr = CsrWrite::take(ctx);
                writecsr.encode(writer)
            }
            StillClass::WriteCsrImmediate => {
                let writecsr = CsrImmWrite::take(ctx);
                writecsr.encode(writer)
            }
        }
    }
}

fn catch_up(entry: u32, state: &mut risico::State<PlacedBytes>, binary: &mut Vec<u8>) {
    let start = state.pc();
    let end = Addr::from(entry + binary.len() as u32);

    std::mem::swap(state.memory_mut().bytes_mut(), binary);
    state.run_while(|state| state.pc() >= start && state.pc() < end);
    std::mem::swap(state.memory_mut().bytes_mut(), binary);

    assert_eq!(state.pc(), end);
}

pub fn generate_binary(cfg: ControlFlowGraph, entry: u32) -> io::Result<Box<[u8]>> {
    let recency_list = RegisterRecencyList::new();

    let mut binary = Vec::<u8>::new();

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
        rvhwfuzzer_encoding::LuiArgs {
            rd: i as u8,
            imm31_12: value >> 12,
        }
        .encode(&mut binary)?;
        rvhwfuzzer_encoding::AddiArgs {
            rd: i as u8,
            rs: i as u8,
            imm11_0: (value & 0xFFF) as u16,
        }
        .encode(&mut binary)?;
    }

    catch_up(entry, &mut ctx.state_mut(), &mut binary);

    for bb in cfg.inner.iter() {
        for class in bb.classes.iter() {
            catch_up(entry, &mut ctx.state_mut(), &mut binary);
            class.instantiate(&mut binary, &mut ctx)?;
        }

        catch_up(entry, &mut ctx.state_mut(), &mut binary);

        let padding = rand::random::<usize>() % (MAX_PADDING - MIN_PADDING) + MIN_PADDING;
        let padding = padding & !0x3;
        let jump_addr = (padding + 4) >> 1;

        match bb.hop {
            HopClass::Jump => {
                rvhwfuzzer_encoding::JalArgs {
                    imm19_12: 0,
                    imm11: 0,
                    rd: 0,
                    imm10_1: jump_addr as u16,
                    imm20: 0,
                }
                .encode(&mut binary)?;
            }
            HopClass::Branch => {
                let hopping_eq_branch = HoppingEqualityBranch::take(&mut ctx);
                hopping_eq_branch.encode(jump_addr as u16, &mut binary)?;
            }
            HopClass::FPU32 => {
                unimplemented!()
            }
        }

        binary.extend(std::iter::repeat(0).take(padding));

        catch_up(entry, &mut ctx.state_mut(), &mut binary);
    }

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
