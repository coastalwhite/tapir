use std::io::{self, Write};
use std::path::Path;

use crate::device_config::Isa;
use crate::memory::{BackingStore, MappedMemory};
use crate::repr::Addr;
use crate::trap::TrapBehavior;
use crate::{State, ECallBehavior};

pub struct RawImage {
    buffer: Vec<u8>,
}

impl RawImage {
    pub fn new(buffer: impl Into<Vec<u8>>) -> Self {
        Self {
            buffer: buffer.into(),
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(Self::new(std::fs::read(path)?))
    }

    pub fn dump(&self) -> io::Result<()> {
        let mut stdout = std::io::stdout().lock();

        for (i, instr_data) in self.buffer.chunks(4).enumerate() {
            let i = i * 4;
            let i_len = format!("{i}").len();
            let Ok(instr_data) = instr_data.try_into() else {
                write!(
                    stdout,
                    "{i}:{i_padding}",
                    i_padding = " ".repeat(12 - i_len - 1)
                )?;
                for b in instr_data {
                    write!(stdout, "{b:02x}")?;
                }
                writeln!(stdout)?;
                break;
            };

            let instr_data = u32::from_le_bytes(instr_data);
            let asm = rvhwfuzzer_encoding::Instruction::decode(&mut &instr_data.to_le_bytes()[..])?
                .map_or("<???>".to_string(), |instruction| instruction.to_string());

            writeln!(
                stdout,
                "{i}:{i_padding}{instr_data:08x}    {asm}",
                i_padding = " ".repeat(12 - i_len - 1)
            )?;
        }

        Ok(())
    }

    pub fn execute(
        &self,
        entry: u32,
        syscall_behavior: ECallBehavior,
        trap_behavior: TrapBehavior,
    ) {
        let mut memory = MappedMemory::full();
        memory.write_to(Addr::default(), &self.buffer);
        let mut state = State::new(Isa::Rv32I, syscall_behavior, trap_behavior, entry, memory);

        loop {
            state.execute_mut();
        }
    }
}
