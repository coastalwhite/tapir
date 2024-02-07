use std::io::stdout;
use std::io::Write;

use risico::device_config::{Document, Isa};
use risico::memory::{BackingStore, MappedMemory};
use risico::raw_image::RawImage;
use risico::{RuntimeParameters, State};

use rvhwfuzzer_encoding::Instruction;

use object::Endianness;
use object::Object;
use object::ObjectSection;
use object::ObjectSymbol;
use object::SectionFlags;
use risico::repr::Addr;
use rvhwfuzzer_encoding::asm::AsmDisplay;

use crate::cli::{CliFlags, RunType};

mod cli;

fn main() {
    let cli = CliFlags::take();

    match cli.run_type() {
        RunType::SyscallEmulation => {
            let elf_data = std::fs::read(cli.file()).expect("Failed to open the file");
            let obj_file = object::File::parse(&*elf_data).expect("Failed to parse as object file");

            if obj_file.format() != object::BinaryFormat::Elf {
                eprintln!("ERROR: Given binary is not an ELF binary");
                std::process::exit(1);
            }

            if obj_file.architecture() != object::Architecture::Riscv32 {
                eprintln!("ERROR: Given binary is not a riscv32 binary");
                std::process::exit(1);
            }

            if cli.do_dump() {
                assert_eq!(obj_file.endianness(), Endianness::Little);

                if let Some(section) = obj_file.section_by_name(".text") {
                    let section_index = section.index();
                    let section_data = section.data().expect("Section data not available");
                    let section_start = section.address() as usize;

                    struct Symbol<'a> {
                        name: &'a str,
                        start: usize,
                    }

                    let mut symbols = obj_file
                        .symbols()
                        .filter(|s| s.section_index().map_or(false, |i| i == section_index))
                        .map(|symbol| Symbol {
                            name: symbol.name().expect("No Symbol Name"),
                            start: (symbol.address() as usize) - section_start,
                        })
                        .collect::<Vec<Symbol>>();

                    symbols.sort_by_key(|symbol| symbol.start);

                    let mut symbol_index = 0;

                    let chunks = section_data.chunks(4);
                    let instructions = chunks.map(|chunk| {
                        let num: &[u8; 4] = chunk.try_into().unwrap();
                        let num = u32::from_le_bytes(num.clone());

                        num
                    });

                    let mut stdout = stdout().lock();

                    for (i, hex) in instructions.enumerate() {
                        if symbols
                            .get(symbol_index)
                            .map_or(false, |symbol| symbol.start == i * 4)
                        {
                            let Symbol { start, name } = symbols.get(symbol_index).unwrap();
                            let start = section_start + start;
                            writeln!(stdout).expect("Fail stdout write");
                            writeln!(stdout, "{start:08x} <{name}>:").expect("Fail to write");
                            symbol_index += 1;
                        }

                        let offset = section_start + i * 4;
                        let instruction = Instruction::decode(&hex.to_le_bytes())
                            .map_or("<???>".to_string(), |instruction| AsmDisplay { ctx: &Default::default(), instr: &instruction }.to_string());
                        writeln!(stdout, "{offset:08x}:\t{hex:08x}\t\t{instruction}")
                            .expect("Failed to write");
                    }
                } else {
                    eprintln!("section not available");
                }

                return;
            }

            let entry = obj_file.entry() as u32;

            let mut memory = MappedMemory::full();
            for section in obj_file.sections() {
                let SectionFlags::Elf { sh_flags } = section.flags() else {
                    eprintln!("ERROR: Section is not ELF");
                    std::process::exit(1);
                };

                // let write = sh_flags & 0x1 != 0;
                let allocate = sh_flags & 0x2 != 0;
                // let execute = sh_flags & 0x4 != 0;

                if allocate {
                    memory.write_to(
                        Addr::from(section.address() as u32),
                        section.data().expect("No data here"),
                    );
                }
            }

            let mut state = State::new(Isa::Rv32I, cli.system_call_behavior(), entry, memory);

            let mut trace_file = cli.trace().map(|trace| {
                std::fs::OpenOptions::new()
                    .read(false)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(trace)
                    .unwrap_or_else(|_| {
                        eprintln!("[ERROR]: Failed to open trace file '{trace}'");
                        std::process::exit(1);
                    })
            });

            loop {
                trace_file.as_mut().map(|f| {
                    writeln!(
                        f,
                        "{rdcycle}/{pc}",
                        rdcycle = state.rdcycle(),
                        pc = state.pc()
                    )
                    .expect("Failed to write to trace file");
                });

                if cli.logging().show_instructions {
                    let hex = state.instruction_fetch().as_u32();

                    if cli.logging().show_cycles {
                        let rdcycle = state.rdcycle();
                        print!("[{rdcycle:010}]\t");
                    }

                    // if let Ok(instruction) = Instruction::try_from(hex) {
                    //     println!("{instruction}\t(= 0x{hex:08x})");
                    // } else {
                    //     println!("<???>\t(= 0x{hex:08x})");
                    // }
                }

                state.execute_mut();
            }
        }
        RunType::RawImage(flags) => {
            let raw_image = RawImage::open(cli.file()).expect("Failed to open raw image");

            if cli.do_dump() {
                raw_image.dump().expect("Failed to print dump");
                return;
            }

            raw_image.execute(flags.entry(), cli.system_call_behavior());

            todo!()
        }
        RunType::DeviceConfig(_) => {
            let file_path = cli.file();
            let device_config = std::fs::read_to_string(file_path).unwrap_or_else(|err| {
                eprintln!(
                    "[ERROR]: Device config file '{file_path}' cannot be openend. Reason: {err}"
                );
                std::process::exit(2);
            });

            let device_config = Document::from_toml(&device_config).unwrap_or_else(|err| {
                eprintln!("[ERROR]: Device config file '{file_path}' invalid. Reason: {err}");
                std::process::exit(1);
            });

            let runtime_parameters = RuntimeParameters::new(
                Isa::Rv32I,
                cli.system_call_behavior(),
                0x0,
                device_config.take_sections(),
            );

            let mut state = runtime_parameters.state();

            let mut trace_file = cli.trace().map(|trace| {
                std::fs::OpenOptions::new()
                    .read(false)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(trace)
                    .unwrap_or_else(|_| {
                        eprintln!("[ERROR]: Failed to open trace file '{trace}'");
                        std::process::exit(1);
                    })
            });

            loop {
                trace_file.as_mut().map(|f| {
                    writeln!(
                        f,
                        "{rdcycle}/{pc}",
                        rdcycle = state.rdcycle(),
                        pc = state.pc()
                    )
                    .expect("Failed to write to trace file");
                });

                if cli.logging().show_instructions {
                    let hex = state.instruction_fetch().as_u32();

                    if cli.logging().show_cycles {
                        let rdcycle = state.rdcycle();
                        print!("[{rdcycle:010}]\t");
                    }

                    // if let Ok(instruction) = Instruction::try_from(hex) {
                    //     println!("{instruction}\t(= 0x{hex:08x})");
                    // } else {
                    //     println!("<???>\t(= 0x{hex:08x})");
                    // }
                }

                state.execute_mut();
            }
        }
    }
}
