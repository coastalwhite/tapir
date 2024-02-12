use rvhwfuzzer_encoding::Instruction;

use crate::arbitrary::{
    ArbitraryGenerationContext, ArbitraryInstruction,
};

#[derive(Debug)]
pub struct NonHoppingBranch;

impl ArbitraryInstruction for NonHoppingBranch {
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
