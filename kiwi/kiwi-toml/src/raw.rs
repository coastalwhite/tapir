use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;

use crate::bits::Bits;
use crate::{FieldDisplay, Format, Instruction, Part};

impl TryFrom<RawKiwiFile> for crate::KiwiFile {
    type Error = String;
    fn try_from(value: RawKiwiFile) -> Result<Self, Self::Error> {
        let mut formats = Vec::new();

        let mut format_lut = HashMap::new();

        for (i, (name, format)) in value.formats.inner.iter().enumerate() {
            let name = name.to_string();
            let mut offset = 0;
            let mut parts = HashMap::new();
            for (k, v) in format.iter().rev() {
                let num_bits = v.num_bits();
                parts.insert(k.clone(), Part { num_bits, offset });
                offset += num_bits;
            }
            let added_fields = value
                .format_fields
                .inner
                .get(&name)
                .map(|fields| fields.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();

            formats.push(Format {
                name: name.clone(),
                num_bits: offset,
                parts,
                added_fields,
            });

            format_lut.insert(name, i);
        }

        let mut field_variants = Vec::new();
        let mut field_lut = HashMap::new();

        for (name, displays) in [
            ("signed-int", &["default"]),
            ("unsigned-int", &["default"]),
            ("signed-hex", &["default"]),
            ("unsigned-hex", &["default"]),
            ("rel-label", &["imm"]),
        ] {
            field_lut.insert(name, field_variants.len());
            field_variants.push(crate::FieldVariant {
                name: name.to_string(),
                arg_max_sizes: vec![0],
                default_display: displays[0].to_string(),
                display: displays
                    .into_iter()
                    .map(|s| (s.to_string(), None))
                    .collect(),
            });
        }

        for (field_name, field) in &value.asm_fields.inner {
            let mut display = Vec::new();

            for (display_name, display_content) in &field.display {
                let content = match display_content {
                    AsmFieldDisplay::Format(template) => {
                        FieldDisplay::Template(crate::FieldTemplate {
                            template: template.clone(),
                            arguments: Vec::new(),
                        })
                    }
                    AsmFieldDisplay::Lut(lut) => FieldDisplay::Lut(lut.clone()),
                };
                display.push((display_name.clone(), Some(content)))
            }

            field_lut.insert(&field_name[..], field_variants.len());
            field_variants.push(crate::FieldVariant {
                name: field_name.clone(),
                arg_max_sizes: vec![field.max_bits],
                default_display: field.default_display.clone(),
                display,
            })
        }

        let mut instructions = Vec::new();

        for (name, encoding) in value.encoding.inner.iter() {
            let mut predefined = Vec::new();
            let mut renamings = HashMap::new();

            let format = &encoding.format;
            let format_idx = format_lut
                .get(format)
                .ok_or(format!("Unknown format: {format}"))?;
            let format = &formats[*format_idx];

            let mut mask = Bits::zeros(format.num_bits);
            let mut activated = Bits::zeros(format.num_bits);

            for (field_name, field) in encoding.fields.iter() {
                let EncodingField::Bits(bits) = field else {
                    let EncodingField::Ident(rename) = field else {
                        continue
                    };

                    renamings.insert(field_name.clone(), rename.clone());

                    continue;
                };

                predefined.push(field_name.clone());

                let part = format
                    .parts
                    .get(field_name)
                    .ok_or(format!("Unknown field name: {field_name}"))?;

                activated.bitor_ones(part.offset, part.num_bits);
                mask.bitor_at(part.offset, bits);
            }

            let assembly = value.asm.inner.get(name).ok_or("Undefined assembly")?;

            let mnemonic = assembly.mnemonic.clone();
            let mut fields = Vec::new();

            for asm_field in &assembly.fields {
                let AsmInstructionField::Str(asm_field) = asm_field else {
                    todo!();
                };

                let (arg, field_name) = asm_field.split_once(':').unwrap();

                let variant = field_lut.get(field_name).expect(&format!("Format {field_name} not defined"));
                let variant = *variant;

                fields.push(crate::InstructionAssemblyField {
                    variant,
                    arguments: vec![arg.to_string()],
                })
            }

            instructions.push(Instruction {
                name: name.clone(),
                encoding: crate::InstructionEncoding {
                    mask,
                    activated,
                    format: *format_idx,
                    predefined,
                    renamings,
                },
                asm: crate::InstructionAssembly { mnemonic, fields },
            });
        }

        instructions.sort_by(|a, b| {
            formats[a.encoding.format]
                .num_bits
                .cmp(&formats[b.encoding.format].num_bits)
                .then_with(|| {
                    a.encoding
                        .activated
                        .count_ones()
                        .cmp(&b.encoding.activated.count_ones())
                })
        });

        Ok(Self {
            name: value.name,
            description: value.description,
            formats,
            field_variants,
            instructions,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct RawKiwiFile {
    name: String,
    description: String,
    formats: Formats,
    #[serde(rename = "format-fields")]
    format_fields: FormatFields,
    #[serde(rename = "asm-fields")]
    asm_fields: AsmFields,
    encoding: Encodings,
    asm: Asm,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Formats {
    inner: HashMap<String, IndexMap<String, FormatFieldInfoSerde>>,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct FormatFields {
    inner: HashMap<String, HashMap<String, Vec<FormatFieldPart>>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FormatFieldPart {
    Bits(Bits),
    Field(String),
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct AsmFields {
    inner: HashMap<String, AsmField>,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Encodings {
    inner: HashMap<String, Encoding>,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Asm {
    inner: HashMap<String, AsmInstruction>,
}

#[derive(Deserialize, Debug)]
pub struct Encoding {
    format: String,
    #[serde(flatten)]
    fields: HashMap<String, EncodingField>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum EncodingField {
    Bits(Bits),
    Ident(String),
    Specify(EncodingFieldSpecify),
}

#[derive(Deserialize, Debug)]
pub struct EncodingFieldSpecify {
    name: String,
    undefined: Option<Vec<Bits>>,
    #[serde(flatten)]
    excludes: HashMap<String, Vec<Bits>>,
}

#[derive(Deserialize, Debug)]
pub struct AsmInstruction {
    mnemonic: String,
    fields: Vec<AsmInstructionField>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AsmInstructionField {
    Str(String),
    Structure(AsmInstructionFieldStruct),
}

#[derive(Deserialize, Debug)]
pub struct AsmInstructionFieldStruct {
    variant: String,
    arguments: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct AsmField {
    max_bits: u32,
    default_display: String,
    display: HashMap<String, AsmFieldDisplay>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AsmFieldDisplay {
    Format(String),
    Lut(Vec<String>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum FormatFieldInfoSerde {
    Bits(u32),
    Full(FormatFieldInfo),
}

impl FormatFieldInfoSerde {
    pub fn num_bits(&self) -> u32 {
        match self {
            FormatFieldInfoSerde::Bits(n) => *n,
            FormatFieldInfoSerde::Full(i) => i.bits,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct FormatFieldInfo {
    bits: u32,
    #[serde(rename = "match")]
    match_group: Option<u32>,
}

impl From<FormatFieldInfoSerde> for FormatFieldInfo {
    fn from(value: FormatFieldInfoSerde) -> Self {
        match value {
            FormatFieldInfoSerde::Bits(bits) => Self {
                bits,
                match_group: None,
            },
            FormatFieldInfoSerde::Full(info) => info,
        }
    }
}
