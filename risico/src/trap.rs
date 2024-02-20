use std::ops::{BitOr, BitOrAssign};

use crate::csr::mtvec::TrapCause;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TrapBehavior(u32);

impl std::fmt::Debug for TrapBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TrapBehavior as T;

        f.debug_struct("TrapBehavior")
            .field("all", &self.contains(T::abort_all()))
            .field(
                "instr_misaligned",
                &self.contains(T::ABORT_ON_INSTR_ADDRESS_MISALIGNED),
            )
            .field(
                "instr_access_fault",
                &self.contains(T::ABORT_ON_INSTR_ACCESS_FAULT),
            )
            .field(
                "illegal_instr",
                &self.contains(T::ABORT_ON_ILLEGAL_INSTRUCTION),
            )
            .field("breakpoint", &self.contains(T::ABORT_ON_BREAKPOINT))
            .field(
                "load_misaligned",
                &self.contains(T::ABORT_ON_LOAD_ADDR_MISALIGNED),
            )
            .field(
                "load_access_fault",
                &self.contains(T::ABORT_ON_LOAD_ACCESS_FAULT),
            )
            .field(
                "store_amo_misaligned",
                &self.contains(T::ABORT_ON_STORE_AMO_ADDRESS_MISALIGNED),
            )
            .field(
                "store_amo_access_fault",
                &self.contains(T::ABORT_ON_STORE_AMO_ACCESS_FAULT),
            )
            .field(
                "smode_ecall",
                &self.contains(T::ABORT_ON_ENV_CALL_FROM_S_MODE),
            )
            .field(
                "umode_ecall",
                &self.contains(T::ABORT_ON_ENV_CALL_FROM_U_MODE),
            )
            // .field("reserved10"          , &self.contains( T::ABORT_ON_RESERVED_10,),)
            .field(
                "mmode_ecall",
                &self.contains(T::ABORT_ON_ENV_CALL_FROM_M_MODE),
            )
            .field(
                "instr_page_fault",
                &self.contains(T::ABORT_ON_INSTR_PAGE_FAULT),
            )
            .field(
                "load_page_fault",
                &self.contains(T::ABORT_ON_LOAD_PAGE_FAULT),
            )
            // .field("reserved14"          , &self.contains( T::ABORT_ON_RESERVED_14,),)
            .field(
                "store_amo_page_fault",
                &self.contains(T::ABORT_ON_STORE_AMO_PAGE_FAULT),
            )
            .field("missing_csr", &self.contains(T::ABORT_ON_MISSING_CSR))
            .finish()
    }
}

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
    // const ABORT_ON_RESERVED_10: Self = Self(1 << 10);
    pub const ABORT_ON_ENV_CALL_FROM_M_MODE: Self = Self(1 << 11);
    pub const ABORT_ON_INSTR_PAGE_FAULT: Self = Self(1 << 12);
    pub const ABORT_ON_LOAD_PAGE_FAULT: Self = Self(1 << 13);
    // const ABORT_ON_RESERVED_14: Self = Self(1 << 14);
    pub const ABORT_ON_STORE_AMO_PAGE_FAULT: Self = Self(1 << 15);

    pub const ABORT_ON_MISSING_CSR: Self = Self(1 << 16);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn abort_all() -> Self {
        Self(0xFFFF)
    }

    pub const fn abort_on_trap(trap: TrapCause) -> Self {
        Self(1 << (trap as u32))
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn does_abort_on(self, exception_code: u32) -> bool {
        let exception_code = exception_code & 0xF;
        self.0 & exception_code == exception_code
    }

    pub const fn does_abort_on_missing_csr(self) -> bool {
        self.0 & Self::ABORT_ON_MISSING_CSR.0 != 0
    }

    pub const fn set_minus(mut self, other: Self) -> Self {
        self.0 &= !other.0;
        self
    }
}

impl BitOr for TrapBehavior {
    type Output = Self;

    #[inline(always)]
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl BitOrAssign for TrapBehavior {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
