use super::CsrInitContext;

#[derive(Debug, Clone)]
pub struct MStatus(u64);

impl MStatus {
    pub fn mstatush_read(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn mstatush_write(&mut self, value: u32) {
        self.0 &= 0x0000_0000_FFFF_FFFF;
        self.0 |= (value as u64) << 32;
    }

    pub fn new(_: &CsrInitContext) -> Self {
        Self(0)
    }

    pub fn is_available() -> bool {
        true
    }

    pub fn read(&self) -> u32 {
        (self.0 & 0xFFFF_FFFF) as u32
    }

    pub fn write(&mut self, value: u32) {
        self.0 &= 0xFFFF_FFFF_0000_0000;
        self.0 |= value as u64;
    }
}
