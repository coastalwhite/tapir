use std::collections::HashSet;

use risico::memory::BackingStore;
use risico::repr::Addr;
use rvhwfuzzer_encoding::{Instruction, XRegIdent};

use crate::backing_store::ProgramMemory;
use crate::{RegisterRecencyList, NUM_REGISTERS};

mod compressed;
mod fpu32;

pub enum HopTarget {
    Padded(u32),
}

pub struct MemWord {
    pub addr: Addr,
    pub value: u32,
}

pub struct GeneratedMemory {
    has_generated: HashSet<Addr>,
    values: Vec<MemWord>,
}

impl GeneratedMemory {
    pub fn new() -> Self {
        Self {
            has_generated: HashSet::new(),
            values: Vec::new(),
        }
    }

    pub fn generate(&mut self, addr: Addr) -> Option<u32> {
        // Check that the addr is word aligned
        debug_assert!(addr.is_word_aligned());

        let did_exist = self.has_generated.insert(addr);

        if did_exist {
            return None;
        }

        let value = fastrand::u32(..);
        self.values.push(MemWord { addr, value });

        Some(value)
    }
}

pub struct ArbitraryGenerationContext {
    pub hop_target: HopTarget,
    pub state: risico::State<ProgramMemory>,
    pub parameter_provider: RegisterRecencyList,
    pub data_memory_ranges: Vec<std::ops::Range<u32>>,
    pub potential_memory_registers: Vec<(XRegIdent, std::ops::Range<u32>)>,
}

fn addr_in_range_of_memory_region(addr: Addr, start: u32, end: u32) -> bool {
    // @Hack: This should be a WrappingRange, not a normal range
    let delta_start = addr.as_u32().saturating_sub(start);
    let delta_end = end.saturating_add(addr.as_u32());

    // 12-bit signed immediate, plus a margin of 8
    const MAX_OFFSET: u32 = (1 << 11) - 8 - 12;

    // let min_abs_diff = u32::min(
    //     addr.as_u32().abs_diff(start),
    //     addr.as_u32().abs_diff(end),
    //     );
    //
    // dbg!(min_abs_diff);

    delta_start < MAX_OFFSET && delta_end < MAX_OFFSET
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

    #[inline(always)]
    pub fn params(&self) -> &RegisterRecencyList {
        &self.parameter_provider
    }

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

        self.state_mut()
            .memory_mut()
            .memory_areas
            .initialize_fill(addr.as_u32()..addr.as_u32() + u32::from(width), || fastrand::u8(..))
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
                    println!("[lw] POTENTIAL REGISTER");
                    break;
                }
            }
        }
    }

    // pub fn take_memory_register(&self) -> Option<(XRegIdent, std::ops::Range<u32>)> {
    //     // @Hack
    //     //
    //     // This is like insanely slow... Like taking up 75% of the runtime slow.  This should be
    //     // fixed for sure. Maybe, we just introduce top level methods to keep track of which
    //     // registers might contain memory addresses or this just needs to be changed fully to a
    //     // more general model where we create memory registers somehow. I am not 100% sure yeth
    //     // how, but this is just too slow.
    //     //
    //     // @Note
    //     // Caching also doesn't really work here, because the negative case is too common.
    //     //
    //     // @Note
    //     // We introduce randomness here to allow for different patterns in the accesses to memory.
    //     //
    //     // For example:
    //     //  LW a0,0[<D1>]
    //     //  SW a0,0[<D2>]
    //     //
    //     // This would be impossible to achieve without this kind of randomness.
    //     //
    //     // This is really expensive though as otherwise we would just cache these things. Maybe, it
    //     // is worth it do some form of intermediate solution. Maybe, we can cache several things
    //     // recalculate only every N times and try to take from the cache in the mean time. That way
    //     // we don't have to check each time.
    //
    //     let register_offset = fastrand::usize(0..=31);
    //     for i in 0..31 {
    //         let rs = self.params().inner[(i + register_offset) % 31];
    //         let addr = self.state().registers().get(rs).as_addr();
    //
    //         let region_offset = fastrand::usize(0..=31);
    //         for j in 0..self.state().memory().memory_areas.len() {
    //             let region = self.state().memory().memory_areas
    //                 [(region_offset + j) % self.state().memory().memory_areas.len()]
    //             .range();
    //
    //             if addr_in_range_of_memory_region(addr, region.start, region.end) {
    //                 return Some((rs, region));
    //             }
    //         }
    //     }
    //
    //     None
    // }
    //
    // // Cached
    // pub fn take_memory_register(&mut self) -> Option<(XRegIdent, std::ops::Range<u32>)> {
    //     if let Some((r, region)) = self.memory_register_cache.clone() {
    //         let addr = self.state().registers().get(r).as_addr();
    //         if addr_in_range_of_memory_region(addr, region.start, region.end) {
    //             return Some((r, region));
    //         }
    //     }
    //
    //     let output = self.uncached_take_memory_register();
    //     self.memory_register_cache = output.clone();
    //     output
    // }
}

pub trait ArbitraryInstruction {
    fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction;
}

impl<T: ArbitraryInstruction> ArbitraryContextualInstruction for T {
    #[inline(always)]
    fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<Instruction> {
        Some(Self::take(ctx))
    }
}

pub trait ArbitraryContextualInstruction {
    fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<Instruction>;
}

macro_rules! impl_regreg_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
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
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
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
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
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
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
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

fn generate_appropriate_offset(addr: Addr, region: std::ops::Range<u32>, width: u32) -> i16 {
    let addr_min = addr.as_u32().saturating_sub(1u32 << 11);
    let addr_max = addr.as_u32().saturating_add((1u32 << 11) - 1);

    const MIN_REGION_BYTES: u32 = 8;

    debug_assert!(width <= MIN_REGION_BYTES, "Width is too large");
    debug_assert!(
        region.len() >= MIN_REGION_BYTES as usize,
        "Regions are expected to be at least {} bytes long",
        MIN_REGION_BYTES
    );

    eprintln!("addr: 0x{:08x}", addr.as_u32());
    eprintln!("addr_min: 0x{addr_min:08x}, addr_max: 0x{addr_max:08x}, before");

    let addr_min = addr_min.max(region.start);
    let addr_max = addr_max.min(region.end - width - 1);

    eprintln!(
        "addr_min: 0x{addr_min:08x}, addr_max: 0x{addr_max:08x}, region: 0x{:08x}..0x{:08x}",
        region.start, region.end
    );

    let offset = fastrand::u32(0..addr_max - addr_min);
    let result_addr = addr_min.wrapping_add(offset);

    result_addr.wrapping_sub(addr.as_u32()) as i32 as i16
}

macro_rules! impl_load {
    ($($name:ident($width:literal)),+ $(,)?) => {
        $(
        impl ArbitraryContextualInstruction for ::rvhwfuzzer_encoding::$name {
            fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<Instruction> {
                let (rs, region) = ctx.take_memory_register()?;
                let rd = ctx.params_mut().take_register_dest();
                let rs_value = ctx.state().registers().get(rs).as_addr();
                let offset = generate_appropriate_offset(rs_value, region, $width);

                eprintln!("rd = {rd:?}\nrs = {rs:?}\noffset = {offset}\nrs value = 0x{:08x}", rs_value);

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
        impl ArbitraryContextualInstruction for ::rvhwfuzzer_encoding::$name {
            fn try_take(ctx: &mut ArbitraryGenerationContext) -> Option<Instruction> {
                let (rs1, region) = ctx.take_memory_register()?;
                let rs2 = ctx.params_mut().take_register_src();
                let rs1_value = ctx.state().registers().get(rs1).as_addr();
                let offset = generate_appropriate_offset(rs1_value, region, $width);

                eprintln!("rs2 = {rs2:?}\nrs1 = {rs1:?}\noffset = {offset}\nrs value = 0x{:08x}", rs1_value);

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
