#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

use ivt_segment_tree::SegmentTree;

#[derive(Arbitrary, Debug)]
pub enum MemoryCommand {
    Insert(u32, u8),
    Fetch(u32),
}

const SIZE: usize = 2048;


fuzz_target!(|data: ([u8; SIZE], Vec<MemoryCommand>)| {
    let (mut arr, cmds) = data;
    let mut initialized = vec![false; SIZE as usize];
    let mut tree = SegmentTree::new();

    let size = SIZE as u32;

    for cmd in cmds {
        use MemoryCommand as C;
        match cmd {
            C::Insert(addr, v) => {
                arr[addr as usize] = v;
                let addr = addr % size;
                initialized[addr as usize] = true;
                tree.insert(addr, v);
            }
            C::Fetch(addr) => {
                if !initialized[addr as usize] {
                    tree.insert(addr, arr[addr as usize]);
                } else {
                    initialized[addr as usize] = true;
                    assert_eq!(tree.get(addr).unwrap(), arr[addr as usize]);
                }
            }
        }
    }
});
