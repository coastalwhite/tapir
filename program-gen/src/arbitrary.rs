use risico::memory::PlacedBytes;

mod fpu32;

use crate::{RegisterId, FPURegisterId};

pub struct ArbitraryGenerationContext<P: ArbitraryParameterProvider> {
    pub state: risico::State<PlacedBytes>,
    pub parameter_provider: P,
}

#[repr(u8)]
#[rustfmt::skip]
pub enum RoundingMode {
    TiesToEven     = 0b000,
    ToZero         = 0b001,
    Down           = 0b010,
    Up             = 0b011,
    ToMaxMagnitude = 0b100,
    Dynamic        = 0b111,
}

pub trait ArbitraryParameterProvider {
    fn take_register_src(&mut self) -> RegisterId;
    fn take_register_dest(&mut self) -> RegisterId;

    fn take_fpu_register_src(&mut self) -> FPURegisterId;
    fn take_fpu_register_dest(&mut self) -> FPURegisterId;
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
    fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self;
}

macro_rules! impl_regreg_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
            fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self {
                Self {
                    rs1: ctx.params_mut().take_register_src().0,
                    rs2: ctx.params_mut().take_register_src().0,
                    rd: ctx.params_mut().take_register_dest().0,
                }
            }
        }
        )+
    };
}

macro_rules! impl_regimm_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
            fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self {
                Self {
                    rs: ctx.params_mut().take_register_src().0,
                    imm11_0: ctx.params_mut().take_u16(12),
                    rd: ctx.params_mut().take_register_dest().0,
                }
            }
        }
        )+
    };
}

macro_rules! impl_shiftimm_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
            fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self {
                Self {
                    rs: ctx.params_mut().take_register_src().0,
                    shamt: ctx.params_mut().take_u8(5),
                    rd: ctx.params_mut().take_register_dest().0,
                }
            }
        }
        )+
    };
}

macro_rules! impl_reghighimm_args {
    ($($name:ident),+ $(,)?) => {
        $(
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
            fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self {
                Self {
                    imm31_12: ctx.params_mut().take_u32(20),
                    rd: ctx.params_mut().take_register_dest().0,
                }
            }
        }
        )+
    };
}

impl_regreg_args! {
    AddArgs,
    SubArgs,
    SllArgs,
    SltArgs,
    SltuArgs,
    XorArgs,
    SrlArgs,
    SraArgs,
    OrArgs,
    AndArgs,
}

impl_regimm_args! {
    AddiArgs,
    SltiArgs,
    SltiuArgs,
    XoriArgs,
    OriArgs,
    AndiArgs,
}

impl_reghighimm_args! { LuiArgs, AuipcArgs }

impl_shiftimm_args! { SlliArgs, SrliArgs, SraiArgs }
