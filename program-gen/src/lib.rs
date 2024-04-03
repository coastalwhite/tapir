mod arbitrary;
mod randombits;

mod classes;

use std::io;

use risico::memory::PlacedBytes;
use rvhwfuzzer_encoding::{FRegIdent, Instruction, RoundingMode, XRegIdent, CXRegIdent};
use rvisa::{MIsa, MXLen, MIsaExt};

use crate::arbitrary::ArbitraryGenerationContext;
use crate::classes::alu::CAluInstruction;
use crate::classes::muldiv::MulDivInstruction;

use self::arbitrary::{ArbitraryInstruction, HopTarget};
use self::classes::alu::AluInstruction;
use self::classes::csr::{CsrImmWrite, CsrRead, CsrWrite};
use self::classes::fpu32::FPU32Instruction;
use self::classes::nonhopping_branches::NonHoppingBranch;
use self::randombits::RandomBits;

const NUM_REGISTERS: usize = 32;
const FPU_NUM_REGISTERS: usize = 32;

const MIN_BASIC_BLOCKS: usize = 1;
const MAX_BASIC_BLOCKS: usize = 100;
const MIN_INSTRUCTIONS_PER_BB: usize = 1;
const MAX_INSTRUCTIONS_PER_BB: usize = 100;

const MIN_PADDING: u32 = 16;
const MAX_PADDING: u32 = 64;

impl RegisterRecencyList {
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

    fn take_signed_immediate(&mut self, bitsize: u32) -> i64 {
        let imm = self.take_immediate(bitsize);

        // Sign-Extend
        let shift = 64 - bitsize;
        (imm << shift) as i64 >> shift
    }

    fn take_u8(&mut self, bitsize: u32) -> u8 {
        debug_assert!(bitsize <= 8);
        self.take_immediate(bitsize) as u8
    }
    fn take_u16(&mut self, bitsize: u32) -> u16 {
        debug_assert!(bitsize <= 16);
        self.take_immediate(bitsize) as u16
    }
    fn take_u32(&mut self, bitsize: u32) -> u32 {
        debug_assert!(bitsize <= 32);
        self.take_immediate(bitsize) as u32
    }

    fn take_i8(&mut self, bitsize: u32) -> i8 {
        debug_assert!(bitsize <= 8);
        (self.take_signed_immediate(bitsize) & 0xFF) as i8
    }
    fn take_i16(&mut self, bitsize: u32) -> i16 {
        debug_assert!(bitsize <= 16);
        (self.take_signed_immediate(bitsize) & 0xFFFF) as i16
    }
    fn take_i32(&mut self, bitsize: u32) -> i32 {
        debug_assert!(bitsize <= 32);
        (self.take_signed_immediate(bitsize) & 0xFFFF_FFFF) as i32
    }

    pub fn new() -> Self {
        // @Hack. This should probably be more random.
        Self {
            entropy: RandomBits::new(),
            inner: std::array::from_fn(|i| XRegIdent::take_masked((i + 1) as u32)),
            fpu: std::array::from_fn(|i| FRegIdent::take_masked(i as u32)),
        }
    }

    pub fn take_compressed_register_src(&mut self) -> CXRegIdent {
        // This is quite hacky, but it works fine
        
        let reg = self.take_register_src();
        let reg = reg as u32;
        CXRegIdent::take_masked(reg & 0x7)
    }

    pub fn take_register_src(&mut self) -> XRegIdent {
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

    pub fn take_fpu_register_src(&mut self) -> FRegIdent {
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

    pub fn take_compressed_register_dest(&mut self) -> CXRegIdent {
        // @Hack. This should not be hard coded
        let value = self.entropy.take_u32(3);
        let register = CXRegIdent::take_masked(value);

        let mut prev = XRegIdent::from(register);
        for recent_register in self.inner.iter_mut() {
            std::mem::swap(recent_register, &mut prev);

            if prev == XRegIdent::from(register) {
                break;
            }
        }

        register
    }

    pub fn take_register_dest(&mut self) -> XRegIdent {
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

    pub fn take_fpu_register_dest(&mut self) -> FRegIdent {
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

struct RegisterRecencyList {
    entropy: RandomBits,
    inner: [XRegIdent; NUM_REGISTERS - 1],
    fpu: [FRegIdent; FPU_NUM_REGISTERS],
}

impl RegisterRecencyList {}

macro_rules! weighted_random {
    (
        [$ctx:ident]
        $($weight:literal => $struct:ident,)+
        $fallback:ident $(,)?
    ) => {
        const MAX_WEIGHT: i32 = 0 $(+ $weight)+;
        
        let mut offset = 0i32;
        let mut r = fastrand::i32(0..MAX_WEIGHT);

        $(
        if $struct::is_available($ctx) {
            offset += $weight;
            if r < offset {
                return $struct::take($ctx);
            }
        } else {
            #[allow(unused_assignments)]
            { r -= $weight; }
        }
        )+

        unreachable!();

    };
}

pub struct Jump;
pub struct HoppingBranch;

impl ArbitraryInstruction for Jump {
    fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction {
        let padding = fastrand::u32(MIN_PADDING..MAX_PADDING);
        let padding = padding & !0x3;
        let offset = padding + 4;

        ctx.hop_target = HopTarget::Padded(padding);

        rvhwfuzzer_encoding::Jal::new(XRegIdent::Zero, offset as i32).into()
    }
}

impl ArbitraryInstruction for HoppingBranch {
    fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction {
        let padding = fastrand::u32(MIN_PADDING..MAX_PADDING);
        let padding = padding & !0x3;
        let offset = padding + 4;

        let rs1 = ctx.params_mut().take_register_src();
        let rs2 = ctx.params_mut().take_register_src();

        let rs1_value = ctx.state().registers().get(rs1);
        let rs2_value = ctx.state().registers().get(rs2);

        ctx.hop_target = HopTarget::Padded(padding);

        if rs1_value == rs2_value {
            rvhwfuzzer_encoding::Beq::new(rs1, rs2, offset as i16).into()
        } else {
            rvhwfuzzer_encoding::Bne::new(rs1, rs2, offset as i16).into()
        }
    }
}

fn instantiate_hop_instruction(ctx: &mut ArbitraryGenerationContext) -> Instruction {
    weighted_random! {
        [ctx]
        8 => Jump,
        8 => HoppingBranch,
        Jump,
    }
}

fn instantiate_still_instruction(ctx: &mut ArbitraryGenerationContext) -> Instruction {
    weighted_random! {
        [ctx]
        32 => AluInstruction,
        32 => CAluInstruction,
        8  => MulDivInstruction,
        8  => FPU32Instruction,
        8  => CsrRead,
        4  => CsrWrite,
        3  => CsrImmWrite,
        1  => NonHoppingBranch,
        AluInstruction
    }
}

fn add_instruction(
    ctx: &mut ArbitraryGenerationContext,
    instruction: Instruction,
) -> io::Result<()> {
    debug_assert_eq!(ctx.state().pc(), ctx.state().memory().end());

    // eprintln!("{instruction}");

    instruction.encode(ctx.state_mut().memory_mut().bytes_mut())?;
    ctx.state_mut().instruction_execute(instruction);

    Ok(())
}

pub fn generate_binary(entry: u32) -> io::Result<Box<[u8]>> {
    let recency_list = RegisterRecencyList::new();

    let state = risico::State::new(
        MIsa::RV32I.with_ext(MIsaExt::INTEGER_MULDIV | MIsaExt::COMPRESSED | MIsaExt::SINGLE_PRECISION_FP),
        risico::StateECallBehavior {
            machine: risico::syscall::ECallBehavior::TrapVector,
            user: risico::syscall::ECallBehavior::TrapVector,
        },
        risico::trap::TrapBehavior::empty(),
        entry,
        PlacedBytes::new(entry, Vec::new()),
        None,
    );

    let mut ctx = ArbitraryGenerationContext {
        hop_target: HopTarget::Padded(0),
        state,
        parameter_provider: recency_list,
    };

    for i in 1..NUM_REGISTERS {
        let value = rand::random::<u32>();
        let r = XRegIdent::take_masked(i as u32);

        let lui = rvhwfuzzer_encoding::Lui::new(r, value);
        let addi = rvhwfuzzer_encoding::Addi::new(r, r, (((value & 0xFFF) << 4) as i32) >> 4);

        add_instruction(&mut ctx, lui.into())?;
        add_instruction(&mut ctx, addi.into())?;
    }

    let num_bbs = fastrand::usize(MIN_BASIC_BLOCKS..MAX_BASIC_BLOCKS);

    for _ in 0..num_bbs {
        let num_instructions = fastrand::usize(MIN_INSTRUCTIONS_PER_BB..MAX_INSTRUCTIONS_PER_BB);

        for _ in 0..num_instructions {
            let instruction = instantiate_still_instruction(&mut ctx);
            add_instruction(&mut ctx, instruction)?;
        }

        let hop_instruction = instantiate_hop_instruction(&mut ctx);
        add_instruction(&mut ctx, hop_instruction)?;

        match ctx.hop_target {
            HopTarget::Padded(padding) => {
                ctx.state_mut()
                    .memory_mut()
                    .bytes_mut()
                    .extend(std::iter::repeat(0).take(padding as usize));
            }
        }
    }

    let binary = std::mem::take(ctx.state_mut().memory_mut().bytes_mut());

    Ok(binary.into_boxed_slice())
}

#[no_mangle]
pub extern "C" fn rv_generate_instructions(entry: u32, length: *mut u32) -> *mut u8 {
    let binary =
        ::std::panic::catch_unwind(|| generate_binary(entry).unwrap()).unwrap_or_else(|err| {
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

    let binary = generate_binary(0)?;

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
