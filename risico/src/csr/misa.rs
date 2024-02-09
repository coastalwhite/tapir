use std::str::FromStr;

use super::CsrInitContext;

#[derive(Debug, Clone)]
pub struct MIsa(rvisa::MIsa);

impl MIsa {
    pub fn new(_: &CsrInitContext) -> Self {
        Self(rvisa::MIsa::from_str("rv32if").unwrap())
    }

    pub fn read(&self) -> u32 {
        self.0.as_u32()
    }

    pub fn write(&mut self, value: u32) {
        self.0 = rvisa::MIsa::from_u32(value);
    }
}
