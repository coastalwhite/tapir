use super::CsrInitContext;

#[derive(Debug, Clone, Copy)]
pub struct MCause(u32);

impl MCause {
    #[inline]
    pub fn new(_: &CsrInitContext) -> Self {
        Self(0)
    }

    #[inline]
    pub fn read(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn write(&mut self, value: u32) {
        self.0 = value;
    }

    #[inline]
    pub fn interrupt(self) -> bool {
        self.0 >> 31 != 0
    }

    #[inline]
    pub fn exception(self) -> u32 {
        self.0 & 0x7FFF_FFFF
    }
}
