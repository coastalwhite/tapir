macro_rules! decode_unsigned_fragmented {
    ($bits:expr$(, $h:literal$(:$l:literal)?)+ ; $padding:literal) => {{
        let mut out = 0;

        $(
        {
            const NUM_BITS: u32 = $($h-$l+)?1;
            const MASK: u32 = (1u32 << NUM_BITS).wrapping_sub(1);
            out <<= NUM_BITS;
            out |= ($bits >> $h$(-$h+$l)?) & MASK;
        }
        )+

        out << $padding
    }};
}

macro_rules! decode_signed_fragmented {
    ($bits:expr$(, $h:literal$(:$l:literal)?)+; $padding:literal) => {{
        let mut num_bits = 0;
        let mut out = 0;

        $(
        {
            const NUM_BITS: u32 = $($h-$l+)?1;
            num_bits += NUM_BITS;
            const MASK: u32 = (1u32 << NUM_BITS).wrapping_sub(1);
            out <<= NUM_BITS;
            out |= ($bits >> $h$(-$h+$l)?) & MASK;
        }
        )+

        let out = (out << (32 - num_bits)) as i32;
        out >> (32 - num_bits - $padding)
    }};
}

macro_rules! encode_fragmented {
    ($bits:expr$(, $h:literal$(:$l:literal)?)+ ; $padding:literal) => {{
        let bits: u32 = $bits as u32;
        let mut num_bits: u32 = $padding;
        $(
        {
            const NUM_BITS: u32 = $($h-$l+)?1;
            num_bits += NUM_BITS;
        }
        )+

        let mut out: u32 = 0;

        $(
        {
            const NUM_BITS: u32 = $($h-$l+)?1;
            const MASK: u32 = (1u32 << NUM_BITS).wrapping_sub(1);
            num_bits -= NUM_BITS;
            let value = (bits >> num_bits) & MASK;
            out |= value << $h$(-$h+$l)?;
        }
        )+

        out
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn decode_unsigned() {
        assert_eq!(decode_unsigned_fragmented!(0x1234, 12:0; 0) , 0x1234);
        assert_eq!(decode_unsigned_fragmented!(0x1234, 12:4; 4) , 0x1230);

        assert_eq!(decode_unsigned_fragmented!(0x1234, 12; 0) , 1);

        assert_eq!(decode_unsigned_fragmented!(0x1234, 12:11; 0) , 2);
        assert_eq!(decode_unsigned_fragmented!(0x1234, 12:11; 4) , 2 << 4 );

        assert_eq!(decode_unsigned_fragmented!(0x1234, 12:8,3:0; 0) , 0x124);
    }

    #[test]
    fn decode_signed() {
        assert_eq!(decode_signed_fragmented!(0x1234, 12:0; 0) , 0xFFFFF234u32 as i32);
        assert_eq!(decode_signed_fragmented!(0x1234, 13:0; 0) , 0x1234u32 as i32);
        assert_eq!(decode_signed_fragmented!(0x1234, 12:4; 4) , 0xFFFFF230u32 as i32);

        assert_eq!(decode_signed_fragmented!(0x1234, 12; 0) , 0xFFFF_FFFFu32 as i32);

        assert_eq!(decode_signed_fragmented!(0x1234, 12:11; 0) , 0xFFFF_FFFEu32 as i32);
        assert_eq!(decode_signed_fragmented!(0x1234, 12:11; 4) , 0xFFFF_FFE0u32 as i32);

        assert_eq!(decode_signed_fragmented!(0x1234, 12:8,3:0; 0) , 0xFFFF_FF24u32 as i32);
    }

    #[test]
    fn encode() {
        assert_eq!(encode_fragmented!(0x1234, 3:0,7:4,11:8,15:12; 0), 0x4321);
        assert_eq!(encode_fragmented!(0x1234, 3:0,7:4,11:8,15:12; 4), 0x3210);
    }
}