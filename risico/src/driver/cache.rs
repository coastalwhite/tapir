#![allow(unused)]

// @TODO: This whole module is a big todo.

use crate::repr::Addr;

type CacheSet = (Vec<CacheLine>, CachePolicyInstance);
type CacheLine = Option<u32>;

#[derive(Debug, Clone, Copy)]
pub enum CachePolicy {
    FIFO,
}

#[derive(Debug, Clone)]
enum CachePolicyInstance {
    FIFO(usize),
}

#[derive(Debug, Clone)]
pub struct Cache {
    associativity: usize,
    sets: Vec<CacheSet>,
    words_per_cache_line: usize,
    policy: CachePolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheResult {
    Hit,
    Miss,
}

enum AccessType {
    Read,
    Write,
}

impl CachePolicy {
    fn instance(self) -> CachePolicyInstance {
        match self {
            Self::FIFO => CachePolicyInstance::FIFO(0),
        }
    }
}

impl Cache {
    pub fn new(
        associativity: usize,
        sets: usize,
        words_per_cache_line: usize,
        policy: CachePolicy,
    ) -> Self {
        assert!(associativity.is_power_of_two());
        assert!(sets.is_power_of_two());
        assert!(words_per_cache_line.is_power_of_two());

        let rp_instance = policy.instance();

        Self {
            associativity,
            sets: vec![(vec![None; associativity], rp_instance); sets],
            words_per_cache_line,
            policy,
        }
    }

    fn get_tag(&self, addr: Addr) -> u32 {
        u32::from(addr) >> (self.sets.len().ilog2() + self.words_per_cache_line.ilog2())
    }

    fn get_index(&self, addr: Addr) -> u32 {
        (u32::from(addr) >> (self.words_per_cache_line.ilog2())) & (self.sets.len() - 1) as u32
    }

    #[inline]
    pub fn read(&mut self, addr: Addr) -> CacheResult {
        self.access(addr, AccessType::Read)
    }

    #[inline]
    pub fn write(&mut self, addr: Addr) -> CacheResult {
        self.access(addr, AccessType::Write)
    }

    fn access(&mut self, addr: Addr, access_type: AccessType) -> CacheResult {
        let index = self.get_index(addr);
        let tag = self.get_tag(addr);

        let (set, rp_policy) = self
            .sets
            .get_mut(index as usize)
            .expect("Sets does not contain all set indices");

        let matching_sets = set
            .iter()
            .enumerate()
            .filter(|(_, line)| match line {
                Some(line_tag) => *line_tag == tag,
                None => false,
            })
            .collect::<Vec<(usize, &Option<u32>)>>();
        
        match &matching_sets[..] {
            &[] => {
                *set.get_mut(rp_policy.replace()).expect("Invalid Result from RP") = Some(tag);
                CacheResult::Miss
            }
            &[(offset, Some(_))] => {
                match access_type {
                    AccessType::Read => rp_policy.read(offset),
                    AccessType::Write => rp_policy.write(offset),
                }

                CacheResult::Hit
            }
            _ => {
                unreachable!()
            }
        }
    }
}

impl CachePolicyInstance {
    fn replace(&mut self) -> usize {
        match self {
            Self::FIFO(ref mut offset) => {
                let replacement_offset = *offset;
                *offset += 1;
                replacement_offset
            },
        }
    }

    fn read(&mut self, offset: usize) {}
    fn write(&mut self, offset: usize) {}
}
