use std::fmt;

use super::CsrInitContext;
use super::mtvec::TrapCause;

/// Machine Exception Delegation Register
///
/// This specifies which exceptions from U- or S-mode to directly transfer to S-mode.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MEDeleg {
    value: u32,
    readonly_mask: u32,
}

impl fmt::Debug for MEDeleg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MEDeleg")
            .field("value", &format!("0x{:08x}", self.value))
            .field("readonly_mask", &format!("0x{:08x}", self.readonly_mask))
            .finish()
    }
}

/// Machine Interrupt Delegation Register
///
/// This specifies which interrupts from U- or S-mode to directly transfer to S-mode.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MIDeleg(u32);

impl fmt::Debug for MIDeleg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MIDeleg")
            .field(&format!("0x{:08x}", self.0))
            .finish()
    }
}

impl MEDeleg {
    pub fn new(_: &CsrInitContext) -> Self {
        // 31-16, we explicitly disabled because they are not custom, reserved or not defined
        // 14: is reserved
        // 11: is only for M-mode
        //   RISC-V Privileged Specification:
        //   > For exceptions that cannot occur in less privileged modes, the corresponding medeleg
        //   > bits should be read-only zero. In particular, medeleg[11] is read-only zero.
        Self { value: 0, readonly_mask: 0xFFFF_4800 }
    }

    #[inline]
    pub fn is_available(&self) -> bool {
        true
    }

    #[inline]
    pub fn read(&self) -> u32 {
        self.value
    }

    #[inline]
    pub fn write(&mut self, value: u32) -> u32 {
        let old = self.value;
        self.value = value & self.readonly_mask;
        old
    }

    #[inline]
    pub fn is_delegated(self, cause: TrapCause) -> bool {
        self.value & (1 << cause as u32) != 0
    }
}

impl MIDeleg {
    pub fn new(_: &CsrInitContext) -> Self {
        Self(0)
    }

    #[inline]
    pub fn read(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn write(&mut self, value: u32) -> u32 {
        let old = self.0;
        self.0 = value;
        old
    }
}
