//! ```toml
//! project_name = "Project Name"
//! ```
//!
//! ```toml
//! isa = "rv32i"
//! ```
//!
//! ```toml
//! [[mem]]
//! from = 0x1A10_1000
//! size = 4
//! ```
//!
//! ```toml
//! [[mem]]
//! from = 0x1A10_1004
//! to   = 0x1A10_1008
//! ```
//!
//! ```toml
//! [[mem]]
//! name = "GPIO In"
//! from = 0x1A10_1008
//! size = 4
//! ```
//!
//! ```toml
//! access = { read = true, write = true, execute = true }
//! ```
//!
//!
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    project_name: String,
    instruction_set: Isa,
    sections: Vec<Section>,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, PartialEq)]
pub enum Isa {
    #[default]
    #[serde(rename = "rv32i")]
    Rv32I,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    name: String,

    start: u32,
    end: u32,

    access: SectionAccessPolicy,
    driver: Driver,
    caching: Option<SectionCaching>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct SectionAccessPolicy {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Driver {
    Store(StoreDriver),
    File(FileDriver),
    Process(ProcessDriver),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoreDriver {
    initialization: StoreInitialization,
    allocate: StoreAllocationKind,
    format: BackingFileFormat,
}

impl StoreDriver {
    pub fn initialization(&self) -> &StoreInitialization {
        &self.initialization
    }

    pub fn allocate(&self) -> &StoreAllocationKind {
        &self.allocate
    }

    pub fn format(&self) -> &BackingFileFormat {
        &self.format
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileDriver {
    path: PathBuf,
    format: BackingFileFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDriver {
    pub path: PathBuf,
    pub args: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum StoreInitialization {
    #[default]
    Zeroed,
    Random,
    File(String),
}

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub enum StoreAllocationKind {
    #[default]
    #[serde(rename = "on-demand")]
    OnDemand,

    #[serde(rename = "preallocate")]
    Preallocate,
}

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub enum BackingFileFormat {
    #[default]
    #[serde(rename = "auto")]
    Automatic,
    #[serde(rename = "elf")]
    Elf,
    #[serde(rename = "hex")]
    Hex,
    #[serde(rename = "binary")]
    Binary,
    #[serde(rename = "binary-ascii")]
    BinaryASCII,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SectionCaching {
    // placement: CachePlacementPolicy,
    pub replacement: CacheReplacementPolicy,
    // writing_hit: CacheWritingHitPolicy,
    // writing_miss: CacheWritingMissPolicy,
    pub associativity: usize,

    pub sets: u32,
    pub cache_line_words: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum CachePlacementPolicy {
    #[serde(rename = "set-associative")]
    SetAssociative,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum CacheReplacementPolicy {
    #[serde(rename = "fifo")]
    FirstInFirstOut,
    #[serde(rename = "random")]
    Random,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
enum CacheWritingHitPolicy {
    #[serde(rename = "write-back")]
    WriteBack,
    #[serde(rename = "write-through")]
    WriteThrough,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
enum CacheWritingMissPolicy {
    #[serde(rename = "write-allocate")]
    WriteAllocate,
    #[serde(rename = "write-around")]
    WriteNoAllocate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentParseError {
    TomlError(toml::de::Error),

    MissingKey(String, &'static str),

    AmbigousEnd(String),
    UnknownEnd(String),
    InvalidMemoryRange(String),
    AmbigousInitialize(String),
}

impl Display for DocumentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TomlError(err) => err.fmt(f),
            Self::MissingKey(section, key) => write!(f, "Section '{section}' is missing key '{key}'"),
            Self::AmbigousEnd(section) => write!(f, "Section '{section}' has both an 'end' address and a 'size'. Please only give one."),
            Self::UnknownEnd(section) => write!(f, "End address for section '{section}' could not be calculated. Give either an 'end' address or a 'size'."),
            Self::InvalidMemoryRange(section) => write!(f, "Memory range for section '{section}' is invalid. Check that the start address is smaller than the end address."),
            Self::AmbigousInitialize(section) => write!(f, "Driver for section '{section}' has both a 'initialize_with' and 'initialize_with_file' key. Please only give one."),
        }
    }
}

impl Error for DocumentParseError {}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigDocument {
    project_name: Option<String>,

    #[serde(default)]
    instruction_set: Isa,

    section: HashMap<String, ConfigSection>,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigSection {
    name: Option<String>,

    start: u32,

    end: Option<u32>,
    size: Option<u32>,

    #[serde(default)]
    access: SectionAccessPolicy,

    driver: Option<ConfigDriver>,
    caching: Option<SectionCaching>,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigDriver {
    #[serde(default)]
    kind: ConfigDriverKind,

    path: Option<PathBuf>,

    #[serde(default)]
    allocate: StoreAllocationKind,

    // Store Initialization
    initialize_with: Option<ConfigInitializeWith>,
    initialize_with_file: Option<String>,

    args: Option<String>,
    #[serde(default)]
    format: BackingFileFormat,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub enum ConfigDriverKind {
    #[default]
    #[serde(rename = "store")]
    Store,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "process")]
    Process,
}

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub enum ConfigInitializeWith {
    #[default]
    #[serde(rename = "zeroed")]
    Zeroed,
    #[serde(rename = "random")]
    Random,
}

impl Document {
    pub fn from_toml(s: &str) -> Result<Self, DocumentParseError> {
        toml::from_str::<ConfigDocument>(s)
            .map_err(|e| DocumentParseError::TomlError(e))
            .and_then(|d| d.try_into())
    }

    pub fn take_sections(self) -> Vec<Section> {
        self.sections
    }
}

impl TryFrom<ConfigDocument> for Document {
    type Error = DocumentParseError;

    fn try_from(config: ConfigDocument) -> Result<Self, Self::Error> {
        let mut sections = Vec::with_capacity(config.section.len());

        for (name, section) in config.section.into_iter() {
            let driver = Section::from_config_with_name(section, name)?;
            sections.push(driver);
        }

        Ok(Self {
            // TODO: Give a better default name
            project_name: config.project_name.unwrap_or_default(),
            instruction_set: config.instruction_set,
            sections,
        })
    }
}


impl Section {
    fn from_config_with_name(
        config: ConfigSection,
        name: String,
    ) -> Result<Self, DocumentParseError> {
        use DocumentParseError::*;

        let start = config.start;
        let end = match (config.end, config.size) {
            (Some(_), Some(_)) => return Err(AmbigousEnd(name)),

            (Some(end), _) => end,
            (_, Some(size)) => start + size,

            (None, None) => return Err(UnknownEnd(name)),
        };

        if start >= end {
            return Err(InvalidMemoryRange(name));
        }

        let name = config.name.unwrap_or(name);

        let access = config.access;
        let driver = match config.driver {
            Some(driver) => Driver::try_from_with_name(driver, &name)?,
            None => Driver::default(),
        };
        let caching = config.caching;

        Ok(Self {
            name,
            start,
            end,
            access,
            driver,
            caching,
        })
    }
    
    pub fn start(&self) -> u32 {
        self.start
    }

    pub fn end(&self) -> u32 {
        self.end
    }

    pub fn range(&self) -> std::ops::Range<u32> {
        self.start..self.end
    }
    
    pub fn access_policy(&self) -> &SectionAccessPolicy {
        &self.access
    }

    pub fn size(&self) -> u32 {
        self.end().abs_diff(self.start())
    }

    pub fn driver(&self) -> &Driver {
        &self.driver
    }

    pub fn cache(&self) -> Option<&SectionCaching> {
        self.caching.as_ref()
    }
}

impl Driver {
    fn try_from_with_name(config: ConfigDriver, name: &str) -> Result<Self, DocumentParseError> {
        enum DriverKindI {
            Store,
            File,
            Process,
        }

        let kind = match config.kind {
            ConfigDriverKind::Store => DriverKindI::Store,
            ConfigDriverKind::File => DriverKindI::File,
            ConfigDriverKind::Process => DriverKindI::Process,
        };

        match kind {
            DriverKindI::Store => {
                let initialize_with = config.initialize_with;
                let initialize_with_file = config.initialize_with_file;

                let initialization = match (initialize_with, initialize_with_file) {
                    (Some(_), Some(_)) => {
                        return Err(DocumentParseError::AmbigousInitialize(name.to_string()))
                    }
                    (None, None) => StoreInitialization::default(),
                    (Some(i), _) => i.into(),
                    (_, Some(i)) => StoreInitialization::File(i),
                };

                let allocate = config.allocate;
                let format = config.format;

                let store_driver = StoreDriver {
                    initialization,
                    allocate,
                    format,
                };

                Ok(Driver::Store(store_driver))
            }
            DriverKindI::File => {
                let path = config
                    .path
                    .ok_or(DocumentParseError::MissingKey(name.to_string(), "path"))?;
                let format = config.format;

                let file_driver = FileDriver { path, format };

                Ok(Driver::File(file_driver))
            }
            DriverKindI::Process => {
                let path = config
                    .path
                    .ok_or(DocumentParseError::MissingKey(name.to_string(), "path"))?;
                let args = config.args;

                let process_driver = ProcessDriver { path, args };

                Ok(Driver::Process(process_driver))
            }
        }
    }
}

impl Default for Driver {
    fn default() -> Self {
        Self::Store(StoreDriver {
            initialization: StoreInitialization::default(),
            allocate: StoreAllocationKind::OnDemand,
            format: BackingFileFormat::default(),
        })
    }
}



impl Default for SectionAccessPolicy {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }
}

impl From<ConfigInitializeWith> for StoreInitialization {
    fn from(value: ConfigInitializeWith) -> Self {
        match value {
            ConfigInitializeWith::Zeroed => Self::Zeroed,
            ConfigInitializeWith::Random => Self::Random,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_driver_eq {
        ($s:literal, $driver:expr) => {
            let driver = toml::from_str::<ConfigDriver>($s)
                .map_err(|e| DocumentParseError::TomlError(e))
                .and_then(|d| Driver::try_from_with_name(d, "test case"));

            assert!(driver.is_ok(), "{err:?}", err = driver.unwrap_err());
            assert_eq!(driver.unwrap(), $driver);
        };
        ($s:literal, ) => {
            let driver = toml::from_str::<ConfigDriver>($s)
                .map_err(|e| DocumentParseError::TomlError(e))
                .and_then(|d| Driver::try_from_with_name(d, "test case"));

            assert!(driver.is_err());
        };
    }

    #[test]
    fn driver() {
        assert_driver_eq!(
            "",
            Driver::Store(StoreDriver {
                initialization: StoreInitialization::Zeroed,
                allocate: StoreAllocationKind::OnDemand,
                format: BackingFileFormat::Automatic,
            })
        );

        assert_driver_eq!(r#"kind = "file""#,);

        assert_driver_eq!(
            r#"
            kind = "file"
            path = "./abc"
            "#,
            Driver::File(FileDriver {
                path: PathBuf::from("./abc"),
                format: BackingFileFormat::Automatic,
            })
        );

        assert_driver_eq!(
            r#"
            kind = "file"
            path = "./abc"
            format = "hex"
            "#,
            Driver::File(FileDriver {
                path: PathBuf::from("./abc"),
                format: BackingFileFormat::Hex,
            })
        );

        assert_driver_eq!("kind = \"c\"",);
    }

    #[test]
    fn pulpino() {
        let driver =
            toml::from_str::<ConfigDocument>(include_str!("../device-configs/pulpino.toml"))
                .map_err(|e| DocumentParseError::TomlError(e))
                .and_then(|d| Document::try_from(d));

        assert!(driver.is_ok(), "{err:?}", err = driver.unwrap_err());
        dbg!(driver.unwrap());
    }
}
