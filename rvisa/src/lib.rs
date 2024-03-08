#![no_std]

use core::fmt::Display;
use core::ops::{BitOr, BitOrAssign};
use core::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MIsa(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MIsaExt(u32);

impl core::fmt::Debug for MIsa {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MIsa")
            .field("mxlen", &self.mxlen())
            .field("extensions", &self.extensions())
            .finish()
    }
}

impl core::fmt::Debug for MIsaExt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MIsaExt")
            .field("atomic", &self.contains(Self::ATOMIC))
            .field("compressed", &self.contains(Self::COMPRESSED))
            .field(
                "double_precision_fp",
                &self.contains(Self::DOUBLE_PRECISION_FP),
            )
            .field("rv32e", &self.contains(Self::RV32E))
            .field(
                "single_precision_fp",
                &self.contains(Self::SINGLE_PRECISION_FP),
            )
            .field("hypervisor", &self.contains(Self::HYPERVISOR))
            .field("rv_i", &self.contains(Self::RV_I))
            .field("integer_muldiv", &self.contains(Self::INTEGER_MULDIV))
            .field("quad_precision_fp", &self.contains(Self::QUAD_PRECISION_FP))
            .field("supervisor_mode", &self.contains(Self::SUPERVISOR_MODE))
            .field("user_mode", &self.contains(Self::USER_MODE))
            .field("vector", &self.contains(Self::VECTOR))
            .field("non_standard", &self.contains(Self::NON_STANDARD))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MXLen {
    Mixed = 0b00,
    X32 = 0b01,
    X64 = 0b10,
    X128 = 0b11,
}

impl MXLen {
    #[inline]
    pub const fn take_masked(x: u32) -> Self {
        match x & 0b11 {
            0b00 => MXLen::Mixed,
            0b01 => MXLen::X32,
            0b10 => MXLen::X64,
            0b11 => MXLen::X128,
            _ => unreachable!(),
        }
    }
}

impl MIsa {
    #[inline]
    pub const fn new(mxlen: MXLen, exts: MIsaExt) -> Self {
        Self((mxlen as u32) << 30 | exts.0)
    }

    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn from_u32(x: u32) -> Self {
        let mxlen = MXLen::take_masked(x >> 30);
        let exts = MIsaExt::from_u32(x).intersect(MIsaExt::MAX_ENABLED);

        Self::new(mxlen, exts)
    }

    #[inline]
    pub const fn mxlen(self) -> MXLen {
        MXLen::take_masked(self.0 >> 30)
    }

    #[inline]
    pub const fn extensions(self) -> MIsaExt {
        MIsaExt(self.0 & 0x3FF_FFFF)
    }

    #[inline]
    pub const fn without_ext(self, ext: MIsaExt) -> Self {
        Self(self.0 & !ext.0)
    }

    #[inline]
    pub const fn with_ext(self, ext: MIsaExt) -> Self {
        Self(self.0 | ext.0)
    }

    #[inline]
    pub const fn contains(self, other: MIsaExt) -> bool {
        self.extensions().contains(other)
    }
}

impl MIsaExt {
    pub const EMPTY: Self = Self(0x000_0000);

    pub const ATOMIC: Self = Self(0x000_0001);
    // pub const BIT_MANIPULATION: Self = Self(0x000_0002);
    pub const COMPRESSED: Self = Self(0x000_0004);
    pub const DOUBLE_PRECISION_FP: Self = Self(0x000_0008);
    pub const RV32E: Self = Self(0x000_0010);
    pub const SINGLE_PRECISION_FP: Self = Self(0x000_0020);
    // pub const G: Self = Self(0x000_0040);
    pub const HYPERVISOR: Self = Self(0x000_0080);
    pub const RV_I: Self = Self(0x000_0100);
    // pub const J: Self = Self(0x000_0200);
    // pub const K: Self = Self(0x000_0400);
    // pub const L: Self = Self(0x000_0800);
    pub const INTEGER_MULDIV: Self = Self(0x000_1000);
    // pub const N: Self = Self(0x000_2000);
    // pub const O: Self = Self(0x000_4000);
    // pub const PACKED_SIMD: Self = Self(0x000_8000);
    pub const QUAD_PRECISION_FP: Self = Self(0x001_0000);
    // pub const R: Self = Self(0x002_0000);
    pub const SUPERVISOR_MODE: Self = Self(0x004_0000);
    // pub const T: Self = Self(0x008_0000);
    pub const USER_MODE: Self = Self(0x010_0000);
    pub const VECTOR: Self = Self(0x020_0000);
    // pub const W: Self = Self(0x040_0000);
    pub const NON_STANDARD: Self = Self(0x080_0000);
    // pub const Y: Self = Self(0x100_0000);
    // pub const Z: Self = Self(0x200_0000);

    const MAX_ENABLED: Self = Self(0x0B5_11BD);

    #[inline]
    pub const fn from_u64(misa: u64) -> Self {
        Self(misa as u32).intersect(Self::MAX_ENABLED)
    }

    #[inline]
    pub const fn from_u32(misa: u32) -> Self {
        Self(misa as u32).intersect(Self::MAX_ENABLED)
    }

    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[inline]
    pub const fn intersect(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for MIsaExt {
    type Output = Self;

    /// Perform a union between the extensions in both [`MIsa`]s.
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for MIsaExt {
    /// Perform a union between the extensions in both [`MIsa`]s.
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MIsaParseError {
    InvalidExtension(char),
    DuplicateExtension(char),
    InvalidStart,
    InvalidBase,
    InvalidMxlen,
}

impl Display for MIsaParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidExtension(c) => write!(f, "RISC-V ISA specifier has an invalid extension with '{c}'"),
            Self::DuplicateExtension(c) => write!(f, "RISC-V ISA specifier has a duplicate extension with '{c}'"),
            Self::InvalidStart => f.write_str("RISC-V ISA specifier does not start with 'rv'"),
            Self::InvalidMxlen => f.write_str("RISC-V ISA specifier does not have valid mxlen. Valid mxlen's are '32', '64' and '128'."),
            Self::InvalidBase => f.write_str("RISC-V ISA specifier does not have valid base. Valid bases are 'i' and 'g'. 'e' is also valid if mxlen=32."),
        }
    }
}

impl FromStr for MIsa {
    type Err = MIsaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !(s.starts_with("rv") || s.starts_with("RV")) {
            return Err(MIsaParseError::InvalidStart);
        }

        let s = &s[2..];

        let (mxlen, slen) = match () {
            _ if s.starts_with("32") => (MXLen::X32, 2),
            _ if s.starts_with("64") => (MXLen::X64, 2),
            _ if s.starts_with("128") => (MXLen::X128, 3),
            _ => return Err(MIsaParseError::InvalidMxlen),
        };

        let s = &s[slen..];

        let mut chars = s.chars();

        let Some(base) = chars.next() else {
            return Err(MIsaParseError::InvalidBase);
        };

        let mut exts = match base {
            'i' => MIsaExt::RV_I,
            // @TODO: This should include Zicsr & Zifencei
            'g' => {
                MIsaExt::RV_I
                    | MIsaExt::INTEGER_MULDIV
                    | MIsaExt::ATOMIC
                    | MIsaExt::SINGLE_PRECISION_FP
                    | MIsaExt::DOUBLE_PRECISION_FP
            }
            'e' if matches!(mxlen, MXLen::X32) => MIsaExt::RV32E,
            _ => return Err(MIsaParseError::InvalidBase),
        };

        for c in chars {
            let ext = match c.to_ascii_lowercase() {
                'a' => MIsaExt::ATOMIC,
                'c' => MIsaExt::COMPRESSED,
                'd' => MIsaExt::DOUBLE_PRECISION_FP,
                'f' => MIsaExt::SINGLE_PRECISION_FP,
                'h' => MIsaExt::HYPERVISOR,
                'm' => MIsaExt::INTEGER_MULDIV,
                'q' => MIsaExt::QUAD_PRECISION_FP,
                's' => MIsaExt::SUPERVISOR_MODE,
                'u' => MIsaExt::USER_MODE,
                'v' => MIsaExt::VECTOR,
                'x' => MIsaExt::NON_STANDARD,
                _ => return Err(MIsaParseError::InvalidExtension(c)),
            };

            if exts.intersect(ext) != MIsaExt::EMPTY {
                return Err(MIsaParseError::DuplicateExtension(c));
            }

            exts |= ext;
        }

        Ok(MIsa::new(mxlen, exts))
    }
}
