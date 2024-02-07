use std::fmt::{Debug, Display, Write};

use serde::Deserialize;

// @Hack, this definitely does not need to allocate
#[derive(Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct Bits {
    inner: BitsInner,
}

impl Debug for Bits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bits<{}>(", self.num_bits())?;

        match self.inner {
            BitsInner::Short(n, bits) => {
                if n % 4 == 0 && n > 8 {
                    write!(f, "{bits:00$X}", (n / 4) as usize)?;
                } else {
                    write!(f, "{bits:00$b}", n as usize)?;
                }
            }
            BitsInner::Long(_) => todo!(),
        }

        f.write_char(')')?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum BitsInner {
    Short(u32, u64),
    Long(Box<LongBits>),
}

impl Bits {
    pub fn zeros(num_bits: u32) -> Self {
        Self {
            inner: BitsInner::zeros(num_bits),
        }
    }

    pub fn count_ones(&self) -> u32 {
        self.inner.count_ones()
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self.inner {
            BitsInner::Short(num_bits, bits) if num_bits <= 32 => Some((bits & 0xFFFF_FFFF) as u32),
            BitsInner::Short(_, _) => None,
            BitsInner::Long(_) => None,
        }
    }

    pub fn le_byte(&self, idx: usize) -> u8 {
        match self.inner {
            BitsInner::Short(_, bits) => bits.to_le_bytes()[idx],
            BitsInner::Long(_) => todo!(),
        }
    }

    pub fn num_bits(&self) -> u32 {
        self.inner.num_bits()
    }

    pub fn bitor_ones(&mut self, offset: u32, num_bits: u32) {
        self.inner.bitor_ones(offset, num_bits)
    }

    pub fn bitor_at(&mut self, offset: u32, rhs: &Self) {
        self.inner.bitor_at(offset, &rhs.inner)
    }
}

impl BitsInner {
    fn zeros(num_bits: u32) -> Self {
        if num_bits < 64 {
            return Self::Short(num_bits, 0);
        }

        // FIX. Remove `as` casting
        let num_cells = num_bits.div_ceil(64) as usize;
        let content = vec![0u64; num_cells];

        BitsInner::Long(Box::new(LongBits { content, num_bits }))
    }

    fn count_ones(&self) -> u32 {
        match self {
            BitsInner::Short(_, bits) => bits.count_ones(),
            BitsInner::Long(long) => long.content.iter().map(|c| c.count_ones()).sum(),
        }
    }

    fn num_bits(&self) -> u32 {
        match self {
            BitsInner::Short(num_bits, _) => *num_bits,
            BitsInner::Long(long) => long.num_bits,
        }
    }

    fn bitor_ones(&mut self, offset: u32, num_ones: u32) {
        if offset > self.num_bits() {
            return;
        }

        match self {
            BitsInner::Short(num_bits, ref mut bits) => {
                let mask = (1u64 << num_ones) - 1;
                let mask = mask << offset;
                let mask = mask & (1u64 << *num_bits).wrapping_sub(1);

                *bits |= mask;
            }
            BitsInner::Long(_) => todo!(),
        }
    }

    fn bitor_at(&mut self, offset: u32, rhs: &BitsInner) {
        if offset > self.num_bits() {
            return;
        }

        match (self, rhs) {
            (BitsInner::Short(num_bits, ref mut bits), BitsInner::Short(_, ref rhs_bits)) => {
                let mask = *rhs_bits;
                let mask = mask << offset;
                let mask = mask & (1u64 << *num_bits).wrapping_sub(1);
                *bits |= mask;
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
struct LongBits {
    content: Vec<u64>,
    num_bits: u32,
}

#[derive(Debug)]
pub enum BitsParseError {
    ExpectedBit(char),
    ExpectedUnderscoreBit(char),
    Empty,
}

impl Display for BitsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitsParseError::Empty => f.write_str("Got an empty string"),
            BitsParseError::ExpectedBit(gotten) => write!(f, "Expected '0' or '1'. Got '{gotten}'"),
            BitsParseError::ExpectedUnderscoreBit(gotten) => {
                write!(f, "Expected '0', '1' or '_'. Got '{gotten}'")
            }
        }
    }
}

impl TryFrom<String> for Bits {
    type Error = BitsParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: BitsInner::try_from(value)?,
        })
    }
}

impl TryFrom<String> for BitsInner {
    type Error = BitsParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(BitsParseError::Empty);
        }

        let mut num_bits = 0u32;
        let mut bits = 0u64;

        if value.starts_with('_') {
            return Err(BitsParseError::ExpectedBit('_'));
        }

        if value.ends_with('_') {
            return Err(BitsParseError::ExpectedBit('_'));
        }

        let mut chars = value.chars();

        let last = loop {
            let Some(c) = chars.next() else {
                return Ok(BitsInner::Short(num_bits, bits));
            };

            if c == '_' {
                continue;
            }

            if !matches!(c, '0' | '1') {
                return Err(BitsParseError::ExpectedUnderscoreBit(c));
            }

            let bit = u64::from(c == '1');

            if num_bits == 63 {
                break bit;
            }

            num_bits += 1;
            bits <<= 1;
            bits |= bit;
        };

        // TODO: This is most definitely wrong
        num_bits = 64;
        let mut content = Vec::new();
        content.push(bits);
        let mut bits = last;

        let last = loop {
            let Some(c) = chars.next() else {
                content.push(bits);
                return Ok(BitsInner::Long(Box::new(LongBits { num_bits, content })));
            };

            if c == '_' {
                continue;
            }

            if !matches!(c, '0' | '1') {
                return Err(BitsParseError::ExpectedUnderscoreBit(c));
            }

            let bit = u64::from(c == '1');

            num_bits += 1;

            if num_bits % 64 == 0 {
                content.push(bits);
                bits = 0u64;
            }

            bits <<= 1;
            bits |= bit;
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bits() {
        assert!(matches!(
            "0100011".to_string().try_into(),
            Ok(BitsInner::Short(7, 0b0100011))
        ));
    }
}
