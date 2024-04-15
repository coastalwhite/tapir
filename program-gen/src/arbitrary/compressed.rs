use rvhwfuzzer_encoding::{
    CAdd, CAddi, CAddi16Sp, CAddi4SpN, CAnd, CAndi, CLi, CLui, CMv, CNop, COr, CSlli, CSrai, CSrli,
    CSub, CXor, XRegIdent,
};

use super::ArbitraryStillInstruction;

macro_rules! exclude_values {
    ($value:ident, $($excluded:expr),+ => $fallback:expr) => {
        $value = $(if $value == $excluded {
            $fallback
        } else )+ {
            $value
        }
    };
}

impl ArbitraryStillInstruction for CAddi16Sp {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let imm = ctx.params_mut().take_i16(6);
        let imm = imm.max(1); // Not-Zero
        let imm = imm << 4; // Multiples of 16
        CAddi16Sp::new(imm).into()
    }
}

impl ArbitraryStillInstruction for CAddi4SpN {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let rd = ctx.params_mut().take_compressed_register_dest();
        let imm = ctx.params_mut().take_u32(8);
        let imm = imm.max(1); // Not-Zero
        let imm = imm << 2; // Multiples of 4
        CAddi4SpN::new(rd, imm).into()
    }
}

impl ArbitraryStillInstruction for CAddi {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let mut rd_rs1 = ctx.params_mut().take_register_src();
        exclude_values!(rd_rs1, XRegIdent::Zero => XRegIdent::A0);
        let imm = ctx.params_mut().take_i8(6);
        let imm = imm.max(1); // Not-Zero
        CAddi::new(rd_rs1, imm).into()
    }
}

impl ArbitraryStillInstruction for CLi {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let mut rd = ctx.params_mut().take_register_dest();
        exclude_values!(rd, XRegIdent::Zero => XRegIdent::A0);
        let imm = ctx.params_mut().take_i8(6);
        CAddi::new(rd, imm).into()
    }
}

impl ArbitraryStillInstruction for CLui {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let mut rd = ctx.params_mut().take_register_dest(); // Non-Zero
        exclude_values!(rd, XRegIdent::Zero, XRegIdent::take_masked(2) => XRegIdent::A0);
        let imm = ctx.params_mut().take_i32(6);
        let imm = imm.max(1); // Not-Zero
        let imm = imm << 12; // Shifted by 12
        CLui::new(rd, imm).into()
    }
}

impl ArbitraryStillInstruction for CSlli {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let rd_rs1 = ctx.params_mut().take_register_src();
        let imm = ctx.params_mut().take_u8(5);
        CSlli::new(rd_rs1, imm).into()
    }
}

impl ArbitraryStillInstruction for CSrli {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        // @TODO: Make a special function for RD_RS
        let rd_rs1 = ctx.params_mut().take_compressed_register_src();
        let imm = ctx.params_mut().take_u8(5);
        CSrli::new(rd_rs1, imm).into()
    }
}

impl ArbitraryStillInstruction for CSrai {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let rd_rs1 = ctx.params_mut().take_compressed_register_src();
        let imm = ctx.params_mut().take_u8(5);
        CSrai::new(rd_rs1, imm).into()
    }
}

impl ArbitraryStillInstruction for CAndi {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let rd_rs1 = ctx.params_mut().take_compressed_register_src();
        let imm = ctx.params_mut().take_i8(5);
        CAndi::new(rd_rs1, imm).into()
    }
}

impl ArbitraryStillInstruction for CAdd {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let mut rd_rs1 = ctx.params_mut().take_register_src();
        let mut rs2 = ctx.params_mut().take_register_src();

        exclude_values!(rd_rs1, XRegIdent::Zero => XRegIdent::A0);
        exclude_values!(rs2, XRegIdent::Zero => XRegIdent::A0);

        CAdd::new(rd_rs1, rs2).into()
    }
}

impl ArbitraryStillInstruction for CMv {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let mut rd_rs1 = ctx.params_mut().take_register_src();
        let mut rs2 = ctx.params_mut().take_register_src();

        exclude_values!(rd_rs1, XRegIdent::Zero => XRegIdent::A0);
        exclude_values!(rs2, XRegIdent::Zero => XRegIdent::A0);

        CMv::new(rd_rs1, rs2).into()
    }
}

impl ArbitraryStillInstruction for CAnd {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let rd_rs1 = ctx.params_mut().take_compressed_register_src();
        let rs2 = ctx.params_mut().take_compressed_register_src();

        CAnd::new(rd_rs1, rs2).into()
    }
}

impl ArbitraryStillInstruction for COr {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let rd_rs1 = ctx.params_mut().take_compressed_register_src();
        let rs2 = ctx.params_mut().take_compressed_register_src();

        COr::new(rd_rs1, rs2).into()
    }
}

impl ArbitraryStillInstruction for CXor {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let rd_rs1 = ctx.params_mut().take_compressed_register_src();
        let rs2 = ctx.params_mut().take_compressed_register_src();

        CXor::new(rd_rs1, rs2).into()
    }
}

impl ArbitraryStillInstruction for CSub {
    fn take(ctx: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        let rd_rs1 = ctx.params_mut().take_compressed_register_src();
        let rs2 = ctx.params_mut().take_compressed_register_src();

        CSub::new(rd_rs1, rs2).into()
    }
}

impl ArbitraryStillInstruction for CNop {
    fn take(_: &mut super::ArbitraryGenerationContext) -> rvhwfuzzer_encoding::Instruction {
        CNop::new().into()
    }
}
