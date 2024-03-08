use super::{CsrInitContext, CsrWriteContext};

#[derive(Debug, Clone)]
pub struct Counter(u64);

impl Counter {
    pub fn new(_: &CsrInitContext) -> Self {
        Self(0)
    }

    #[inline]
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    #[inline]
    pub fn low_read(&self) -> u32 {
        (self.0 & 0xFFFF_FFFF) as u32
    }

    #[inline]
    pub fn low_write(&mut self, value: u32, _: &CsrWriteContext) {
        self.0 &= 0xFFFF_FFFF_0000_0000;
        self.0 |= u64::from(value);
    }

    #[inline]
    pub fn high_read(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    #[inline]
    pub fn high_write(&mut self, value: u32, _: &CsrWriteContext) {
        self.0 &= 0xFFFF_FFFF;
        self.0 |= u64::from(value) << 32;
    }
}
