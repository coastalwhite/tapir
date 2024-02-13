use std::ops::{BitOr, BitOrAssign};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Flags(u8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    /// Round to Nearest, Ties to Even,
    #[default]
    TiesToEven,

    /// Round to Zero
    ToZero,
    /// Round Down (towards -Infinity)
    Down,
    /// Round Up (towards Infinity)
    Up,
    /// Round to Nearest, Ties to Max Magnitude
    TiesToMaxMagnitude,
}

impl Flags {
    pub const INEXACT: Self = Self(0b00001);
    pub const UNDERFLOW: Self = Self(0b00010);
    pub const OVERFLOW: Self = Self(0b00100);
    pub const DIVIDE_BY_ZERO: Self = Self(0b01000);
    pub const INVALID: Self = Self(0b10000);

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn contains(self, x: Flags) -> bool {
        self.0 & x.0 == x.0
    }

    #[inline]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn as_u16(self) -> u16 {
        self.0 as u16
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }
}

impl BitOr for Flags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Flags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub mod f32 {
    use std::cmp::Ordering;

    use crate::Flags;

    use super::RoundingMode;

    const SIGN_MASK: u32 = 0x8000_0000;
    const EXPONENT_MASK: u32 = 0x7F80_0000;
    const SIGNIFICANT_MASK: u32 = 0x007F_FFFF;

    type F32Inner = u32;

    #[derive(Clone, Copy)]
    pub struct F32(F32Inner);

    impl std::fmt::Debug for F32 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.to_f32().fmt(f)
        }
    }

    impl std::fmt::Display for F32 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.to_f32().fmt(f)
        }
    }

    // fn normSubnormalF32Sig( significant: u32 ) -> F32 {
    //     let shift_distance = significant.leading_zeros() - 8;
    //     let exponent = 1u8.wrapping_sub(shift_distance as u8);
    //     let significant = significant << shift_distance;
    //     F32::pack(false, exponent, significant << shift_distance)
    // }

    #[inline(always)]
    const fn sign(x: F32Inner) -> bool {
        x >> 31 != 0
    }

    #[inline(always)]
    const fn exponent(x: F32Inner) -> u8 {
        ((x >> 23) & 0xFF) as u8
    }

    #[inline(always)]
    const fn significant(x: F32Inner) -> u32 {
        x & SIGNIFICANT_MASK
    }

    #[inline(always)]
    const fn fraction(x: F32Inner) -> u32 {
        significant(x)
            | if x & EXPONENT_MASK != 0 {
                0x0080_0000
            } else {
                0x0000_0000
            }
    }

    impl F32 {
        pub const CANNONICAL_NAN: Self = Self(0x7FC0_0000);
        pub const BIAS: u8 = 127;

        pub const SIGNALING_NAN: Self = Self(0x7F80_0001);
        pub const QUIET_NAN: Self = Self(0x7FC0_0000);

        const IMPLIED_MOST_SIGNIFICANT: u32 = 0x0080_0000;

        #[inline(always)]
        pub const fn from_bits(bits: u32) -> Self {
            Self(bits)
        }

        #[inline(always)]
        pub const fn to_bits(self) -> u32 {
            self.0
        }

        #[inline(always)]
        pub const fn from_f32(x: f32) -> Self {
            unsafe { core::mem::transmute(x) }
        }

        #[inline(always)]
        pub const fn to_f32(self) -> f32 {
            unsafe { core::mem::transmute(self) }
        }

        #[inline(always)]
        pub const fn sign(self) -> bool {
            sign(self.0)
        }

        #[inline(always)]
        pub const fn exponent(self) -> u8 {
            exponent(self.0)
        }

        #[inline(always)]
        pub const fn significant(self) -> u32 {
            significant(self.0)
        }

        #[inline(always)]
        pub const fn fraction(self) -> u32 {
            fraction(self.0)
        }

        #[inline(always)]
        pub const fn is_subnormal(self) -> bool {
            (self.exponent() == 0x00) & (self.significant() != 0)
        }

        #[inline(always)]
        pub const fn is_zero(self) -> bool {
            self.to_bits() & 0x7FFF_FFFF == 0
        }

        #[inline(always)]
        pub const fn is_normal(self) -> bool {
            let exponent = self.exponent();
            (exponent != 0x00) & (exponent != 0xFF)
        }

        #[inline(always)]
        pub const fn is_nan(self) -> bool {
            (self.exponent() == 0xFF) & (self.significant() != 0)
        }

        #[inline(always)]
        pub const fn is_infinite(self) -> bool {
            self.0 & 0x7FFF_FFFF == self.0 & 0x7F80_0000
        }

        #[inline(always)]
        pub const fn is_signaling_nan(self) -> bool {
            self.0 & 0x7FC0_0000 == 0x7F80_0000 && (self.significant() != 0)
        }

        #[inline(always)]
        pub const fn is_quiet_nan(self) -> bool {
            self.0 & 0x7FC0_0000 == 0x7FC0_0000
        }

        #[inline(always)]
        const fn new_infinity(sign: bool) -> Self {
            Self(((sign as u32) << 31) | 0x7F80_0000)
        }

        #[inline(always)]
        const fn new_zero(sign: bool) -> Self {
            Self((sign as u32) << 31)
        }

        #[inline(always)]
        const fn is_nan_or_infinity(self) -> bool {
            self.exponent() == 0xFF
        }

        #[inline(always)]
        pub(crate) const fn normalized(self) -> (i16, u32) {
            assert!(self.is_subnormal() | self.is_normal() | self.is_zero());

            if self.is_subnormal() {
                let significant = self.significant();
                let extra_exp = significant.leading_zeros() - 8;
                let mantissa = significant << extra_exp;
                (-126 - extra_exp as i16, mantissa)
            } else {
                let exp = self.exponent() as i16 - Self::BIAS as i16;
                let significant = self.significant();
                (exp, significant | Self::IMPLIED_MOST_SIGNIFICANT)
            }
        }

        #[inline(always)]
        const fn pack(sign: bool, exponent: u8, significant: u32) -> Self {
            Self(((sign as u32) << 31) | ((exponent as u32) << 23) | significant)
        }

        #[inline(always)]
        const fn unpack(self) -> (bool, i16, u32) {
            let sign = self.sign();
            let exponent = (self.exponent() as i16) - (Self::BIAS as i16);
            let significant = self.significant();

            (sign, exponent, significant)
        }

        #[inline(always)]
        const fn add_one_to_mantissa(mantissa: u32) -> (i16, u32) {
            let has_overflow = mantissa & 0x007F_FFFF == 0x007F_FFFF;
            if has_overflow {
                (1, 0x8000_0000)
            } else {
                (0, mantissa + 1)
            }
        }

        fn shift_rounding(
            sign: bool,
            mantissa: u64,
            shift_distance: u32,
            rm: RoundingMode,
        ) -> (i16, u32) {
            debug_assert_ne!(mantissa, 0);
            debug_assert!(shift_distance < 64);

            let truncate_mask = (1u64 << shift_distance).wrapping_sub(1);

            let truncated = mantissa & truncate_mask;

            let mut exp_difference = 0;
            let mut mantissa = (mantissa >> shift_distance) as u32;

            if truncated == 0 {
                return (exp_difference, mantissa);
            }

            match rm {
                RoundingMode::TiesToEven => {
                    let tie_point = 1u64 << (shift_distance - 1);
                    if truncated > tie_point || (truncated == tie_point && mantissa & 1 != 0) {
                        (exp_difference, mantissa) = Self::add_one_to_mantissa(mantissa);
                    }
                }
                RoundingMode::Up => {
                    if !sign {
                        (exp_difference, mantissa) = Self::add_one_to_mantissa(mantissa);
                    }
                }
                RoundingMode::Down => {
                    if sign {
                        (exp_difference, mantissa) = Self::add_one_to_mantissa(mantissa);
                    }
                }
                RoundingMode::ToZero => {}
                RoundingMode::TiesToMaxMagnitude => {
                    let tie_point = 1u64 << (shift_distance - 1);
                    if truncated >= tie_point {
                        (exp_difference, mantissa) = Self::add_one_to_mantissa(mantissa);
                    }
                }
            }

            (exp_difference, mantissa)
        }

        fn rounding(sign: bool, mantissa: u64, rm: RoundingMode) -> (i16, u32) {
            debug_assert_ne!(mantissa, 0);

            let leading_zeros = mantissa.leading_zeros();

            debug_assert!(leading_zeros >= 16);
            debug_assert!(leading_zeros <= 32 + 8);

            let shift_distance = (32 + 8) - leading_zeros;

            Self::shift_rounding(sign, mantissa, shift_distance, rm)
        }

        pub fn eq(self, other: Self) -> (bool, Flags) {
            let a = self;
            let b = other;

            let mut flags = Flags::empty();

            if a.is_nan() | a.is_nan() {
                if a.is_signaling_nan() | b.is_signaling_nan() {
                    flags |= Flags::INVALID;
                }

                return (false, flags);
            }

            (a.to_bits() == b.to_bits(), flags)
        }

        pub fn max(self, other: Self) -> (Self, Flags) {
            let a = self;
            let b = other;

            let mut flags = Flags::empty();
            
            if !a.is_nan_or_infinity() & !b.is_nan_or_infinity() {
                let (sign_a, exp_a, significant_a) = a.unpack();
                let (sign_b, exp_b, significant_b) = b.unpack();

                let sign_z = sign_a | sign_b;

                use Ordering as O;
                let result = match (sign_a.cmp(&sign_b), exp_a.cmp(&exp_b), significant_a.cmp(&significant_b)) {
                    (O::Greater, _, _) => b,
                    (O::Less, _, _) => a,
                    (O::Equal, O::Less, _) if sign_z => a,
                    (O::Equal, O::Less, _) => b,
                    (O::Equal, O::Greater, _) if sign_z => b,
                    (O::Equal, O::Greater, _) => a,
                    (O::Equal, O::Equal, O::Less) if sign_z => a,
                    (O::Equal, O::Equal, O::Less) => b,
                    (O::Equal, O::Equal, O::Greater) if sign_z => b,
                    (O::Equal, O::Equal, O::Greater) => a,
                    (O::Equal, O::Equal, O::Equal) => a,
                };

                return (result, flags);
            }

            if a.is_infinite() & b.is_infinite() {
                let value = if a.sign() < b.sign() { a } else { b };
                return (value, flags);
            }

            if a.is_signaling_nan() | b.is_signaling_nan() {
                flags |= Flags::INVALID;
            }

            if a.is_nan() & b.is_nan() {
                return (Self::CANNONICAL_NAN, flags);
            }

            if a.is_nan() {
                return (b, flags);
            }
            if b.is_nan() {
                return (a, flags);
            }

            (a, Flags::empty())
        }

        pub fn min(self, other: Self) -> (Self, Flags) {
            let a = self;
            let b = other;

            let mut flags = Flags::empty();

            if !a.is_nan_or_infinity() & !b.is_nan_or_infinity() {
                let (sign_a, exp_a, significant_a) = a.unpack();
                let (sign_b, exp_b, significant_b) = b.unpack();

                let sign_z = sign_a | sign_b;

                use Ordering as O;
                let result = match (sign_a.cmp(&sign_b), exp_a.cmp(&exp_b), significant_a.cmp(&significant_b)) {
                    (O::Greater, _, _) => a,
                    (O::Less, _, _) => b,
                    (O::Equal, O::Less, _) if sign_z => b,
                    (O::Equal, O::Less, _) => a,
                    (O::Equal, O::Greater, _) if sign_z => a,
                    (O::Equal, O::Greater, _) => b,
                    (O::Equal, O::Equal, O::Less) if sign_z => b,
                    (O::Equal, O::Equal, O::Less) => a,
                    (O::Equal, O::Equal, O::Greater) if sign_z => a,
                    (O::Equal, O::Equal, O::Greater) => b,
                    (O::Equal, O::Equal, O::Equal) => a,
                };

                return (result, flags);
            }

            if a.is_infinite() & b.is_infinite() {
                let value = if a.sign() > b.sign() { a } else { b };
                return (value, flags);
            }

            if a.is_signaling_nan() | b.is_signaling_nan() {
                flags |= Flags::INVALID;
            }

            if a.is_nan() & b.is_nan() {
                return (Self::CANNONICAL_NAN, flags);
            }

            if a.is_nan() {
                return (b, flags);
            }
            if b.is_nan() {
                return (a, flags);
            }

            (a, Flags::empty())
        }

        pub fn mul(a: Self, b: Self, rm: RoundingMode) -> Self {
            let sign_z = a.sign() ^ b.sign();

            if a.is_nan_or_infinity() | b.is_nan_or_infinity() {
                if a.is_signaling_nan() | b.is_signaling_nan() {
                    // @TODO: NaN Propogation
                    // @TODO: Invalid flag
                    return Self::SIGNALING_NAN;
                }

                if a.is_nan() | b.is_nan() {
                    // @TODO: NaN Propogation
                    return Self::QUIET_NAN;
                }

                if a.is_zero() | b.is_zero() {
                    return Self::QUIET_NAN;
                }

                return Self::new_infinity(sign_z);
            }

            if a.is_zero() | b.is_zero() {
                return Self::new_zero(sign_z);
            }

            if a.is_subnormal() & b.is_subnormal() {
                return Self::new_zero(sign_z);
            }

            let (_, exp_a, significant_a) = a.unpack();
            let (_, exp_b, significant_b) = b.unpack();

            let (exp_a, mant_a) = a.normalized();
            let (exp_b, mant_b) = b.normalized();

            dbg!(exp_a, exp_b);

            let exp_z = exp_a + exp_b;
            let mant_z = mant_a as u64 * mant_b as u64;

            // NOTE: The we know that the mantissa of an f32 is 24 bits (with the implied bit).
            // Then, the product might only need renormalization by one.
            const NEEDS_RENORMALIZATION_BIT_MASK: u64 = 0x0000_8000_0000_0000;

            let needs_renormalization = mant_z & NEEDS_RENORMALIZATION_BIT_MASK != 0;

            let exp_z = exp_z + (needs_renormalization as i16);

            if exp_z < -126 {
                if exp_z < -126 - 23 {
                    // @TODO: Underflow
                    return Self::new_zero(sign_z);
                }

                let subnormal_shift = -126 - exp_z;
                let (exp_z, frac_z) = Self::shift_rounding(
                    sign_z,
                    mant_z,
                    23 + (needs_renormalization as u32) + subnormal_shift as u32,
                    rm,
                );

                let frac_z = frac_z & 0x007F_FFFF;

                debug_assert_eq!(frac_z & 0xFF80_0000, 0);
                debug_assert!(exp_z == 0 || exp_z == 1);

                return Self::pack(sign_z, exp_z as u8, frac_z);
            }

            let (exp_diff_z, frac_z) =
                Self::shift_rounding(sign_z, mant_z, 23 + (needs_renormalization as u32), rm);
            let frac_z = frac_z & 0x007F_FFFF;

            let exp_z = exp_z + (Self::BIAS as i16);
            let exp_z = exp_z + exp_diff_z;

            if exp_z > 127 {
                // @TODO: Overflow
                return Self::new_infinity(sign_z);
            }

            Self::pack(sign_z, exp_z as u8, frac_z)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::f32::*;
    use super::*;

    fn softfp_get_rounding_mode() -> ::softfp::RoundingMode {
        ::softfp::RoundingMode::TiesToEven
    }

    fn softfp_set_exception_flags(exp: ::softfp::ExceptionFlags) {}

    #[test]
    fn read_normalized() {
        let (exp, mantissa) = F32::from_bits(0x0000_0001).normalized();
    }

    #[test]
    fn it_works() {
        ::softfp::register_get_rounding_mode(softfp_get_rounding_mode);
        ::softfp::register_set_exception_flags(softfp_set_exception_flags);

        let b = 0.5f32.to_bits();
        for a in 0..u32::MAX {
            // let a = 0x00FF_FFFF;

            let softfp_a = ::softfp::F32::new(a);
            let softfp_b = ::softfp::F32::new(b);

            let my_a = F32::from_bits(a);
            let my_b = F32::from_bits(b);

            let softfp_result = softfp_a * softfp_b;
            let my_result = F32::mul(my_a, my_b, RoundingMode::TiesToEven);

            let softfp_result = softfp_result.0;
            let my_result = my_result.to_bits();

            if softfp_result != my_result {
                eprintln!("Mismatch:");
                eprintln!("a      = 0x{:08x} : {}", a, f32::from_bits(a));
                eprintln!("b      = 0x{:08x} : {}", b, f32::from_bits(b));
                eprintln!(
                    "SoftFP = 0x{:08x} : {}",
                    softfp_result,
                    f32::from_bits(softfp_result)
                );
                eprintln!(
                    "MyFp   = 0x{:08x} : {}",
                    my_result,
                    f32::from_bits(my_result)
                );

                panic!();
            }
        }
        //
        // let a = F32::from_bits(0.1f32.to_bits());
        // let b = F32::from_bits(0.2f32.to_bits());
        // let c = F32::from_bits(0x0040_0000);
        //
        // let result_1 = F32::mul(a, b, RoundingMode::TiesToEven);
        // let result_2 = F32::mul(a, c, RoundingMode::Up);
        //
        // eprintln!("0.1 * 0.2 = {}", f32::from_bits(result_1.to_bits()));
        // eprintln!("0.1 * 0.2 = {}", 0.1f32 * 0.2f32);
        // eprintln!("0.1 * Sn = {} (0x{:08x})", f32::from_bits(result_2.to_bits()), result_2.to_bits());
    }
}
