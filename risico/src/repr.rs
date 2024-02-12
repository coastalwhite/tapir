use serde::{Serialize, Deserialize};

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Offset(i32);

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Size(u32);

/// 4 Bytes that don't have a decimal value until it is explicitly declared.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Word(u32);

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Addr(u32);

macro_rules! is_zero {
    ($($t:ty),+ $(,)?) => {
        $(
        impl $t {
            pub fn is_zero(self) -> bool {
                self.0 == 0
            }
        }
        )+
    }
}
macro_rules! from_u32 {
    ($($t:ty$( => $inner_t:ty)?),+ $(,)?) => {
        $(
        impl From<u8> for $t {
            #[inline]
            fn from(value: u8) -> Self {
                Self(u32::from(value) $( as $inner_t)?)
            }
        }
        impl From<u16> for $t {
            #[inline]
            fn from(value: u16) -> Self {
                Self(u32::from(value) $( as $inner_t)?)
            }
        }
        impl From<u32> for $t {
            #[inline]
            fn from(value: u32) -> Self {
                Self(value $( as $inner_t)?)
            }
        }
        )+
    };
}
macro_rules! from_i32 {
    ($($t:ty$( => $inner_t:ty)?),+ $(,)?) => {
        $(
        impl From<i8> for $t {
            #[inline]
            fn from(value: i8) -> Self {
                Self(i32::from(value) $( as $inner_t)?)
            }
        }
        impl From<i16> for $t {
            #[inline]
            fn from(value: i16) -> Self {
                Self(i32::from(value) $( as $inner_t)?)
            }
        }
        impl From<i32> for $t {
            #[inline]
            fn from(value: i32) -> Self {
                Self(value $( as $inner_t)?)
            }
        }
        )+
    };
}
macro_rules! hex_display {
    ($($t:ty),+ $(,)?) => {
        $(
        impl std::fmt::LowerHex for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::LowerHex::fmt(&self.0, f)
            }
        }
        impl std::fmt::UpperHex for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::UpperHex::fmt(&self.0, f)
            }
        }
        )+
    };
}
macro_rules! dec_display {
    ($($t:ty),+ $(,)?) => {
        $(
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
        )+
    };
}
macro_rules! display_as_hex {
    ($($t:ty),+ $(,)?) => {
        $(
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "0x{:08x}", self.0)
            }
        }
        )+
    };
}
macro_rules! inner_add_sub_wrapping {
    ($($t:ty),+ $(,)?) => {
        $(
        impl std::ops::Add for $t {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0.wrapping_add(rhs.0))
            }
        }
        impl std::ops::AddAssign for $t {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }
        impl std::ops::Sub for $t {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0.wrapping_sub(rhs.0))
            }
        }
        impl std::ops::SubAssign for $t {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }
        )+
    };
}
macro_rules! inner_add_sub {
    ($($t:ty),+ $(,)?) => {
        $(
        impl std::ops::Add for $t {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }
        impl std::ops::AddAssign for $t {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        impl std::ops::Sub for $t {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }
        impl std::ops::SubAssign for $t {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
        )+
    };
}
macro_rules! inner_bitwise {
    ($($t:ty),+ $(,)?) => {
        $(
        impl std::ops::BitAnd for $t {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
        impl std::ops::BitAndAssign for $t {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }
        impl std::ops::BitOr for $t {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
        impl std::ops::BitOrAssign for $t {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }
        impl std::ops::BitXor for $t {
            type Output = Self;
            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }
        impl std::ops::BitXorAssign for $t {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }
        impl std::ops::Shl<Size> for $t {
            type Output = Self;
            #[inline]
            fn shl(self, rhs: Size) -> Self::Output {
                Self(self.0 << rhs.0)
            }
        }
        impl std::ops::Shl<usize> for $t {
            type Output = Self;
            #[inline]
            fn shl(self, rhs: usize) -> Self::Output {
                Self(self.0 << rhs)
            }
        }
        impl std::ops::ShlAssign<Size> for $t {
            #[inline]
            fn shl_assign(&mut self, rhs: Size) {
                self.0 <<= rhs.0;
            }
        }
        impl std::ops::ShlAssign<usize> for $t {
            #[inline]
            fn shl_assign(&mut self, rhs: usize) {
                self.0 <<= rhs;
            }
        }
        impl std::ops::Shr<Size> for $t {
            type Output = Self;
            #[inline]
            fn shr(self, rhs: Size) -> Self::Output {
                Self(self.0 >> rhs.0)
            }
        }
        impl std::ops::Shr<usize> for $t {
            type Output = Self;
            #[inline]
            fn shr(self, rhs: usize) -> Self::Output {
                Self(self.0 >> rhs)
            }
        }
        impl std::ops::ShrAssign<Size> for $t {
            #[inline]
            fn shr_assign(&mut self, rhs: Size) {
                self.0 >>= rhs.0;
            }
        }
        impl std::ops::ShrAssign<usize> for $t {
            #[inline]
            fn shr_assign(&mut self, rhs: usize) {
                self.0 >>= rhs;
            }
        }
        impl std::ops::Not for $t {
            type Output = Self;
            #[inline]
            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }
        )+
    };
}
macro_rules! eq_u32 {
    ($($t:ty),+ $(,)?) => {
        $(
        impl PartialEq<u32> for $t {
            #[inline]
            fn eq(&self, rhs: &u32) -> bool {
                self.0 == *rhs
            }
        }
        )+
    }
}
macro_rules! eq_i32 {
    ($($t:ty),+ $(,)?) => {
        $(
        impl PartialEq<i32> for $t {
            #[inline]
            fn eq(&self, rhs: &i32) -> bool {
                self.0 == *rhs
            }
        }
        )+
    }
}
macro_rules! bytes {
    ($($t:ty),+ $(,)?) => {
        $(
        impl $t {
            pub fn to_le_bytes(self) -> [u8; 4] {
                self.0.to_le_bytes()
            }

            pub fn to_be_bytes(self) -> [u8; 4] {
                self.0.to_be_bytes()
            }

            pub fn to_le_halfwords(self) -> [u16; 2] {
                [(self.0 & 0xFF) as u16, self.0.wrapping_shr(16) as u16]
            }

            pub fn to_be_halfwords(self) -> [u16; 2] {
                [self.0.wrapping_shr(16) as u16, (self.0 & 0xFF) as u16]
            }

            pub fn with_le_byte(self, index: usize, byte: u8) -> Self {
                debug_assert!(index < 4);
                let mut bytes = self.to_le_bytes();
                bytes[index] = byte;
                Self::from_le_bytes(bytes)
            }

            pub fn with_be_byte(self, index: usize, byte: u8) -> Self {
                debug_assert!(index < 4);
                let mut bytes = self.to_be_bytes();
                bytes[index] = byte;
                Self::from_be_bytes(bytes)
            }
        }
        )+
    }
}
macro_rules! alignment {
    ($($t:ty),+ $(,)?) => {
        $(
        impl $t {
            pub fn is_word_aligned(self) -> bool {
                self.0 & 0b11 == 0
            }
        }
        )+
    };
}
macro_rules! from_bytes_i32 {
    ($($t:ty),+ $(,)?) => {
        $(
        impl $t {
            pub fn from_le_bytes(bytes: [u8; 4]) -> Self {
                Self(i32::from_le_bytes(bytes))
            }
            pub fn from_be_bytes(bytes: [u8; 4]) -> Self {
                Self(i32::from_be_bytes(bytes))
            }
        }
        )+
    }
}
macro_rules! from_bytes_u32 {
    ($($t:ty),+ $(,)?) => {
        $(
        impl $t {
            pub fn from_le_bytes(bytes: [u8; 4]) -> Self {
                Self(u32::from_le_bytes(bytes))
            }
            pub fn from_be_bytes(bytes: [u8; 4]) -> Self {
                Self(u32::from_be_bytes(bytes))
            }
        }
        )+
    }
}

is_zero! { Offset, Size, Word, Addr }

from_u32! { Size, Word, Addr }
from_i32! { Offset, Word => u32 }
hex_display! { Offset, Size, Word, Addr }
dec_display! { Offset, Size }
display_as_hex! { Addr }

inner_add_sub_wrapping! { Word }
inner_add_sub! { Offset, Size }
inner_bitwise! { Word }
eq_u32! { Size, Addr }
eq_i32! { Offset }

bytes! { Offset, Size, Word, Addr }
alignment! { Offset, Size, Word, Addr }
from_bytes_i32! { Offset }
from_bytes_u32! { Size, Word, Addr }

impl Offset {
    #[inline]
    pub fn as_i32(self) -> i32 {
        self.0
    }
}

impl Size {
    #[inline]
    pub fn from_usize(s: usize) -> Option<Self> {
        #[cfg(not(any(target_pointer_width = "16", target_pointer_width = "32")))]
        if s > u32::MAX as usize {
            return None;
        }

        Some(Self(s as u32))
    }

    #[inline]
    pub fn as_usize(self) -> usize {
        #[cfg(target_pointer_width = "16")]
        compile_fail!("Cannot convert to usize");

        self.0 as usize
    }

    #[inline]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        if u32::MAX - self.0 < rhs.0 {
            return None;
        }

        Some(self + rhs)
    }
}

impl Word {
    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn as_i32(self) -> i32 {
        self.0 as i32
    }

    #[inline]
    pub fn as_addr(self) -> Addr {
        Addr(self.as_u32())
    }

    #[inline]
    pub fn as_offset(self) -> Offset {
        Offset(self.as_i32())
    }

    #[inline]
    pub fn as_size(self) -> Size {
        Size(self.as_u32())
    }

    #[inline]
    pub fn as_shift(self) -> Size {
        Size(self.as_u32() & 0x0000_001F)
    }

    #[inline]
    pub fn is_all_1(self) -> bool {
        self.0 == 0xFFFF_FFFF
    }

    #[inline]
    pub fn sign_extend_u8(byte: u8) -> Self {
        Self::from(byte).sign_extend::<8>()
    }

    #[inline]
    pub fn sign_extend_u16(halfword: u16) -> Self {
        Self::from(halfword).sign_extend::<16>()
    }

    #[inline]
    pub fn sign_extend<const N: usize>(self) -> Self {
        assert!(N > 0 && N <= 32);
        Self::from((self.as_i32() << (32 - N)) >> (32 - N))
    }
}

impl Addr {
    #[inline]
    pub fn offset(self, offset: impl Into<Offset>) -> Self {
        let offset = offset.into();
        let offset = offset.0 as u32;

        Self(self.0.wrapping_add(offset))
    }

    #[inline]
    pub fn end(self, size: impl Into<Size>) -> Self {
        let size = size.into();

        Self(self.0.wrapping_add(size.0))
    }

    #[inline]
    pub fn word_align(self) -> Self {
        Self(self.0 & 0xFFFF_FFFC)
    }

    #[inline]
    pub fn word_offset(self) -> u8 {
        (self.0 & 0x3) as u8
    }

    #[inline]
    pub fn in_blocks(self, block_size: usize) -> (usize, usize) {
        // TODO: Better check
        (self.0 as usize / block_size, self.0 as usize % block_size)
    }

    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn to(self, to: Addr) -> AddrIterator {
        AddrIterator {
            start: self,
            end: to,
        }
    }
}

impl From<Addr> for Word {
    fn from(value: Addr) -> Word {
        Word(value.0)
    }
}

impl From<Addr> for u32 {
    fn from(value: Addr) -> Self {
        value.0
    }
}

pub struct AddrIterator {
    start: Addr,
    end: Addr,
}

impl Iterator for AddrIterator {
    type Item = Addr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        let addr = self.start;
        self.start.0 = self.start.0.wrapping_add(1);
        Some(addr)
    }
}
