use rvisa::MIsaExt;

use super::{CsrInitContext, CsrWriteContext};

#[derive(Debug, Clone, Copy)]
pub struct MIsa(rvisa::MIsa);

impl MIsa {
    pub fn new(ctx: &CsrInitContext) -> Self {
        Self(ctx.isa)
    }

    pub fn read(&self) -> u32 {
        self.0.as_u32()
    }

    pub fn isa(self) -> rvisa::MIsa {
        self.0
    }

    pub fn write(&mut self, value: u32, ctx: &CsrWriteContext) {
        self.0 = rvisa::MIsa::from_u32(value);

        let cannot_turn_off_compressed =
            !ctx.pc.is_word_aligned() && !self.0.contains(MIsaExt::COMPRESSED);

        if cannot_turn_off_compressed {
            self.0 = self.0.with_ext(MIsaExt::COMPRESSED);
        }
    }
}
