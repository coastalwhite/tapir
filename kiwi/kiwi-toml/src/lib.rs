use std::collections::HashMap;

use serde::Deserialize;

mod bits;
mod bytemask;
pub mod new_raw;
pub mod raw;
pub mod result;
pub mod to_result;

use raw::RawKiwiFile;

use self::bits::Bits;
use self::raw::FormatFieldPart;

#[derive(Deserialize, Debug)]
#[serde(try_from = "RawKiwiFile")]
pub struct KiwiFile {
    name: String,
    description: String,
    formats: Vec<Format>,
    field_variants: Vec<FieldVariant>,
    instructions: Vec<Instruction>,
}

impl KiwiFile {
    pub fn from_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn formats(&self) -> &[Format] {
        &self.formats
    }

    pub fn field_variants(&self) -> &[FieldVariant] {
        &self.field_variants
    }

    fn decode(&self, encoded: u32) -> Option<&Instruction> {
        for instruction in self.instructions.iter().rev() {
            let activated = instruction.encoding.activated.as_u32().unwrap();
            let mask = instruction.encoding.mask.as_u32().unwrap();

            if activated & encoded == mask {
                return Some(instruction);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    name: String,
    encoding: InstructionEncoding,
    asm: InstructionAssembly,
}

impl Instruction {
    pub fn format<'a>(&'_ self, kiwi_file: &'a KiwiFile) -> &'a Format {
        &kiwi_file.formats()[self.encoding.format]
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn encoding(&self) -> &InstructionEncoding {
        &self.encoding
    }

    pub fn assembly(&self) -> &InstructionAssembly {
        &self.asm
    }
}

#[derive(Debug, Clone)]
pub struct InstructionEncoding {
    mask: Bits,
    activated: Bits,

    format: usize,

    predefined: Vec<String>,
    renamings: HashMap<String, String>,
}

impl InstructionEncoding {
    pub fn mask(&self) -> &Bits {
        &self.mask
    }

    pub fn activated(&self) -> &Bits {
        &self.activated
    }

    pub fn predefined(&self) -> &[String] {
        &self.predefined
    }

    pub fn renamings(&self) -> &HashMap<String, String> {
        &self.renamings
    }

    pub fn format(&self) -> usize {
        self.format
    }
}

#[derive(Debug, Clone)]
pub struct InstructionAssembly {
    mnemonic: String,
    fields: Vec<InstructionAssemblyField>,
}

#[derive(Debug, Clone)]
pub struct InstructionAssemblyField {
    pub variant: usize,
    pub arguments: Vec<String>,
}

impl InstructionAssembly {
    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    pub fn fields(&self) -> &[InstructionAssemblyField] {
        &self.fields
    }
}

impl InstructionAssemblyField {
    pub fn variant<'a>(&self, kiwi_file: &'a KiwiFile) -> &'a FieldVariant {
        &kiwi_file.field_variants[self.variant]
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }
}

#[derive(Debug)]
pub struct Format {
    name: String,
    num_bits: u32,
    parts: HashMap<String, Part>,
    added_fields: Vec<(String, Vec<FormatFieldPart>)>,
}

impl Format {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn num_bits(&self) -> u32 {
        self.num_bits
    }

    pub fn parts(&self) -> &HashMap<String, Part> {
        &self.parts
    }

    pub fn added_fields(&self) -> &[(String, Vec<FormatFieldPart>)] {
        &self.added_fields
    }
}

#[derive(Debug)]
pub struct Part {
    pub num_bits: u32,
    pub offset: u32,
}

#[derive(Debug)]
pub struct FieldVariant {
    name: String,
    arg_max_sizes: Vec<u32>,
    default_display: String,
    display: Vec<(String, Option<FieldDisplay>)>,
}

impl FieldVariant {
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn arg_max_sizes(&self) -> &[u32] {
        &self.arg_max_sizes
    }

    pub fn default_display(&self) -> &str {
        &self.default_display
    }

    pub fn display(&self) -> &[(String, Option<FieldDisplay>)] {
        &self.display
    }
}

#[derive(Debug)]
pub enum FieldDisplay {
    Template(FieldTemplate),
    Lut(Vec<String>),
}

#[derive(Debug)]
pub struct FieldTemplate {
    template: String,
    arguments: Vec<(usize, FieldTemplateArgument)>,
}

#[derive(Debug)]
pub struct FieldTemplateArgument;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let file: KiwiFile = toml::from_str(include_str!("../../example.kiwi.toml")).unwrap();
        dbg!(file.field_variants);

        assert!(false);
    }

    // #[test]
    fn decoding() {
        let file: KiwiFile = toml::from_str(include_str!("../../example.kiwi.toml")).unwrap();
        let encoded = 0x00B51533;
        let i = file.decode(encoded).unwrap();
        dbg!(&i);

        for (name, part) in file.formats[i.encoding.format].parts.iter() {
            let v = encoded & (((1u32 << part.num_bits) - 1) << part.offset);
            let v = v >> part.offset;
            println!("{name}: {v:00$b}", part.num_bits as usize);
        }

        assert!(false);
    }
}
