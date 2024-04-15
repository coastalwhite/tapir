use rvhwfuzzer_encoding::{Instruction, XRegIdent};

use super::{ArbitraryGenerationContext, ArbitraryHopInstruction, ArbitraryStillInstruction, Branch, HopTarget, Jump};

#[derive(Debug)]
pub struct NonHoppingBranch;

const MIN_PADDING: u32 = 16;
const MAX_PADDING: u32 = 64;

impl ArbitraryHopInstruction for Jump {
    fn take(_ctx: &mut ArbitraryGenerationContext) -> (Instruction, HopTarget) {
        // @TODO: Use context to generate random numbers
        let padding = fastrand::u32(MIN_PADDING..MAX_PADDING);
        let padding = padding & !0x3;
        let offset = padding + 4;

        let hop_target = HopTarget::Padded(padding);

        (rvhwfuzzer_encoding::Jal::new(XRegIdent::Zero, offset as i32).into(), hop_target)
    }
}

impl ArbitraryHopInstruction for Branch {
    fn take(ctx: &mut ArbitraryGenerationContext) -> (Instruction, HopTarget) {
        // @TODO: Use context to generate random numbers
        let padding = fastrand::u32(MIN_PADDING..MAX_PADDING);
        let padding = padding & !0x3;
        let offset = padding + 4;

        let rs1 = ctx.params_mut().take_register_src();
        let rs2 = ctx.params_mut().take_register_src();

        let rs1_value = ctx.state().registers().get(rs1);
        let rs2_value = ctx.state().registers().get(rs2);

        let hop_target = HopTarget::Padded(padding);

        // @TODO: Add inequality branches
        if rs1_value == rs2_value {
            (rvhwfuzzer_encoding::Beq::new(rs1, rs2, offset as i16).into(), hop_target)
        } else {
            (rvhwfuzzer_encoding::Bne::new(rs1, rs2, offset as i16).into(), hop_target)
        }
    }
}

impl ArbitraryStillInstruction for Branch {
    fn take(ctx: &mut ArbitraryGenerationContext) -> Instruction {
        use rvhwfuzzer_encoding as I;

        let r = ctx.params_mut().take_u8(1);

        let rs1 = ctx.params_mut().take_register_src();
        let rs2 = ctx.params_mut().take_register_src();

        let rs1_value = ctx.state().registers().get(rs1);
        let rs2_value = ctx.state().registers().get(rs2);

        let imm = ctx.params_mut().take_u16(12) << 1;

        if r == 0 {
            let is_equal = rs1_value == rs2_value;

            if is_equal {
                I::Bne::new(rs1, rs2, imm as i16).into()
            } else {
                I::Beq::new(rs1, rs2, imm as i16).into()
            }
        } else {
            let is_signed = ctx.params_mut().take_u8(1);

            if is_signed == 0 {
                let is_less_than = rs1_value.as_i32() < rs2_value.as_i32();

                if is_less_than {
                    I::Bge::new(rs1, rs2, imm as i16).into()
                } else {
                    I::Blt::new(rs1, rs2, imm as i16).into()
                }
            } else {
                let is_less_than = rs1_value.as_u32() < rs2_value.as_u32();

                if is_less_than {
                    I::Bgeu::new(rs1, rs2, imm as i16).into()
                } else {
                    I::Bltu::new(rs1, rs2, imm as i16).into()
                }
            }
        }
    }
}
