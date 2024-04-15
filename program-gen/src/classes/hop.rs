use crate::arbitrary::{ArbitraryHopInstruction, Branch, Jump, HopTarget};

pub struct Hop;

impl ArbitraryHopInstruction for Hop {
    fn take(ctx: &mut crate::arbitrary::ArbitraryGenerationContext) -> (rvhwfuzzer_encoding::Instruction, HopTarget) {
        let n = fastrand::u8(0..2);

        match n {
            0 => Jump::take(ctx),
            1 => Branch::take(ctx),
            _ => unreachable!(),
        }
    }
}
