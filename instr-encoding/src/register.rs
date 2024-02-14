#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum XRegIdent {
    Zero = 0,
    Ra = 1,
    Sp = 2,
    Gp = 3,
    Tp = 4,
    T0 = 5,
    T1 = 6,
    T2 = 7,
    Fp = 8,
    S1 = 9,
    A0 = 10,
    A1 = 11,
    A2 = 12,
    A3 = 13,
    A4 = 14,
    A5 = 15,
    A6 = 16,
    A7 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    S8 = 24,
    S9 = 25,
    S10 = 26,
    S11 = 27,
    T3 = 28,
    T4 = 29,
    T5 = 30,
    T6 = 31,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CXRegIdent {
    Fp = 0,
    S1 = 1,
    A0 = 2,
    A1 = 3,
    A2 = 4,
    A3 = 5,
    A4 = 6,
    A5 = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum FRegIdent {
    Ft0 = 0,
    Ft1 = 1,
    Ft2 = 2,
    Ft3 = 3,
    Ft4 = 4,
    Ft5 = 5,
    Ft6 = 6,
    Ft7 = 7,
    Fs0 = 8,
    Fs1 = 9,
    Fa0 = 10,
    Fa1 = 11,
    Fa2 = 12,
    Fa3 = 13,
    Fa4 = 14,
    Fa5 = 15,
    Fa6 = 16,
    Fa7 = 17,
    Fs2 = 18,
    Fs3 = 19,
    Fs4 = 20,
    Fs5 = 21,
    Fs6 = 22,
    Fs7 = 23,
    Fs8 = 24,
    Fs9 = 25,
    Fs10 = 26,
    Fs11 = 27,
    Ft8 = 28,
    Ft9 = 29,
    Ft10 = 30,
    Ft11 = 31,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CFRegIdent {
    Fs0 = 0,
    Fs1 = 1,
    Fa0 = 2,
    Fa1 = 3,
    Fa2 = 4,
    Fa3 = 5,
    Fa4 = 6,
    Fa5 = 7,
}

impl XRegIdent {
    pub fn take(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Zero),
            1 => Some(Self::Ra),
            2 => Some(Self::Sp),
            3 => Some(Self::Gp),
            4 => Some(Self::Tp),
            5 => Some(Self::T0),
            6 => Some(Self::T1),
            7 => Some(Self::T2),
            8 => Some(Self::Fp),
            9 => Some(Self::S1),
            10 => Some(Self::A0),
            11 => Some(Self::A1),
            12 => Some(Self::A2),
            13 => Some(Self::A3),
            14 => Some(Self::A4),
            15 => Some(Self::A5),
            16 => Some(Self::A6),
            17 => Some(Self::A7),
            18 => Some(Self::S2),
            19 => Some(Self::S3),
            20 => Some(Self::S4),
            21 => Some(Self::S5),
            22 => Some(Self::S6),
            23 => Some(Self::S7),
            24 => Some(Self::S8),
            25 => Some(Self::S9),
            26 => Some(Self::S10),
            27 => Some(Self::S11),
            28 => Some(Self::T3),
            29 => Some(Self::T4),
            30 => Some(Self::T5),
            31 => Some(Self::T6),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn take_masked(v: u32) -> Self {
        match v & 31 {
            0 => Self::Zero,
            1 => Self::Ra,
            2 => Self::Sp,
            3 => Self::Gp,
            4 => Self::Tp,
            5 => Self::T0,
            6 => Self::T1,
            7 => Self::T2,
            8 => Self::Fp,
            9 => Self::S1,
            10 => Self::A0,
            11 => Self::A1,
            12 => Self::A2,
            13 => Self::A3,
            14 => Self::A4,
            15 => Self::A5,
            16 => Self::A6,
            17 => Self::A7,
            18 => Self::S2,
            19 => Self::S3,
            20 => Self::S4,
            21 => Self::S5,
            22 => Self::S6,
            23 => Self::S7,
            24 => Self::S8,
            25 => Self::S9,
            26 => Self::S10,
            27 => Self::S11,
            28 => Self::T3,
            29 => Self::T4,
            30 => Self::T5,
            31 => Self::T6,
            _ => unreachable!(),
        }
    }
}

impl FRegIdent {
    pub fn take(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Ft0),
            1 => Some(Self::Ft1),
            2 => Some(Self::Ft2),
            3 => Some(Self::Ft3),
            4 => Some(Self::Ft4),
            5 => Some(Self::Ft5),
            6 => Some(Self::Ft6),
            7 => Some(Self::Ft7),
            8 => Some(Self::Fs0),
            9 => Some(Self::Fs1),
            10 => Some(Self::Fa0),
            11 => Some(Self::Fa1),
            12 => Some(Self::Fa2),
            13 => Some(Self::Fa3),
            14 => Some(Self::Fa4),
            15 => Some(Self::Fa5),
            16 => Some(Self::Fa6),
            17 => Some(Self::Fa7),
            18 => Some(Self::Fs2),
            19 => Some(Self::Fs3),
            20 => Some(Self::Fs4),
            21 => Some(Self::Fs5),
            22 => Some(Self::Fs6),
            23 => Some(Self::Fs7),
            24 => Some(Self::Fs8),
            25 => Some(Self::Fs9),
            26 => Some(Self::Fs10),
            27 => Some(Self::Fs11),
            28 => Some(Self::Ft8),
            29 => Some(Self::Ft9),
            30 => Some(Self::Ft10),
            31 => Some(Self::Ft11),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn take_masked(v: u32) -> Self {
        match v & 31 {
            0 => Self::Ft0,
            1 => Self::Ft1,
            2 => Self::Ft2,
            3 => Self::Ft3,
            4 => Self::Ft4,
            5 => Self::Ft5,
            6 => Self::Ft6,
            7 => Self::Ft7,
            8 => Self::Fs0,
            9 => Self::Fs1,
            10 => Self::Fa0,
            11 => Self::Fa1,
            12 => Self::Fa2,
            13 => Self::Fa3,
            14 => Self::Fa4,
            15 => Self::Fa5,
            16 => Self::Fa6,
            17 => Self::Fa7,
            18 => Self::Fs2,
            19 => Self::Fs3,
            20 => Self::Fs4,
            21 => Self::Fs5,
            22 => Self::Fs6,
            23 => Self::Fs7,
            24 => Self::Fs8,
            25 => Self::Fs9,
            26 => Self::Fs10,
            27 => Self::Fs11,
            28 => Self::Ft8,
            29 => Self::Ft9,
            30 => Self::Ft10,
            31 => Self::Ft11,
            _ => unreachable!(),
        }
    }
}

impl CXRegIdent {
    pub fn take(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Fp),
            1 => Some(Self::S1),
            2 => Some(Self::A0),
            3 => Some(Self::A1),
            4 => Some(Self::A2),
            5 => Some(Self::A3),
            6 => Some(Self::A4),
            7 => Some(Self::A5),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn take_masked(v: u32) -> Self {
        match v & 7 {
            0 => Self::Fp,
            1 => Self::S1,
            2 => Self::A0,
            3 => Self::A1,
            4 => Self::A2,
            5 => Self::A3,
            6 => Self::A4,
            7 => Self::A5,
            _ => unreachable!(),
        }
    }
}

impl CFRegIdent {
    pub fn take(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Fs0),
            1 => Some(Self::Fs1),
            2 => Some(Self::Fa0),
            3 => Some(Self::Fa1),
            4 => Some(Self::Fa2),
            5 => Some(Self::Fa3),
            6 => Some(Self::Fa4),
            7 => Some(Self::Fa5),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn take_masked(v: u32) -> Self {
        match v & 7 {
            0 => Self::Fs0,
            1 => Self::Fs1,
            2 => Self::Fa0,
            3 => Self::Fa1,
            4 => Self::Fa2,
            5 => Self::Fa3,
            6 => Self::Fa4,
            7 => Self::Fa5,
            _ => unreachable!(),
        }
    }
}

impl From<CXRegIdent> for XRegIdent {
    #[inline(always)]
    fn from(value: CXRegIdent) -> Self {
        Self::take_masked(value as u32 + 8)
    }
}

impl From<CFRegIdent> for FRegIdent {
    #[inline(always)]
    fn from(value: CFRegIdent) -> Self {
        Self::take_masked(value as u32 + 8)
    }
}

impl ::core::fmt::Display for XRegIdent {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        #[rustfmt::skip]
        static LOOK_UP: [&str; 32] = [
            "zero", "ra", "sp",  "gp",  "tp", "t0", "t1", "t2",
            "fp",   "s1", "a0",  "a1",  "a2", "a3", "a4", "a5",
            "a6",   "a7", "s2",  "s3",  "s4", "s5", "s6", "s7",
            "s8",   "s9", "s10", "s11", "t3", "t4", "t5", "t6",
        ];
        f.write_str(LOOK_UP[*self as u8 as usize])
    }
}

impl ::core::fmt::Display for FRegIdent {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        #[rustfmt::skip]
        static LOOK_UP: [&str; 32] = [
            "ft0", "ft1", "ft2",  "ft3",  "ft4", "ft5", "ft6",  "ft7",
            "fs0", "fs1", "fa0",  "fa1",  "fa2", "fa3", "fa4",  "fa5",
            "fa6", "fa7", "fs2",  "fs3",  "fs4", "fs5", "fs6",  "fs7",
            "fs8", "fs9", "fs10", "fs11", "ft8", "ft9", "ft10", "ft11",
        ];
        f.write_str(LOOK_UP[*self as u8 as usize])
    }
}

impl ::core::fmt::Display for CXRegIdent {
    #[inline(always)]
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        XRegIdent::from(*self).fmt(f)
    }
}

impl ::core::fmt::Display for CFRegIdent {
    #[inline(always)]
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        FRegIdent::from(*self).fmt(f)
    }
}
