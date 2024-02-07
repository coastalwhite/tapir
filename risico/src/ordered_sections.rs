use crate::device_config::{
    Driver as DCDriver, ProcessDriver, Section, SectionAccessPolicy, StoreAllocationKind,
    StoreDriver, StoreInitialization, CacheReplacementPolicy,
};
use crate::driver::cache::{Cache, CachePolicy};
use crate::driver::process::DriverProcess;
use crate::memory::{
    BackingStore, Driver, DynamicDataArray, MappedMemoryBuilder, MemoryPermissions, MemoryRegion,
    StaticDataArray,
};
use crate::repr::Addr;

use object::{Object, ObjectSection};

pub struct OrderedSections(Vec<Section>);

impl OrderedSections {
    pub fn new(mut sections: Vec<Section>) -> Result<Self, ()> {
        sections.sort_by_key(Section::start);

        let has_overlapping_sections = sections
            .windows(2)
            .any(|w| u32::max(w[0].start(), w[1].start()) <= u32::min(w[0].end(), w[1].end()));

        if has_overlapping_sections {
            return Err(());
        }

        Ok(Self(sections))
    }

    pub fn memory_builder(&self) -> MappedMemoryBuilder {
        let mut mapped_memory_builder = MappedMemoryBuilder::new();

        let offset = 0u32;
        for section in self.0.iter() {
            // TODO: This is kind of dirty
            let SectionAccessPolicy {
                read,
                write,
                execute,
            } = *section.access_policy();
            let permissions = MemoryPermissions {
                read,
                write,
                execute,
            };

            let driver = match section.driver() {
                DCDriver::Store(store_driver) => {
                    let mut driver = match store_driver.allocate() {
                        StoreAllocationKind::OnDemand => {
                            Driver::DynamicArray(DynamicDataArray::new(section.size()))
                        }
                        StoreAllocationKind::Preallocate => {
                            Driver::StaticArray(StaticDataArray::new(section.size()))
                        }
                    };

                    match store_driver.initialization() {
                        StoreInitialization::Zeroed => {}
                        StoreInitialization::Random => {
                            match store_driver.allocate() {
                                StoreAllocationKind::OnDemand => unimplemented!(),
                                StoreAllocationKind::Preallocate => {
                                    let mut rng = rand::thread_rng();
                                    let rnd_data = (0..section.size())
                                        .map(|_| rand::Rng::gen(&mut rng))
                                        .collect::<Vec<u8>>();
                                    driver.write_to(0u32.into(), &rnd_data);
                                }
                            };
                        }
                        StoreInitialization::File(path) => {
                            // Deal with ELF files
                            let contents = if path.starts_with("elf:") {
                                let path = &path[4..];
                                let contents = std::fs::read(path).unwrap_or_else(|err| {
                                    eprintln!("[ERROR]: Failed to open '{path}'. Reason: {err}",);
                                    std::process::exit(1);
                                });

                                let obj_file = object::File::parse(&*contents)
                                    .expect("Failed to parse as object file");

                                if obj_file.format() != object::BinaryFormat::Elf {
                                    eprintln!("ERROR: Given binary is not an ELF binary");
                                    std::process::exit(1);
                                }

                                if obj_file.architecture() != object::Architecture::Riscv32 {
                                    eprintln!("ERROR: Given binary is not a riscv32 binary");
                                    std::process::exit(1);
                                }

                                // TODO: Make section customizable
                                let Some(elf_section) = obj_file.section_by_name(".text") else {
                                    eprintln!("ERROR: Given binary does not contain `.text` section");
                                    std::process::exit(1);
                                };

                                elf_section
                                    .data()
                                    .expect("Section data not available")
                                    .to_vec()
                            } else {
                                std::fs::read(path).unwrap_or_else(|err| {
                                    eprintln!("[ERROR]: Failed to open '{path}'. Reason: {err}",);
                                    std::process::exit(1);
                                })
                            };

                            if contents.len() as u32 > section.size() {
                                eprintln!("[ERROR]: Contents of '{path}' to large for section.",);
                                std::process::exit(1);
                            }

                            driver.write_to(Addr::from(0u32), &contents);
                        }
                    }

                    driver
                }
                DCDriver::Process(ProcessDriver { path, args: _args }) => {
                    Driver::Process(DriverProcess::new(section.size(), path).unwrap())
                }
                _ => unimplemented!(),
            };

            if section.start() != offset {
                mapped_memory_builder = mapped_memory_builder.skip_to(section.start());
            }

            let driver = match section.cache() {
                Some(cache) => {
                    let cache = Cache::new(cache.associativity, cache.sets as usize, cache.cache_line_words as usize, match cache.replacement {
                        CacheReplacementPolicy::FirstInFirstOut => CachePolicy::FIFO,
                        CacheReplacementPolicy::Random => unimplemented!(),
                    });
                    driver.with_permissions_and_cache(permissions, cache)
                },
                None => driver.with_permissions(permissions),
            };

            mapped_memory_builder = mapped_memory_builder.region(driver);
        }

        mapped_memory_builder
    }
}
