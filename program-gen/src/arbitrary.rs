use risico::repr::Addr;
use rvhwfuzzer_encoding::{Instruction, XRegIdent};

use crate::backing_store::ProgramMemory;
use crate::{RegisterRecencyList, NUM_REGISTERS};

pub enum HopTarget {
    Padded(u32),
}

pub struct Jump;
pub struct Branch;

mod compressed;
mod fpu32;
mod hop;

pub struct ArbitraryGenerationContext {
    pub state: risico::State<ProgramMemory>,
    pub parameter_provider: RegisterRecencyList,
    pub data_memory_ranges: Vec<std::ops::Range<u32>>,
    pub potential_memory_registers: Vec<(XRegIdent, std::ops::Range<u32>)>,
}

fn addr_in_range_of_memory_region(addr: Addr, start: u32, end: u32) -> bool {
    // @Hack: This should be a WrappingRange, not a normal range
    // @Hack: This is currently way to conservative. Even taking -8 instead of -4.
    addr.as_u32() >= start && addr.as_u32() < end - 8

    // @Hack: This should take into account that we can have an offset value
    // 12-bit signed immediate, plus a margin of 8
    // const MAX_OFFSET: u32 = (1 << 11) - 8 - 12;

    // let min_abs_diff = u32::min(
    //     addr.as_u32().abs_diff(start),
    //     addr.as_u32().abs_diff(end),
    //     );
    //
    // dbg!(min_abs_diff);

    // delta_start < MAX_OFFSET && delta_end < MAX_OFFSET
}

impl ArbitraryGenerationContext {
    #[inline(always)]
    pub fn state(&self) -> &risico::State<ProgramMemory> {
        &self.state
    }

    #[inline(always)]
    pub fn state_mut(&mut self) -> &mut risico::State<ProgramMemory> {
        &mut self.state
    }

    #[inline(always)]
    pub fn take_state(self) -> risico::State<ProgramMemory> {
        self.state
    }

    // #[inline(always)]
    // pub fn params(&self) -> &RegisterRecencyList {
    //     &self.parameter_provider
    // }

    #[inline(always)]
    pub fn params_mut(&mut self) -> &mut RegisterRecencyList {
        &mut self.parameter_provider
    }

    // Problems for the memory operations:
    //
    // 1. Initiating the memory
    //    * You don't want to initiate it all, because you will be using only a little of it. It
    //      is actually better not to use a lot, because that increases data locality.
    //    * You want this to not cause too much runtime overhead.
    //    * Maybe look into Interval / Segment Trees.
    // 2. Find which registers can reach into memory properly.
    //    * This is more difficult as you don't want to run over every memory region for every
    //      register every time you want to check whether a memory instruction is available.
    //    * Three main methods:
    //      1. Check every time you try to take a register for a memory operation. Caching doesn't
    //         really work as the negative case is very promenent.
    //      2. Update a list of viable registers every time a register value is updates. This would
    //         require intensive hooking into risico, but that might not be the worst.
    //      3. Check periodically for registers that are viable for memory operations,
    //    * For all three methods, we generate a bitmap of the valid addresses beforehand to easy
    //      initial checking, sort of like a Bloom filter.
    pub fn ensure_memory_available(&mut self, addr: Addr, width: u8) {
        debug_assert!(width <= 4);

        // @TODO
        // Make variable according to the width
        let width = 8;

        // @Note
        // This should at least generate from floor(addr, 4) to (addr + 4) because risico uses all
        // that memory to deal with misaligned memory accesses.
        self.state_mut()
            .memory_mut()
            .memory_areas
            .initialize_fill(addr.word_align().as_u32()..addr.as_u32() + width, || fastrand::u8(..))
    }

    // @Improve
    // The registers are not really tempted to go near the available data memory ranges. Maybe, it
    // would be a good idea to add an incentive for that or something. Next to that, it might also
    // make sense to add different operations to the memory:
    //
    //  - Adjust a known memory address
    //  - Adjust a memory address that is adjacent to a known memory address
    //  - Adjust a entirely random memory address
    //
    // This way you increase the amount of data locality.
    pub fn take_memory_register(&mut self) -> Option<(XRegIdent, std::ops::Range<u32>)> {
        if self.potential_memory_registers.is_empty() {
            return None;
        }

        let mut idx = fastrand::usize(0..self.potential_memory_registers.len());

        loop {
            let (reg, area) = &self.potential_memory_registers[idx];

            let addr = self.state().registers().get(*reg).as_addr();

            if addr_in_range_of_memory_region(addr, area.start, area.end) {
                return Some((*reg, area.clone()));
            }

            self.potential_memory_registers.remove(idx);

            if self.potential_memory_registers.is_empty() {
                break;
            }

            idx %= self.potential_memory_registers.len();
        }

        None
    }

    pub fn fill_potential_memory_registers(&mut self) {
        if self.data_memory_ranges.is_empty() {
            return;
        }

        let num_memory_areas = self.data_memory_ranges.len();

        // dbg!(self.state().registers());

        for rs in 1..NUM_REGISTERS {
            let rs = XRegIdent::take_masked(rs as u32);
            let addr = self.state().registers().get(rs).as_addr();

            // println!("Addr[{rs}]: 0x{:08x}", addr.as_u32());

            let region_offset = fastrand::usize(0..num_memory_areas);
            for j in 0..num_memory_areas {
                let region = self.data_memory_ranges[(region_offset + j) % num_memory_areas].clone();

                if addr_in_range_of_memory_region(addr, region.start, region.end) {
                    self.potential_memory_registers.push((rs, region));
                    break;
                }
            }
        }
    }

    fn generate_appropriate_offset(&mut self, addr: Addr, region: std::ops::Range<u32>, width: u32) -> i16 {
        const REACH: u32 = 1 << 11;

        let middle = addr.as_u32();

        let reach = middle.saturating_sub(REACH)..middle.saturating_add(REACH);

        debug_assert!(region.contains(&reach.start) || region.contains(&(reach.end - 1)));

        let overlap = u32::max(region.start, reach.start)..u32::min(region.end, reach.end);

        let target = fastrand::u32(overlap);

        let offset = i64::from(target) - i64::from(middle);
        let offset = offset as i16;
        
        self.ensure_memory_available(Addr::from(target), width as u8);
        
        offset
    }

}

pub trait ArbitraryStillInstruction {
    fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction;
}

pub trait ArbitraryHopInstruction {
    fn take(ctx: &mut ArbitraryGenerationContext) -> (Instruction, HopTarget);
}

pub trait ArbitraryContextualStillInstruction {
    fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<Instruction>;
}

pub trait ArbitraryContextualHopInstruction {
    fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<(Instruction, HopTarget)>;
}

impl<T: ArbitraryStillInstruction> ArbitraryContextualStillInstruction for T {
    #[inline(always)]
    fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<Instruction> {
        Some(Self::take(ctx))
    }
}

macro_rules! impl_regreg_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryStillInstruction for ::rvhwfuzzer_encoding::$name {
            fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction {
                Self::new(
                    ctx.params_mut().take_register_dest(),
                    ctx.params_mut().take_register_src(),
                    ctx.params_mut().take_register_src(),
                ).into()
            }
        }
        )+
    };
}

macro_rules! impl_regimm_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryStillInstruction for ::rvhwfuzzer_encoding::$name {
            fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction {
                Self::new(
                    ctx.params_mut().take_register_dest(),
                    ctx.params_mut().take_register_src(),
                    ctx.params_mut().take_u16(12) as _,
                ).into()
            }
        }
        )+
    };
}

macro_rules! impl_shiftimm_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryStillInstruction for ::rvhwfuzzer_encoding::$name {
            fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction {
                Self::new(
                    ctx.params_mut().take_register_dest(),
                    ctx.params_mut().take_register_src(),
                    ctx.params_mut().take_u8(5),
                ).into()
            }
        }
        )+
    };
}

macro_rules! impl_reghighimm_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryStillInstruction for ::rvhwfuzzer_encoding::$name {
            fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction {
                Self::new(
                    ctx.params_mut().take_register_dest(),
                    ctx.params_mut().take_u32(20) << 12,
                ).into()
            }
        }
        )+
    };
}

impl_regreg_args! {
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
}

impl_regreg_args! {
    Mul,
    MulH,
    MulHsu,
    MulHu,
    Div,
    DivU,
    Rem,
    RemU,
}

impl_regimm_args! {
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
}

impl_reghighimm_args! { Lui, Auipc }

impl_shiftimm_args! { Slli, Srli, Srai }

// pub fn min_wrapping_sub(pivot: u32, a: u32, b: u32) -> u32 {
//     if pivot.wrapping_sub(a) < pivot.wrapping_sub(b) {
//         a
//     } else {
//         b
//     }
// }
//
// /// Exclusive range that wraps around the u32 overflow
// #[derive(Debug, Clone, Copy)]
// struct WrappingRange {
//     start: u32,
//     end: u32,
// }
//
// impl WrappingRange {
//     pub fn contains(self, x: u32) -> bool {
//         debug_assert_ne!(self.start, self.end);
//
//         let greater = x >= self.start;
//         let lesser = x < self.end;
//
//         if self.start < self.end {
//             greater && lesser
//         } else {
//             greater || lesser
//         }
//     }
//
//     pub fn len(self) -> u32 {
//         debug_assert_ne!(self.start, self.end);
//         self.end.wrapping_sub(self.start) + 1
//     }
//
//     pub fn intersect(self, other: Self) -> Self {
//         // @Hack
//
//
//
//         Self { start, end }
//     }
// }


macro_rules! impl_load {
    ($($name:ident($width:literal)),+ $(,)?) => {
        $(
        impl ArbitraryContextualStillInstruction for ::rvhwfuzzer_encoding::$name {
            fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<Instruction> {
                let (rs, region) = ctx.take_memory_register()?;
                let rd = ctx.params_mut().take_register_dest();
                let rs_value = ctx.state().registers().get(rs).as_addr();
                let offset = ctx.generate_appropriate_offset(rs_value, region, $width);

                // eprintln!("rd = {rd:?}\nrs = {rs:?}\noffset = {offset}\nrs value = 0x{:08x}", rs_value);

                Some(Self::new(
                    rd,
                    rs,
                    offset,
                ).into())
            }
        }
        )+
    };
}

macro_rules! impl_store {
    ($($name:ident($width:literal)),+ $(,)?) => {
        $(
        impl ArbitraryContextualStillInstruction for ::rvhwfuzzer_encoding::$name {
            fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<Instruction> {
                let (rs1, region) = ctx.take_memory_register()?;
                let rs2 = ctx.params_mut().take_register_src();
                let rs1_value = ctx.state().registers().get(rs1).as_addr();
                let offset = ctx.generate_appropriate_offset(rs1_value, region, $width);

                // eprintln!("rs2 = {rs2:?}\nrs1 = {rs1:?}\noffset = {offset}\nrs value = 0x{:08x}", rs1_value);

                Some(Self::new(
                    rs1,
                    rs2,
                    offset,
                ).into())
            }
        }
        )+
    };
}

impl_load! {
    Lb(1),
    Lh(2),
    Lw(4),
    Lbu(1),
    Lhu(2),
}

impl_store! {
    Sb(1),
    Sh(2),
    Sw(4),
}
