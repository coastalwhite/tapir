use risico::memory::PlacedBytes;
use rvhwfuzzer_encoding::{XRegIdent, FRegIdent, RoundingMode, Instruction};

mod fpu32;

pub struct ArbitraryGenerationContext<P: ArbitraryParameterProvider> {
    pub state: risico::State<PlacedBytes>,
    pub parameter_provider: P,
}

pub trait ArbitraryParameterProvider {
    fn take_register_src(&mut self) -> XRegIdent;
    fn take_register_dest(&mut self) -> XRegIdent;

    fn take_fpu_register_src(&mut self) -> FRegIdent;
    fn take_fpu_register_dest(&mut self) -> FRegIdent;
    fn take_static_rounding_mode(&mut self) -> RoundingMode;
    fn take_rounding_mode(&mut self) -> RoundingMode;

    fn take_immediate(&mut self, bitsize: u32) -> u64;
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
}

impl<P: ArbitraryParameterProvider> ArbitraryGenerationContext<P> {
    pub fn state(&self) -> &risico::State<PlacedBytes> {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut risico::State<PlacedBytes> {
        &mut self.state
    }

    pub fn params(&self) -> &P {
        &self.parameter_provider
    }

    pub fn params_mut(&mut self) -> &mut P {
        &mut self.parameter_provider
    }
}

pub trait ArbitraryInstruction {
    fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Instruction;
}

macro_rules! impl_regreg_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
            fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Instruction {
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
            fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Instruction {
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
            fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Instruction {
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
            fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Instruction {
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
