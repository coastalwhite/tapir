use rvhwfuzzer_encoding::Instruction;

use crate::RegisterRecencyList;
use crate::backing_store::ProgramMemory;

mod fpu32;
mod compressed;

pub enum HopTarget {
    Padded(u32),
}

pub struct ArbitraryGenerationContext {
    pub hop_target: HopTarget,
    pub state: risico::State<ProgramMemory>,
    pub parameter_provider: RegisterRecencyList,
}

impl ArbitraryGenerationContext {
    pub fn state(&self) -> &risico::State<ProgramMemory> {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut risico::State<ProgramMemory> {
        &mut self.state
    }

    pub fn take_state(self) -> risico::State<ProgramMemory> {
        self.state
    }

    pub fn params(&self) -> &RegisterRecencyList {
        &self.parameter_provider
    }

    pub fn params_mut(&mut self) -> &mut RegisterRecencyList {
        &mut self.parameter_provider
    }
}

pub trait ArbitraryInstruction {
    #[inline(always)]
    fn is_available(_ctx: &ArbitraryGenerationContext) -> bool {
        true
    }

    fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction;
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
