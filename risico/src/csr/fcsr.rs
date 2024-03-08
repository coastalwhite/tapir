use super::{CsrInitContext, CsrWriteContext};

#[derive(Clone, Debug)]
pub struct Fcsr {
    inner: u32,
}

#[derive(Clone, Copy)]
pub struct ExceptionFlags(u8);

impl std::fmt::Debug for ExceptionFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExceptionFlags")
            .field("inexact", &self.contains(Self::INEXACT))
            .field("underflow", &self.contains(Self::UNDERFLOW))
            .field("overflow", &self.contains(Self::OVERFLOW))
            .field("divide_by_zero", &self.contains(Self::DIVIDE_BY_ZERO))
            .field("invalid", &self.contains(Self::INVALID))
            .field("raw_bits", &self.0)
            .finish()
    }
}

impl ExceptionFlags {
    pub const INVALID: Self = Self(0b10000);
    pub const DIVIDE_BY_ZERO: Self = Self(0b01000);
    pub const OVERFLOW: Self = Self(0b00100);
    pub const UNDERFLOW: Self = Self(0b00010);
    pub const INEXACT: Self = Self(0b00001);

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, x: Self) -> bool {
        self.0 & x.0 == x.0
    }

    pub fn to_bits(self) -> u8 {
        self.0
    }

    pub fn from_bits(flags: u8) -> Self {
        debug_assert_eq!(flags & 0xE0, 0);
        Self(flags)
    }
}

impl Into<::softfloat_wrapper::ExceptionFlags> for ExceptionFlags {
    fn into(self) -> ::softfloat_wrapper::ExceptionFlags {
        // @Hack. This is far from guaranteed to work. This should be changed.
        ::softfloat_wrapper::ExceptionFlags::from_bits(self.to_bits())
    }
}

impl From<rsoftfloat::Flags> for ExceptionFlags {
    fn from(value: rsoftfloat::Flags) -> Self {
        let mut flags = Self::empty();

        // @Hack. What is this. This should just be a bit cast.
        if value.is_invalid() {
            flags |= Self::INVALID;
        }
        if value.is_divide_by_zero() {
            flags |= Self::DIVIDE_BY_ZERO;
        }
        if value.is_overflow() {
            flags |= Self::OVERFLOW;
        }
        if value.is_underflow() {
            flags |= Self::UNDERFLOW;
        }
        if value.is_inexact() {
            flags |= Self::INEXACT;
        }

        flags
    }
}

impl std::ops::BitOr for ExceptionFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ExceptionFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Fcsr {
    const FFLAGS_MASK: u32 = 0x1F;
    const FRM_MASK: u32 = 0xE0;

    pub fn get_fflags(&self) -> ExceptionFlags {
        ExceptionFlags::from_bits(self.fflags_read() as u8)
    }

    pub fn set_fflags(&mut self, flags: ExceptionFlags) {
        self.inner =
            (self.inner & !Self::FFLAGS_MASK) | ((u32::from(flags.to_bits()) << 0) & Self::FFLAGS_MASK);
    }

    pub fn add_fflags(&mut self, flags: impl Into<ExceptionFlags>) {
        self.set_fflags(self.get_fflags() | flags.into())
    }

    pub fn set_flag(&mut self, flag: ExceptionFlags) {
        self.set_fflags(self.get_fflags() | flag)
    }

    pub fn fflags_read(&self) -> u32 {
        self.inner & Self::FFLAGS_MASK
    }
    pub fn fflags_write(&mut self, value: u32, _: &CsrWriteContext) {
        self.inner = (self.inner & !Self::FFLAGS_MASK) | ((value << 0) & Self::FFLAGS_MASK);
    }

    pub fn frm_read(&self) -> u32 {
        (self.inner & Self::FRM_MASK) >> 5
    }
    pub fn frm_write(&mut self, value: u32, _: &CsrWriteContext) {
        self.inner = (self.inner & !Self::FRM_MASK) | ((value << 5) & Self::FRM_MASK);
    }

    #[inline]
    pub fn new(ctx: &CsrInitContext) -> Self {
        let frm = 0b000; // TiesToEven
        let fflags = 0b00000;

        Self {
            inner: (frm << 5) | fflags,
        }
    }

    #[inline]
    pub fn read(&self) -> u32 {
        self.inner
    }

    #[inline]
    pub fn write(&mut self, value: u32, _: &CsrWriteContext) {
        // bitlen = |frm| + |fflags|
        //        = 3     + 5
        //        = 8
        let value = value & 0xFF;
        self.inner = value;
    }
}
