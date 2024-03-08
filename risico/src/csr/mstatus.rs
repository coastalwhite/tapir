use crate::memory::Endianness;

use super::{CsrInitContext, Mode, CsrWriteContext};

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

macro_rules! bitfields {
    (
        $(
            $(#[$($attrss:meta)*])*
            $mask:ident, $get:ident, $set:ident = $offset:literal
        ),+ $(,)?
    ) => {
        $(
        $(#[$($attrss)*])*
        const $mask: u64 = 1 << $offset;

        $(#[$($attrss)*])*
        pub const fn $get(self) -> bool {
            self.0 & Self::$mask != 0
        }

        $(#[$($attrss)*])*
        pub fn $set(&mut self, value: bool) {
            self.0 &= !Self::$mask;
            self.0 |= u64::from(value) << $offset;
        }
        )+
    };
}

impl MStatus {
    /// Summarize dirty
    const SD_MASK: u64 = 1 << 31;

    /// Machine-mode Previous Privilege mode
    const MPP_MASK: u64 = 3 << 11;

    const MBE_MASK: u64 = 1 << (32 + 5);
    const SBE_MASK: u64 = 1 << (32 + 4);
    const UBE_MASK: u64 = 1 << 6;

    const LEAST_SUPPORTED_PRIVILEGE: Mode = Mode::User;

    bitfields! {
        /// Machine-mode Global Interrupt Enable
        MIE_MASK, mie, set_mie = 3,

        /// Machine-mode Previous Interrupt Enable
        MPIE_MASK, mpie, set_mpie =  7,
        
        /// Modify Privilege
        ///
        /// This modifies the effective privilege mode at which loads and stores execute
        MPRV_MASK, mprv, set_mprv =  17,
    }

    pub fn mstatush_read(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn mstatush_write(&mut self, value: u32, _: &CsrWriteContext) {
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

    pub fn write(&mut self, value: u32, _: &CsrWriteContext) {
        let mpp = self.mpp();

        self.0 &= 0xFFFF_FFFF_0000_0000;
        self.0 |= (value as u64) & !(Self::MIE_MASK);
        
        if !matches!(self.mpp(), Mode::Machine | Mode::User) {
            self.set_mpp(mpp);
        }
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

    pub const fn has_interrupts(self, mode: Mode) -> bool {
        self.mie() && matches!(mode, Mode::Machine)
    }

    pub fn trap_to_machine(&mut self, current_mode: Mode) {
        // RISC-V Specification:
        //
        // When a trap is taken from privilege mode y into privilege mode x, xPIE is set to the
        // value of xIE; xIE is set to 0; and xPP is set to y.

        self.set_mpie(self.mie());
        self.set_mie(false);
        self.set_mpp(current_mode);
    }

    /// Returns from a machine-mode trap, giving the new privilege mode.
    pub fn return_from_machine_trap(&mut self) -> Mode {
        // RISC-V Specification:
        //
        // When executing an xRET instruction, supposing xPP holds the value y, xIE is set to xPIE;
        // the privilege mode is changed to y; xPIE is set to 1; and xPP is set to the
        // least-privileged supported mode (U if U-mode is implemented, else M). If y̸=M, x RET also
        // sets MPRV=0.

        let previous_privilege = self.mpp();
        self.set_mie(self.mpie());
        self.set_mpie(true);
        self.set_mpp(Self::LEAST_SUPPORTED_PRIVILEGE);

        if !matches!(previous_privilege, Mode::Machine) {
            self.set_mprv(false);
        }

        previous_privilege
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

    /// Machine Mode Data Memory Access Big-Endian
    pub const fn mbe(self) -> Endianness {
        match self.0 & Self::MBE_MASK != 0 {
            false => Endianness::Little,
            true => Endianness::Big,
        }
    }

    /// Supervisor Mode Data Memory Access Big-Endian
    pub const fn sbe(self) -> Endianness {
        match self.0 & Self::SBE_MASK != 0 {
            false => Endianness::Little,
            true => Endianness::Big,
        }
    }

    /// User Mode Data Memory Access Big-Endian
    pub const fn ube(self) -> Endianness {
        match self.0 & Self::UBE_MASK != 0 {
            false => Endianness::Little,
            true => Endianness::Big,
        }
    }
}
