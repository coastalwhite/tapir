use risico::memory::BackingStore;
use risico::repr::{Addr, Word};

use crate::MemoryArea;

pub struct ProgramMemory {
    pub bin: MemoryArea,
    pub memory_areas: Box<[MemoryArea]>,
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

impl ProgramMemory {
    pub fn find_area(&self, addr: Addr) -> Option<&MemoryArea> {
        if self.bin.contains_addr(addr) {
            return Some(&self.bin);
        }

        println!("addr: 0x{addr}");

        for area in self.memory_areas.iter() {
            if area.contains_addr(addr) {
                return Some(area);
            }
        }

        None
    }

    pub fn find_area_mut(&mut self, addr: Addr) -> Option<&mut MemoryArea> {
        if self.bin.contains_addr(addr) {
            return Some(&mut self.bin);
        }

        println!("addr: 0x{addr}");

        for area in self.memory_areas.iter_mut() {
            if area.contains_addr(addr) {
                return Some(area);
            }
        }

        None
    }

    pub fn unwrapped_find_area(&self, addr: Addr) -> &MemoryArea {
        self.find_area(addr).expect("Unable to find memory area")
    }

    pub fn unwrapped_find_area_mut(&mut self, addr: Addr) -> &mut MemoryArea {
        self.find_area_mut(addr).expect("Unable to find memory area")
    }
}

impl BackingStore for ProgramMemory {
    #[inline]
    fn get_le_bytes(&self, at: Addr) -> u32 {
        self.unwrapped_find_area(at).get_le_bytes(at)
    }

    #[inline]
    fn set_le_bytes(&mut self, at: Addr, value: u32) {
        self.unwrapped_find_area_mut(at).set_le_bytes(at, value)
    }

    #[inline]
    fn write_to(&mut self, at: Addr, src: &[u8]) {
        self.unwrapped_find_area_mut(at).write_to(at, src)
    }

    #[inline]
    fn set_byte(&mut self, at: Addr, byte: u8) {
        self.unwrapped_find_area_mut(at).set_byte(at, byte)
    }

    #[inline]
    fn get_byte(&self, at: Addr) -> u8 {
        self.unwrapped_find_area(at).get_byte(at)
    }
}
