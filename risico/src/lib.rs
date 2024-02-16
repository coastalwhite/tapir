// mod decode;
mod csr;
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
pub use execute::State;

use self::device_config::{Isa, Section};
use self::memory::MappedMemory;
use self::ordered_sections::OrderedSections;
use self::syscall::SystemCallBehavior;
use self::trap::TrapBehavior;

pub struct RuntimeParameters {
    isa: Isa,
    syscall_behavior: SystemCallBehavior,
    trap_behavior: TrapBehavior,
    entry: u32,
    sections: OrderedSections,
}

impl RuntimeParameters {
    pub fn new(
        isa: Isa,
        syscall_behavior: SystemCallBehavior,
        trap_behavior: TrapBehavior,
        entry: u32,
        sections: Vec<Section>,
    ) -> Self {
        let sections = OrderedSections::new(sections).unwrap();

        Self {
            isa,
            syscall_behavior,
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
            self.syscall_behavior,
            self.trap_behavior,
            self.entry,
            self.allocate_memory(),
        )
    }
}
