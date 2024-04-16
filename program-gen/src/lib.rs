mod arbitrary;
mod randombits;

mod classes;

mod backing_store;

use std::io;

use risico::repr::{Addr, Size};
use risico::memory::{Segment, SegmentTree};
use rvhwfuzzer_encoding::{CXRegIdent, FRegIdent, Instruction, RoundingMode, XRegIdent};
use rvisa::{MIsa, MIsaExt};

use crate::arbitrary::{ArbitraryGenerationContext, ArbitraryHopInstruction, Branch};
use crate::classes::alu::CAluInstruction;
use crate::classes::memory::MemoryInstruction;
use crate::classes::muldiv::MulDivInstruction;

use self::arbitrary::HopTarget;
use self::backing_store::ProgramMemory;
use self::classes::alu::AluInstruction;
use self::classes::csr::{CsrImmWrite, CsrRead, CsrWrite};
use self::classes::fpu32::FPU32Instruction;
use self::classes::hop::Hop;
use self::randombits::RandomBits;

const NUM_REGISTERS: usize = 32;
const FPU_NUM_REGISTERS: usize = 32;

const MIN_BASIC_BLOCKS: usize = 1;
const MAX_BASIC_BLOCKS: usize = 100;
const MIN_INSTRUCTIONS_PER_BB: usize = 1;
const MAX_INSTRUCTIONS_PER_BB: usize = 100;

struct RegisterRecencyList {
    entropy: RandomBits,
    inner: [XRegIdent; NUM_REGISTERS - 1],
    fpu: [FRegIdent; FPU_NUM_REGISTERS],
}

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

    // fn take_rounding_mode(&mut self) -> RoundingMode {
    //     let rm = self.entropy.take(3);
    //
    //     use RoundingMode as R;
    //     match rm {
    //         0b001 => R::ToZero,
    //         0b010 => R::Down,
    //         0b011 => R::Up,
    //         0b100 => R::TiesToMaxMagnitude,
    //         0b111 => R::Dynamic,
    //
    //         // @NOTE: This is biased towards the TiesToEven operation
    //         _ => R::TiesToEven,
    //     }
    // }

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
        let offset: u8 = self.entropy.take_u8(4);
        let weight: u8 = self.entropy.take_u8(8);

        match weight {
            0..=127 => self.inner[u8::min(offset & 0x3, 2) as usize],
            128..=191 => self.inner[4 + (offset & 0x3) as usize],
            192..=223 => self.inner[8 + (offset & 0x3) as usize],
            224..=239 => XRegIdent::Zero,
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

macro_rules! weighted_random {
    (
        [$ctx:ident]
        $($weight:literal => $struct:ident,)+
        $fallback:ident $(,)?
    ) => {
        const MAX_WEIGHT: u32 = 0 $(+ $weight)+;

        let mut r = fastrand::u32(0..MAX_WEIGHT);

        $(
        if r < $weight {
            if let Some(instr) = <$struct as arbitrary::ArbitraryContextualStillInstruction>::try_take($ctx) {
                return instr;
            }

            #[allow(unused_assignments)]
            { r = 0; }
        } else {
            #[allow(unused_assignments)]
            { r -= $weight; }
        }
        )+

        <$fallback as arbitrary::ArbitraryStillInstruction>::take($ctx)
    };
}

fn instantiate_hop_instruction(ctx: &mut ArbitraryGenerationContext) -> (Instruction, HopTarget) {
    <Hop as ArbitraryHopInstruction>::take(ctx)
}

fn instantiate_still_instruction(ctx: &mut ArbitraryGenerationContext) -> Instruction {
    weighted_random! {
        [ctx]
        32 => CAluInstruction,
        160 => MemoryInstruction,
        192 => AluInstruction,
        8  => MulDivInstruction,
        8  => FPU32Instruction,
        8  => CsrRead,
        4  => CsrWrite,
        3  => CsrImmWrite,
        1  => Branch,
        AluInstruction
    }
}

fn add_instruction(
    ctx: &mut ArbitraryGenerationContext,
    instruction: Instruction,
) -> io::Result<()> {
    debug_assert_eq!(ctx.state().pc(), ctx.state().memory().bin.end());

    instruction.encode(&mut ctx.state_mut().memory_mut().bin.bytes)?;
    ctx.state_mut().instruction_execute(instruction);

    Ok(())
}

pub struct MemoryArea {
    pub start: Addr,
    pub bytes: Vec<u8>,
}

pub struct Program {
    bin: MemoryArea,
    memory_areas: SegmentTree,
    entry: u32,
}

impl Program {
    /// Get the segments of the program memory
    pub fn data_memory_segments(&self) -> std::slice::Iter<Segment> {
        self.memory_areas.segments_iter()
    }

    /// Take the instruction memory
    pub fn take_instruction_memory(self) -> MemoryArea {
        self.bin
    }

    /// Get a reference to the instruction memory
    pub fn instruction_memory(&self) -> &[u8] {
        &self.bin.bytes
    }

    /// Get the entry point of the program
    pub fn entry(&self) -> u32 {
        self.entry
    }
}

impl MemoryArea {
    #[inline]
    pub fn contains_addr(&self, addr: Addr) -> bool {
        // @Hack: Should this really be an `as u32`??
        addr > self.start && addr.as_u32() - self.start.as_u32() < self.bytes.len() as u32
    }

    #[inline]
    pub fn start(&self) -> Addr {
        self.start
    }

    #[inline]
    pub fn end(&self) -> Addr {
        // @Hack: Should this really be an `as u32`??
        Addr::from(self.start.as_u32() + (self.bytes.len() as u32))
    }

    #[inline]
    pub fn len(&self) -> Size {
        // @Hack: This should not unwrap
        Size::from_usize(self.bytes.len()).unwrap()
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn range(&self) -> std::ops::Range<u32> {
        self.start().as_u32()..self.end().as_u32()
    }
}

pub fn generate_binary(
    entry: u32,
    data_memory_ranges: &[std::ops::Range<u32>],
) -> io::Result<Program> {
    let memory_areas = SegmentTree::new();
    let recency_list = RegisterRecencyList::new();

    let memory = ProgramMemory {
        bin: MemoryArea {
            start: Addr::from(entry),
            bytes: Vec::new(),
        },
        memory_areas,
    };

    let state = risico::State::new(
        MIsa::RV32I
            .with_ext(MIsaExt::INTEGER_MULDIV | MIsaExt::COMPRESSED | MIsaExt::SINGLE_PRECISION_FP),
        risico::StateECallBehavior {
            machine: risico::syscall::ECallBehavior::TrapVector,
            user: risico::syscall::ECallBehavior::TrapVector,
        },
        risico::trap::TrapBehavior::empty(),
        entry,
        memory,
        None,
    );

    let mut ctx = ArbitraryGenerationContext {
        state,
        parameter_provider: recency_list,
        data_memory_ranges: data_memory_ranges.to_vec(),
        potential_memory_registers: Vec::new(),
    };

    for i in 1..NUM_REGISTERS {
        let value = rand::random::<u32>();
        let r = XRegIdent::take_masked(i as u32);

        let lui = rvhwfuzzer_encoding::Lui::new(r, value);
        let addi = rvhwfuzzer_encoding::Addi::new(r, r, (((value & 0xFFF) << 4) as i16) >> 4);

        add_instruction(&mut ctx, lui.into())?;
        add_instruction(&mut ctx, addi.into())?;
    }

    let num_bbs = fastrand::usize(MIN_BASIC_BLOCKS..MAX_BASIC_BLOCKS);

    for _ in 0..num_bbs {
        // println!("# of potential registers: {}", ctx.potential_memory_registers.len());
        ctx.fill_potential_memory_registers();
        let num_instructions = fastrand::usize(MIN_INSTRUCTIONS_PER_BB..MAX_INSTRUCTIONS_PER_BB);

        for _ in 0..num_instructions {
            let instruction = instantiate_still_instruction(&mut ctx);
            add_instruction(&mut ctx, instruction)?;
        }

        let (hop_instruction, hop_target) = instantiate_hop_instruction(&mut ctx);
        add_instruction(&mut ctx, hop_instruction)?;

        match hop_target {
            HopTarget::Padded(padding) => {
                ctx.state_mut()
                    .memory_mut()
                    .bin
                    .bytes
                    .extend(std::iter::repeat(0).take(padding as usize));
            }
        }
    }

    let ProgramMemory { bin, memory_areas } = ctx.take_state().take_memory();

    Ok(Program {
        bin,
        memory_areas,
        entry,
    })
}

// #[no_mangle]
// pub extern "C" fn rv_generate_instructions(entry: u32, length: *mut u32) -> *mut u8 {
//     let binary =
//         ::std::panic::catch_unwind(|| generate_binary(entry).unwrap()).unwrap_or_else(|err| {
//             eprintln!("Panic occurred: {err:?}");
//             ::std::process::exit(1);
//         });
//
//     if let Some(length) = unsafe { length.as_mut() } {
//         *length = binary.len() as u32;
//     }
//
//     Box::leak(binary).as_mut_ptr().cast()
// }
//
// #[no_mangle]
// pub extern "C" fn rv_free_instructions(ptr: *mut u8, length: u32) {
//     unsafe {
//         let slice = std::slice::from_raw_parts_mut(ptr, length as usize);
//         let _: Box<[u8]> = <Box<[u8]>>::from_raw(slice);
//     }
// }

#[test]
fn show_concrete() -> std::io::Result<()> {
    use std::io::Write;

    let mut stdout = std::io::stdout().lock();
    let stdout = &mut stdout;

    // let mut num_instructions = 0u64;

    for _ in 0..100 {
        let binary = generate_binary(0, &[0x7000_0000..0x8000_0000])?;

        let mut binary = &binary.bin.bytes[..];

        while !binary.is_empty() {
            match ::rvhwfuzzer_encoding::Instruction::decode(&mut binary).unwrap() {
                None => {
                    writeln!(stdout, "<unknown instr>")?;
                }
                Some(instr) => {
                    writeln!(stdout, "{}", instr)?;
                }
            }
        }
    }

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
    // std::fs::write("test.bin", &binary.bin.bytes)?;
    //
    // writeln!(stdout)?;

    // num_instructions += (binary.bin.len().as_usize() / 4) as u64;

    // writeln!(stdout, "Bytes: {}", binary.len())?;
    // writeln!(stdout, "Instructions: ~{}", binary.len() / 4)?;
    //
    // writeln!(stdout)?;
    // }
    //
    // drop(stdout);

    // println!("# of instructions: {num_instructions}");

    assert!(false);

    Ok(())
}
