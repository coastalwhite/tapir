use crate::bits::Bits;

#[derive(Debug)]
pub struct BytesMask {
    content: Box<[BytesMaskCell]>,
}

impl BytesMask {
    pub fn new(num_bytes: usize) -> Self {
        let mut content = vec![
            BytesMaskCell {
                enable: 0,
                value: 0
            };
            num_bytes
        ]
        .into_boxed_slice();
        Self { content }
    }

    fn add_to(&mut self, byte_idx: usize, cell: BytesMaskCell) {
        assert!(self.content[byte_idx].enable & cell.enable == 0);

        self.content[byte_idx].enable |= cell.enable;
        self.content[byte_idx].value |= cell.value;
    }

    pub fn set_bits_at(&mut self, bits: &Bits, at: u32) {
        assert!(bits.num_bits() + at <= self.content.len() as u32 * 8);
        assert!(bits.num_bits() != 0);

        let start_byte = at / 8;
        let end_byte = (at + bits.num_bits()) / 8;

        if start_byte == end_byte {
            let in_byte_offset = at % 8;
            let enable = (1u8 << bits.num_bits()).wrapping_sub(1) << in_byte_offset;
            let value = (bits.as_u32().unwrap() as u8) << in_byte_offset;

            self.add_to(start_byte as usize, BytesMaskCell { enable, value });
        } else {
            // todo!("Unfinished");

            let in_byte_offset = at % 8;

            let enable = 255 ^ (1u8 << in_byte_offset).wrapping_sub(1);
            let value = bits.le_byte(0) << in_byte_offset;

            self.add_to(start_byte as usize, BytesMaskCell { enable, value });

            for byte_idx in start_byte + 1..end_byte {
                let enable = 255;
                let leftover =
                    bits.le_byte((byte_idx - start_byte) as usize - 1) >> (8 - in_byte_offset);
                let new = bits.le_byte((byte_idx - start_byte) as usize) << in_byte_offset;
                let value = leftover | new;

                self.add_to(start_byte as usize, BytesMaskCell { enable, value });
            }

            // @TODO: END BYTE
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BytesMaskCell {
    enable: u8,
    value: u8,
}
