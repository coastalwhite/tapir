use std::collections::HashMap;
use std::fs::File;

use std::cell::RefCell;
use std::io::{Read, Seek, Write};

use crate::driver::cache::{Cache, CacheResult};
use crate::driver::process::DriverProcess;
use crate::repr::{Addr, Size, Word};
use crate::util::u32_to_usize;

pub use ivt_segment_tree::{Segment, SegmentTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

pub struct PlacedBytes {
    start: Addr,
    bytes: Vec<u8>,
}

impl PlacedBytes {
    pub fn new(start: u32, bytes: Vec<u8>) -> Self {
        Self {
            start: Addr::from(start),
            bytes,
        }
    }

    pub fn end(&self) -> Addr {
        self.start.offset(self.bytes().len() as i32)
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    pub fn bytes_mut(&mut self) -> &mut Vec<u8> {
        self.bytes.as_mut()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

pub type AddressRegion = std::ops::RangeInclusive<u32>;

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    permissions: MemoryPermissions,
    driver: Driver,
    cache: Option<RefCell<Cache>>,
}

#[derive(Debug, Clone)]
pub struct MappedMemory {
    map: Vec<(AddressRegion, MemoryRegion)>,
    last_access_cache_result: RefCell<Option<CacheResult>>,
}

#[derive(Debug, Clone)]
pub enum Driver {
    StaticArray(StaticDataArray),
    DynamicArray(DynamicDataArray),
    Word(u32),
    File(DriverFile),
    Process(DriverProcess),
}

pub struct MappedMemoryBuilder {
    default_entry: Option<u32>,
    offset: u32,
    map: Vec<(AddressRegion, MemoryRegion)>,
}

#[derive(Debug, Clone)]
pub struct StaticDataArray {
    size: u32,
    data: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct DynamicDataArray {
    size: u32,
    data: HashMap<u32, [u32; Self::ALLOC_SIZE]>,
}

#[derive(Debug)]
pub struct DriverFile {
    size: u32,
    file: RefCell<File>,
}

impl Clone for DriverFile {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Driver {
    #[inline]
    pub fn size(&self) -> u32 {
        match self {
            Self::Word(_) => 4,
            Self::StaticArray(arr) => arr.size,
            Self::DynamicArray(arr) => arr.size,
            Self::File(file) => file.size,
            Self::Process(process) => process.size(),
        }
    }

    #[inline]
    pub fn with_permissions(self, permissions: MemoryPermissions) -> MemoryRegion {
        MemoryRegion {
            permissions,
            driver: self,
            cache: None,
        }
    }

    #[inline]
    pub fn with_permissions_and_cache(
        self,
        permissions: MemoryPermissions,
        cache: Cache,
    ) -> MemoryRegion {
        MemoryRegion {
            permissions,
            driver: self,
            cache: Some(RefCell::new(cache)),
        }
    }
}

impl StaticDataArray {
    #[inline]
    pub fn new(size: u32) -> Self {
        // TODO: Improve size bound
        Self {
            size,
            data: std::iter::repeat(0)
                .take(u32_to_usize(size / 4 + 1))
                .collect(),
        }
    }
}

impl MemoryPermissions {
    pub const READ: Self = Self {
        read: true,
        write: false,
        execute: false,
    };
    pub const READ_WRITE: Self = Self {
        read: true,
        write: true,
        execute: false,
    };
    pub const READ_EXECUTE: Self = Self {
        read: true,
        write: false,
        execute: true,
    };
    pub const READ_WRITE_EXECUTE: Self = Self {
        read: true,
        write: true,
        execute: true,
    };
}

impl MappedMemory {
    pub fn builder() -> MappedMemoryBuilder {
        MappedMemoryBuilder::new()
    }

    pub fn full() -> Self {
        Self {
            map: vec![(
                0..=0xFFFF_FFFF,
                MemoryRegion {
                    permissions: MemoryPermissions::READ_WRITE_EXECUTE,
                    driver: Driver::DynamicArray(DynamicDataArray::new(0xFFFF_FFFF)),
                    cache: None,
                },
            )],
            last_access_cache_result: RefCell::new(None),
        }
    }

    pub fn rwx_until(addr: u32) -> Self {
        Self {
            map: vec![(
                0..=addr - 1,
                MemoryRegion {
                    permissions: MemoryPermissions::READ_WRITE_EXECUTE,
                    driver: Driver::StaticArray(StaticDataArray::new(addr)),
                    cache: None,
                },
            )],
            last_access_cache_result: RefCell::new(None),
        }
    }

    pub fn get_region(&self, at: Addr) -> Option<(&MemoryRegion, u32)> {
        let at = &Word::from(at).as_u32();
        let (range, region) = self.map.iter().find(|(range, _)| range.contains(at))?;
        Some((region, at - range.start()))
    }

    pub fn get_region_mut(&mut self, at: Addr) -> Option<(&mut MemoryRegion, u32)> {
        let at = &Word::from(at).as_u32();
        let (range, region) = self.map.iter_mut().find(|(range, _)| range.contains(at))?;
        Some((region, at - range.start()))
    }

    pub fn get_last_access_cache_result(&self) -> Option<CacheResult> {
        self.last_access_cache_result.clone().into_inner()
    }

    pub fn get_without_cache(&self, at: Addr) -> u32 {
        let Some((region, offset)) = self.get_region(at) else {
            eprintln!("WARNING: Read from unmapped address 0x{at:08x}. Returning `0`.");
            return 0;
        };

        region.driver.get_le_bytes(Addr::from(offset))
    }
}

impl MappedMemoryBuilder {
    pub fn new() -> Self {
        Self {
            default_entry: None,
            offset: 0,
            map: Vec::default(),
        }
    }

    pub fn build(self) -> MappedMemory {
        // panic!(
        //     "{:#08x?}",
        //     self.map
        //         .iter()
        //         .map(|(range, _)| range.clone())
        //         .collect::<Vec<AddressRegion>>()
        // );
        MappedMemory {
            map: self.map,
            last_access_cache_result: RefCell::new(None),
        }
    }

    pub fn skip_to(mut self, offset: u32) -> Self {
        debug_assert!(self.offset < offset);
        self.offset = offset;
        self
    }

    pub fn region(mut self, region: MemoryRegion) -> Self {
        let size = region.driver.size();

        let offset_start = self.offset;

        self.offset = self.offset.checked_add(size).unwrap_or_else(|| {
            panic!("Invalid Address Space");
        });
        self.map
            .push((offset_start..=offset_start + size - 1, region));
        self
    }

    pub fn default_entry(mut self) -> Self {
        self.default_entry = Some(self.offset);
        self
    }
}

impl BackingStore for DynamicDataArray {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        if at.is_word_aligned() {
            if Word::from(at).as_u32() >= self.size {
                panic!("Out of range");
            }

            let block_index = Self::block_index(at);
            return if let Some(block) = self.data.get(&block_index) {
                block[Self::block_offset(at)]
            } else {
                0
            };
        }

        u32::from_le_bytes([
            self.get_byte(at.offset(0)),
            self.get_byte(at.offset(1)),
            self.get_byte(at.offset(2)),
            self.get_byte(at.offset(3)),
        ])
    }

    fn set_le_bytes(&mut self, at: Addr, word: u32) {
        if at.is_word_aligned() {
            if Word::from(at).as_u32() >= self.size {
                panic!("Out of range");
            }

            let block_index = Self::block_index(at);
            let block_offset = Self::block_offset(at);

            let block = self
                .data
                .entry(block_index)
                .or_insert([0; Self::ALLOC_SIZE]);
            block[block_offset] = word;
            return;
        }

        let [b0, b1, b2, b3] = word.to_le_bytes();

        self.set_byte(at.offset(0), b0);
        self.set_byte(at.offset(1), b1);
        self.set_byte(at.offset(2), b2);
        self.set_byte(at.offset(3), b3);
    }
}

impl BackingStore for StaticDataArray {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        if at.is_word_aligned() {
            let at = u32_to_usize(Word::from(at).as_u32());
            return match self.data.get(at / 4) {
                Some(word) => *word,
                None => {
                    eprintln!("ERROR: Out of range read of static array (size = 0x{:08x}, read addr = 0x{:08x}).", self.data.len(), at / 4);
                    std::process::exit(1)
                }
            };
        }

        u32::from_le_bytes([
            self.get_byte(at.offset(0)),
            self.get_byte(at.offset(1)),
            self.get_byte(at.offset(2)),
            self.get_byte(at.offset(3)),
        ])
    }

    fn set_le_bytes(&mut self, at: Addr, word: u32) {
        if at.is_word_aligned() {
            let at = u32_to_usize(Word::from(at).as_u32());
            match self.data.get_mut(at / 4) {
                Some(dword) => *dword = word,
                None => {
                    eprintln!("ERROR: Out of range write of static array (size = 0x{:08x}, write addr = 0x{:08x}, value = 0x{word:08x}).", self.data.len(), at / 4);
                    std::process::exit(1);
                }
            }
            return;
        }

        let [b0, b1, b2, b3] = word.to_le_bytes();

        self.set_byte(at.offset(0), b0);
        self.set_byte(at.offset(1), b1);
        self.set_byte(at.offset(2), b2);
        self.set_byte(at.offset(3), b3);
    }
}

impl BackingStore for DriverFile {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        let at = Word::from(at).as_u32();

        let mut file = self.file.borrow_mut();
        file.seek(std::io::SeekFrom::Start(at.into()))
            .expect("Failed to seek into file.");
        let mut buffer = [0u8; 4];
        let amount = file.read(&mut buffer).expect("Failed to read into file.");

        if amount < 4 {
            eprintln!(
                "WARNING: Failed to read (full) word from file. Filling remaining bytes with 0"
            );
            for i in (amount..4).rev() {
                buffer[i] = buffer[i - amount];
            }
        }

        // TODO: Make endianness configurable
        u32::from_be_bytes(buffer)
    }

    fn set_le_bytes(&mut self, at: Addr, word: u32) {
        let at = Word::from(at).as_u32();

        let mut file = self.file.borrow_mut();
        file.seek(std::io::SeekFrom::Start(at.into()))
            .expect("Failed to seek into file.");

        // TODO: Make endianness configurable
        let buffer = word.to_be_bytes();
        let amount = file.write(&buffer).expect("Failed to write to file.");

        if amount < 4 {
            eprintln!("WARNING: Failed to write full word into file.");
        }
    }
}

impl DynamicDataArray {
    const ALLOC_SIZE: usize = 1024;

    pub fn new(size: u32) -> Self {
        Self {
            size,
            data: HashMap::new(),
        }
    }

    pub fn block_index(at: Addr) -> u32 {
        at.in_blocks(Self::ALLOC_SIZE).0 as u32
    }

    pub fn block_offset(at: Addr) -> usize {
        (at.in_blocks(Self::ALLOC_SIZE).1) >> 2
    }

    pub fn num_allocated_blocks(&self) -> usize {
        self.data.len()
    }
}

impl BackingStore for Driver {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        match self {
            Driver::Word(ref word) => {
                debug_assert_eq!(at, 0);
                *word
            }
            Driver::StaticArray(ref array) => array.get_le_bytes(at),
            Driver::DynamicArray(ref array) => array.get_le_bytes(at),
            Driver::File(ref driver_file) => driver_file.get_le_bytes(at),
            Driver::Process(ref driver_process) => driver_process.get_le_bytes(at),
        }
    }
    fn set_le_bytes(&mut self, at: Addr, word: u32) {
        match self {
            Driver::Word(ref mut dword) => {
                debug_assert_eq!(at, 0);
                *dword = word;
            }
            Driver::StaticArray(ref mut array) => array.set_le_bytes(at, word),
            Driver::DynamicArray(ref mut array) => array.set_le_bytes(at, word),
            Driver::File(ref mut driver_file) => driver_file.set_le_bytes(at, word),
            Driver::Process(ref mut driver_process) => driver_process.set_le_bytes(at, word),
        }
    }
}

impl BackingStore for MappedMemory {
    fn is_executable(&self, at: Addr) -> bool {
        let Some((region, _)) = self.get_region(at) else {
            eprintln!("WARNING: Checking executable status for unmapped address. Returning false.");
            return false;
        };

        region.permissions.execute
    }

    #[inline]
    fn get_le_bytes(&self, at: Addr) -> u32 {
        self.last_access_cache_result.borrow_mut().take();

        let Some((region, offset)) = self.get_region(at) else {
            eprintln!("WARNING: Read from unmapped address 0x{at:08x}. Returning `0`.");
            return 0;
        };

        if let Some(cache_result) = region
            .cache
            .as_ref()
            .map(|cache| cache.borrow_mut().read(at))
        {
            *self.last_access_cache_result.borrow_mut() = Some(cache_result);
        }

        region.driver.get_le_bytes(Addr::from(offset))
    }

    fn set_le_bytes(&mut self, at: Addr, value: u32) {
        self.last_access_cache_result.borrow_mut().take();

        let Some((region, offset)) = self.get_region_mut(at) else {
            eprintln!("WARNING: Write of 0x{value:08x} to unmapped address 0x{at:08x}. Ignoring.");
            return;
        };

        region.driver.set_le_bytes(Addr::from(offset), value);

        if let Some(cache_result) = region
            .cache
            .as_ref()
            .map(|cache| cache.borrow_mut().write(at))
        {
            *self.last_access_cache_result.borrow_mut() = Some(cache_result);
        }
    }
}

pub trait BackingStore {
    fn get_le_bytes(&self, at: Addr) -> u32;
    fn set_le_bytes(&mut self, at: Addr, value: u32);

    #[inline]
    fn get_be_bytes(&self, at: Addr) -> u32 {
        self.get_le_bytes(at).swap_bytes()
    }
    #[inline]
    fn set_be_bytes(&mut self, at: Addr, value: u32) {
        self.set_le_bytes(at, value.swap_bytes())
    }

    #[inline]
    fn get(&self, at: Addr, endianness: Endianness) -> u32 {
        match endianness {
            Endianness::Little => self.get_le_bytes(at),
            Endianness::Big => self.get_be_bytes(at),
        }
    }
    #[inline]
    fn set(&mut self, at: Addr, value: u32, endianness: Endianness) {
        match endianness {
            Endianness::Little => self.set_le_bytes(at, value),
            Endianness::Big => self.set_be_bytes(at, value),
        }
    }

    fn is_executable(&self, _at: Addr) -> bool {
        true
    }

    fn write_to(&mut self, at: Addr, src: &[u8]) {
        for (i, b) in src.iter().enumerate() {
            let i = Size::from_usize(i).unwrap();
            // TODO: Use `set` instead of `set_byte`
            self.set_byte(at.end(i), *b);
        }
    }

    fn read_from(&self, at: Addr, length: Size) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(length.as_usize());

        for i in at.to(at.end(length)) {
            // TODO: Use `get` instead of `get_byte`
            buffer.push(self.get_byte(i));
        }

        buffer
    }

    fn get_byte(&self, at: Addr) -> u8 {
        let word = self.get_le_bytes(at.word_align());
        word.to_le_bytes()[at.word_offset() as usize]
    }

    fn set_byte(&mut self, at: Addr, byte: u8) {
        let word = self.get_le_bytes(at.word_align());
        let mut bytes = word.to_le_bytes();

        bytes[at.word_offset() as usize] = byte;

        self.set_le_bytes(at.word_align(), u32::from_le_bytes(bytes));
    }

    fn get_le_halfword(&self, at: Addr) -> u16 {
        let word_addr = Word::from(at).as_u32();
        if word_addr & 0x1 == 0 {
            let word = self.get_le_bytes(at.word_align());

            return if word_addr & 0x2 != 0 {
                ((word & 0xFFFF_0000) >> 16) as u16
            } else {
                (word & 0x0000_FFFF) as u16
            };
        }

        u16::from_le_bytes([self.get_byte(at.offset(0)), self.get_byte(at.offset(1))])
    }

    fn set_le_halfword(&mut self, at: Addr, halfword: u16) {
        let word = self.get_le_bytes(at.word_align());
        match at.word_offset() {
            0b00 => self.set_le_bytes(
                at.word_align(),
                (word & 0xFFFF_0000) | (u32::from(halfword.to_le()) << 00),
            ),
            0b01 => self.set_le_bytes(
                at.word_align(),
                (word & 0xFF00_00FF) | (u32::from(halfword.to_le()) << 08),
            ),
            0b10 => self.set_le_bytes(
                at.word_align(),
                (word & 0x0000_FFFF) | (u32::from(halfword.to_le()) << 16),
            ),
            0b11 => {
                let [b0, b1] = halfword.to_le_bytes();

                self.set_byte(at.offset(0), b0);
                self.set_byte(at.offset(1), b1);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn get_be_halfword(&self, at: Addr) -> u16 {
        self.get_le_halfword(at).swap_bytes()
    }
    #[inline]
    fn set_be_halfword(&mut self, at: Addr, value: u16) {
        self.set_le_halfword(at, value.swap_bytes())
    }

    #[inline]
    fn get_halfword(&self, at: Addr, endianness: Endianness) -> u16 {
        match endianness {
            Endianness::Little => self.get_le_halfword(at),
            Endianness::Big => self.get_be_halfword(at),
        }
    }
    #[inline]
    fn set_halfword(&mut self, at: Addr, value: u16, endianness: Endianness) {
        match endianness {
            Endianness::Little => self.set_le_halfword(at, value),
            Endianness::Big => self.set_be_halfword(at, value),
        }
    }
}

impl BackingStore for Vec<u8> {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        let at = u32_to_usize(Word::from(at).as_u32());

        if at > self.len() {
            panic!();
        }

        u32::from_le_bytes(self[at..at + 4].try_into().unwrap())
    }

    fn set_le_bytes(&mut self, at: Addr, value: u32) {
        let at = u32_to_usize(Word::from(at).as_u32());

        if at > self.len() {
            self.resize(at + 4, 0);
        }

        self[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_to(&mut self, at: Addr, src: &[u8]) {
        let at = u32_to_usize(Word::from(at).as_u32());

        if at > self.len() {
            self.resize(at + 4, 0);
        }

        self[at..at + src.len()].copy_from_slice(src);
    }

    fn set_byte(&mut self, at: Addr, byte: u8) {
        let at = u32_to_usize(Word::from(at).as_u32());
        self[at] = byte;
    }

    fn get_byte(&self, at: Addr) -> u8 {
        let at = u32_to_usize(Word::from(at).as_u32());
        self[at]
    }
}

impl BackingStore for PlacedBytes {
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

impl BackingStore for SegmentTree {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        let mut bytes = [0u8; 4];
        self.get_range(at.as_u32(), &mut bytes)
            .expect("Unable to load memory");
        u32::from_le_bytes(bytes)
    }

    fn set_le_bytes(&mut self, at: Addr, value: u32) {
        let value = value.to_le_bytes();
        self.fill_range(at.as_u32(), &value);
    }

    fn write_to(&mut self, at: Addr, src: &[u8]) {
        self.fill_range(at.as_u32(), src)
    }
}
