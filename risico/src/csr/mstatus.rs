use super::{CsrInitContext, Mode};

#[derive(Debug, Clone, Copy)]
pub struct MStatus(u64);

impl MStatus {
    const MIE_MASK: u64 = 1 << 3;
    const MPIE_MASK: u64 = 1 << 7;
    const MPP_MASK: u64 = 3 << 11;

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

    /// Machine Global Interrupt-Enable
    pub const fn mie(self) -> bool {
        self.0 & Self::MIE_MASK != 0
    }

    /// Machine Interrupt-Enable prior to Trap
    pub const fn mpie(self) -> bool {
        self.0 & Self::MPIE_MASK != 0
    }

    /// Set Machine Interrupt-Enable prior to Trap
    pub fn set_mpie(&mut self, value: bool) {
        self.0 &= !Self::MPIE_MASK;
        self.0 |= u64::from(value) << 7;
    }

    /// Machine Previous-Privilege Mode
    pub const fn mpp(self) -> Mode {
        Mode::take_masked(((self.0 >> 11) & 0xFFFF_FFFF) as u32)
    }

    /// Set Machine Previous-Privilege Mode
    pub fn set_mpp(&mut self, mode: Mode) {
        self.0 &= !Self::MPP_MASK;
        self.0 |= (mode as u64) << 11;
    }
}
