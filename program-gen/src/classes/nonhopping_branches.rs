use std::io;

use risico::RegIdent;

use crate::arbitrary::{
    ArbitraryGenerationContext, ArbitraryInstruction, ArbitraryParameterProvider,
};
use crate::RegisterId;

#[derive(Debug)]
pub enum NonHoppingBranch {
    Equality(NonHoppingEqualityBranch),
    Inequality(NonHoppingInequalityBranch),
}

#[derive(Debug)]
pub struct NonHoppingEqualityBranch {
    rs1: RegisterId,
    rs2: RegisterId,
    imm12_1: u16,
    is_equal: bool,
}

#[derive(Debug)]
pub struct NonHoppingInequalityBranch {
    rs1: RegisterId,
    rs2: RegisterId,
    imm12_1: u16,
    is_signed: bool,
    is_less_than: bool,
}

impl ArbitraryInstruction for NonHoppingBranch {
    fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self {
        let r = ctx.params_mut().take_u8(1);

        if r == 0 {
            Self::Equality(NonHoppingEqualityBranch::take(ctx))
        } else {
            Self::Inequality(NonHoppingInequalityBranch::take(ctx))
        }
    }
}

impl ArbitraryInstruction for NonHoppingEqualityBranch {
    fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self {
        let rs1 = ctx.params_mut().take_register_src();
        let rs2 = ctx.params_mut().take_register_src();

        let rs1_value = ctx.state().registers().get(RegIdent::from(rs1.0));
        let rs2_value = ctx.state().registers().get(RegIdent::from(rs2.0));

        let imm12_1 = ctx.params_mut().take_u16(11);

        let is_equal = rs1_value == rs2_value;

        Self {
            rs1,
            rs2,
            imm12_1,
            is_equal,
        }
    }
}

impl ArbitraryInstruction for NonHoppingInequalityBranch {
    fn take<P: ArbitraryParameterProvider>(ctx: &mut ArbitraryGenerationContext<P>) -> Self {
        let rs1 = ctx.params_mut().take_register_src();
        let rs2 = ctx.params_mut().take_register_src();

        let rs1_value = ctx.state().registers().get(RegIdent::from(rs1.0));
        let rs2_value = ctx.state().registers().get(RegIdent::from(rs2.0));

        let imm12_1 = ctx.params_mut().take_u16(11);

        let is_signed = rand::random();

        let is_less_than = if is_signed {
            rs1_value.as_i32() < rs2_value.as_i32()
        } else {
            rs1_value.as_u32() < rs2_value.as_u32()
        };

        Self {
            rs1,
            rs2,
            imm12_1,
            is_signed,
            is_less_than,
        }
    }
}

impl NonHoppingBranch {
    pub fn encode(&self, writer: &mut impl io::Write) -> io::Result<()> {
        match self {
            NonHoppingBranch::Equality(v) => v.encode(writer),
            NonHoppingBranch::Inequality(v) => v.encode(writer),
        }
    }
}

impl NonHoppingInequalityBranch {
    pub fn encode(&self, writer: &mut impl io::Write) -> io::Result<()> {
        let imm4_1 = self.imm12_1 & 0xF;
        let imm10_5 = (self.imm12_1 >> 4) & 0x3F;
        let imm11 = (self.imm12_1 >> 10) & 1;
        let imm12 = (self.imm12_1 >> 11) & 1;

        let imm4_1 = imm4_1 as u8;
        let imm10_5 = imm10_5 as u8;
        let imm11 = imm11 as u8;
        let imm12 = imm12 as u8;

        let rs1 = self.rs1.0;
        let rs2 = self.rs2.0;

        match (self.is_less_than, self.is_signed) {
            (false, true) => rvhwfuzzer_encoding::BltArgs {
                imm10_5,
                imm4_1,
                imm11,
                rs1,
                imm12,
                rs2,
            }
            .encode(writer),
            (true, true) => rvhwfuzzer_encoding::BgeArgs {
                imm10_5,
                imm4_1,
                imm11,
                rs1,
                imm12,
                rs2,
            }
            .encode(writer),
            (false, false) => rvhwfuzzer_encoding::BltuArgs {
                imm10_5,
                imm4_1,
                imm11,
                rs1,
                imm12,
                rs2,
            }
            .encode(writer),
            (true, false) => rvhwfuzzer_encoding::BgeuArgs {
                imm10_5,
                imm4_1,
                imm11,
                rs1,
                imm12,
                rs2,
            }
            .encode(writer),
        }
    }
}

impl NonHoppingEqualityBranch {
    pub fn encode(&self, writer: &mut impl io::Write) -> io::Result<()> {
        let imm4_1 = self.imm12_1 & 0xF;
        let imm10_5 = (self.imm12_1 >> 4) & 0x3F;
        let imm11 = (self.imm12_1 >> 10) & 1;
        let imm12 = (self.imm12_1 >> 11) & 1;

        let imm4_1 = imm4_1 as u8;
        let imm10_5 = imm10_5 as u8;
        let imm11 = imm11 as u8;
        let imm12 = imm12 as u8;

        let rs1 = self.rs1.0;
        let rs2 = self.rs2.0;

        if self.is_equal {
            rvhwfuzzer_encoding::BneArgs {
                imm10_5,
                imm4_1,
                imm11,
                rs1,
                imm12,
                rs2,
            }
            .encode(writer)
        } else {
            rvhwfuzzer_encoding::BeqArgs {
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
