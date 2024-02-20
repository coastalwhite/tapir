use super::{CsrInitContext, Mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtStatus {
    /// The extension(s) state is disabled or not implemented and any interaction with said state
    /// results in an illegal instruction exception.
    Off = 0b00,

    /// The extension(s) state is enabled (for XS, at least 1 is enabled). The state is set as the
    /// initial state.
    Initial = 0b01,

    /// The extension(s) state is enabled (for XS, at least 1 is enabled). The state is the same as
    /// the last context switch and is potentially different from the `Initial` state.
    Clean = 0b10,

    /// The extension(s) state is enabled (for XS, at least 1 is enabled). The state is potentially
    /// changed since the last context switch.
    Dirty = 0b11,
}

impl ExtStatus {
    #[inline(always)]
    pub const fn take_masked(x: u8) -> Self {
        match x & 0b11 {
            0b00 => Self::Off,
            0b01 => Self::Initial,
            0b10 => Self::Clean,
            0b11 => Self::Dirty,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MStatus(u64);

impl MStatus {
    const SD_MASK: u64 = 1 << 31;
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

    /// Summarize dirty
    pub const fn sd(self) -> bool {
        matches!(self.fs(), ExtStatus::Dirty)
            | matches!(self.vs(), ExtStatus::Dirty)
            | matches!(self.xs(), ExtStatus::Dirty)
    }

    pub fn mark_fs_dirty(&mut self) {
        debug_assert_ne!(self.fs(), ExtStatus::Off);
        self.0 |= 0b11 << 13;
    }

    pub fn mark_vs_dirty(&mut self) {
        debug_assert_ne!(self.vs(), ExtStatus::Off);
        self.0 |= 0b11 << 9;
    }

    pub fn mark_xs_dirty(&mut self) {
        debug_assert_ne!(self.xs(), ExtStatus::Off);
        self.0 |= 0b11 << 15;
    }

    /// F-ext status
    pub const fn fs(self) -> ExtStatus {
        ExtStatus::take_masked(((self.0 >> 13) & 0xFF) as u8)
    }

    /// V-ext status
    pub const fn vs(self) -> ExtStatus {
        ExtStatus::take_masked(((self.0 >> 9) & 0xFF) as u8)
    }

    /// User-defined-ext status
    pub const fn xs(self) -> ExtStatus {
        ExtStatus::take_masked(((self.0 >> 15) & 0xFF) as u8)
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
