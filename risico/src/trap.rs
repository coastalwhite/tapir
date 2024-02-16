use std::ops::{BitOr, BitOrAssign};

use crate::csr::mtvec::TrapCause;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrapBehavior(u64);

impl TrapBehavior {
    pub const ABORT_ON_INSTR_ADDRESS_MISALIGNED: Self = Self(1 << 0);
    pub const ABORT_ON_INSTR_ACCESS_FAULT: Self = Self(1 << 1);
    pub const ABORT_ON_ILLEGAL_INSTRUCTION: Self = Self(1 << 2);
    pub const ABORT_ON_BREAKPOINT: Self = Self(1 << 3);
    pub const ABORT_ON_LOAD_ADDR_MISALIGNED: Self = Self(1 << 4);
    pub const ABORT_ON_LOAD_ACCESS_FAULT: Self = Self(1 << 5);
    pub const ABORT_ON_STORE_AMO_ADDRESS_MISALIGNED: Self = Self(1 << 6);
    pub const ABORT_ON_STORE_AMO_ACCESS_FAULT: Self = Self(1 << 7);
    pub const ABORT_ON_ENV_CALL_FROM_U_MODE: Self = Self(1 << 8);
    pub const ABORT_ON_ENV_CALL_FROM_S_MODE: Self = Self(1 << 9);
    const ABORT_ON_RESERVED_10: Self = Self(1 << 10);
    pub const ABORT_ON_ENV_CALL_FROM_M_MODE: Self = Self(1 << 11);
    pub const ABORT_ON_INSTR_PAGE_FAULT: Self = Self(1 << 12);
    pub const ABORT_ON_LOAD_PAGE_FAULT: Self = Self(1 << 13);
    const ABORT_ON_RESERVED_14: Self = Self(1 << 14);
    pub const ABORT_ON_STORE_AMO_PAGE_FAULT: Self = Self(1 << 15);

    pub const ABORT_ON_MISSING_CSR: Self = Self(1 << 16);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn abort_all() -> Self {
        Self(0xFFFF)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn abort_on_trap(trap: TrapCause) -> Self {
        Self(1 << (trap as u32))
    }

    pub const fn does_abort_on(self, exception_code: u32) -> bool {
        let exception_code = exception_code & 0xF;
        let exception_code = exception_code as u64;
        self.0 & exception_code == exception_code
    }

    pub const fn does_abort_on_missing_csr(self) -> bool {
        self.contains(Self::ABORT_ON_MISSING_CSR)
    }
}

impl BitOr for TrapBehavior {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for TrapBehavior {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
