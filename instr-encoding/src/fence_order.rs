use std::fmt::{Debug, Display, Write};
use std::ops::{BitOr, BitOrAssign};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FenceOrder(u8);

impl Debug for FenceOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FenceOrder")
            .field("device_input", &self.has_device_input())
            .field("device_output", &self.has_device_output())
            .field("memory_write", &self.has_memory_write())
            .field("memory_read", &self.has_memory_read())
            .finish()
    }
}

impl Display for FenceOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_device_input() {
            f.write_char('i')?;
        }
        if self.has_device_output() {
            f.write_char('o')?;
        }
        if self.has_memory_read() {
            f.write_char('r')?;
        }
        if self.has_memory_write() {
            f.write_char('w')?;
        }

        Ok(())
    }
}

impl FenceOrder {
    pub const DEVICE_INPUT: Self = Self(0b1000);
    pub const DEVICE_OUTPUT: Self = Self(0b0100);
    pub const MEMORY_WRITE: Self = Self(0b0010);
    pub const MEMORY_READ: Self = Self(0b0001);

    #[inline(always)]
    pub fn take_masked(bits: u32) -> Self {
        Self((bits & 0b1111) as u8)
    }

    #[inline(always)]
    pub fn encode(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[inline(always)]
    pub fn has_device_input(self) -> bool {
        self.0 & Self::DEVICE_INPUT.0 != 0
    }

    #[inline(always)]
    pub fn has_device_output(self) -> bool {
        self.0 & Self::DEVICE_OUTPUT.0 != 0
    }

    #[inline(always)]
    pub fn has_memory_write(self) -> bool {
        self.0 & Self::MEMORY_WRITE.0 != 0
    }

    #[inline(always)]
    pub fn has_memory_read(self) -> bool {
        self.0 & Self::MEMORY_READ.0 != 0
    }

    #[inline(always)]
    pub fn intersect(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for FenceOrder {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for FenceOrder {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
