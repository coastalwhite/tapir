// mod decode;
pub mod csr;
mod execute;
pub mod repr;

pub mod raw_image;

mod ordered_sections;

pub(crate) mod util;

pub mod device_config;
pub mod driver;
pub mod memory;
pub mod syscall;
pub mod trap;

// pub use decode::Instruction;
pub use execute::{State, StateECallBehavior};
use rvisa::MIsa;

use self::device_config::{Isa, Section};
use self::memory::MappedMemory;
use self::ordered_sections::OrderedSections;
use self::syscall::ECallBehavior;
use self::trap::TrapBehavior;

pub struct RuntimeParameters {
    isa: MIsa,
    ecall_behavior: StateECallBehavior,
    trap_behavior: TrapBehavior,
    entry: u32,
    sections: OrderedSections,
}

impl RuntimeParameters {
    pub fn new(
        isa: MIsa,
        ecall_behavior: StateECallBehavior,
        trap_behavior: TrapBehavior,
        entry: u32,
        sections: Vec<Section>,
    ) -> Self {
        let sections = OrderedSections::new(sections).unwrap();

        Self {
            isa,
            ecall_behavior,
            trap_behavior,
            entry,
            sections,
        }
    }

    pub fn allocate_memory(&self) -> MappedMemory {
        self.sections.memory_builder().build()
    }

    pub fn state(&self) -> State<MappedMemory> {
        State::new(
            self.isa,
            self.ecall_behavior,
            self.trap_behavior,
            self.entry,
            self.allocate_memory(),
            None,
        )
    }
}
