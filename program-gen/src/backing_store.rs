use risico::memory::{BackingStore, SegmentTree};
use risico::repr::{Addr, Word};

use crate::MemoryArea;

pub struct ProgramMemory {
    pub bin: MemoryArea,
    pub memory_areas: SegmentTree,
}

pub fn u32_to_usize(x: u32) -> usize {
    // @Hack
    x as usize
}

impl BackingStore for MemoryArea {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        if at < self.start {
            panic!();
        }

        let offset = u32_to_usize(Word::from(at).as_u32() - Word::from(self.start).as_u32());

        if offset > self.bytes.len() {
            panic!();
        }

        u32::from_le_bytes(self.bytes[offset..offset + 4].try_into().unwrap())
    }

    fn set_le_bytes(&mut self, at: Addr, value: u32) {
        if at < self.start {
            panic!();
        }

        let offset = u32_to_usize(Word::from(at).as_u32() - Word::from(self.start).as_u32());

        if offset > self.bytes.len() {
            panic!();
        }

        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_to(&mut self, at: Addr, src: &[u8]) {
        if at < self.start {
            panic!();
        }

        let offset = u32_to_usize(Word::from(at).as_u32() - Word::from(self.start).as_u32());

        if offset + src.len() > self.bytes.len() {
            panic!();
        }

        self.bytes[offset..offset + src.len()].copy_from_slice(src);
    }

    fn set_byte(&mut self, at: Addr, byte: u8) {
        if at < self.start {
            panic!();
        }

        let offset = u32_to_usize(Word::from(at).as_u32() - Word::from(self.start).as_u32());
        self.bytes[offset] = byte;
    }

    fn get_byte(&self, at: Addr) -> u8 {
        if at < self.start {
            panic!();
        }

        let offset = u32_to_usize(Word::from(at).as_u32() - Word::from(self.start).as_u32());
        self.bytes[offset]
    }
}

impl BackingStore for ProgramMemory {
    #[inline]
    fn get_le_bytes(&self, at: Addr) -> u32 {
        if self.bin.contains_addr(at) {
            return self.bin.get_le_bytes(at);
        }

        self.memory_areas.get_le_bytes(at)
    }

    #[inline]
    fn set_le_bytes(&mut self, at: Addr, value: u32) {
        if self.bin.contains_addr(at) {
            return self.bin.set_le_bytes(at, value);
        }

        self.memory_areas.set_le_bytes(at, value)
    }

    #[inline]
    fn write_to(&mut self, at: Addr, src: &[u8]) {
        if self.bin.contains_addr(at) {
            return self.bin.write_to(at, src);
        }

        self.memory_areas.write_to(at, src)
    }

    #[inline]
    fn set_byte(&mut self, at: Addr, byte: u8) {
        if self.bin.contains_addr(at) {
            return self.bin.set_byte(at, byte);
        }

        self.memory_areas.set_byte(at, byte)
    }

    #[inline]
    fn get_byte(&self, at: Addr) -> u8 {
        if self.bin.contains_addr(at) {
            return self.bin.get_byte(at);
        }

        self.memory_areas.get_byte(at)
    }
}
