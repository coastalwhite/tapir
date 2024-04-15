pub struct RandomBits {
    source: fastrand::Rng,
    current: u64,
    remaining: u32,
}

impl RandomBits {
    pub fn new() -> Self {
        Self {
            source: Default::default(),
            current: 0,
            remaining: 0,
        }
    }

    pub fn take(&mut self, num_bits: u32) -> u64 {
        debug_assert!(num_bits > 0);
        debug_assert!(num_bits <= 64);

        // The short path where the current still has enough entropy bits to feed the request.
        if self.remaining > num_bits {
            let mask = 1u64.wrapping_shl(num_bits).wrapping_sub(1);
            let result = self.current & mask;

            self.current >>= num_bits;
            self.remaining -= num_bits;

            return result;
        }

        // The long path where we need to feed more entropy bits.
        let result = self.current << self.remaining;

        let num_bits = num_bits - self.remaining;

        let new: u64 = self.source.u64(..);

        self.remaining = 64 - num_bits;
        self.current = new >> num_bits;

        let mask = 1u64.wrapping_shl(num_bits).wrapping_sub(1);

        result & (new & mask)
    }

    pub fn take_u8(&mut self, num_bits: u32) -> u8 {
        debug_assert!(num_bits <= 8);
        self.take(num_bits) as u8
    }

    pub fn take_u32(&mut self, num_bits: u32) -> u32 {
        debug_assert!(num_bits <= 32);
        self.take(num_bits) as u32
    }
}