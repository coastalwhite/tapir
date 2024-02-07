use std::collections::HashMap;
use std::collections::HashSet;

use crate::bytemask::BytesMask;
use crate::new_raw;
use crate::new_raw::FieldOrBits;
use crate::result;
use crate::result::Assembly;
use crate::result::AssemblyField;
use crate::result::BitRange;
use crate::result::EnumType;
use crate::result::FieldId;
use crate::result::Instruction;
use crate::result::InstructionField;
use crate::result::InstructionFieldPart;
use crate::result::IntegerType;
use crate::result::Type;
use crate::result::TypeContent;
use crate::result::TypeId;

#[derive(Debug)]
pub enum KiwiFileError {
    InvalidSnakeCase(String),
    UnknownType(String),
    MissingAssembly(String),
    AdditionalAssembly(String),
    UnknownFormat(String),
    ZeroBitField(String),
    UnknownFormatField(String),
    DuplicateFieldName(String),
    FormatFieldsNotByteRound(u32),
    FormatEncodingSizeNotByteRound(u32),
    FormatEncodingTooLarge(u32),
}

type KiwiFileResult<T> = Result<T, KiwiFileError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SnakeCaseIdent<'a>(&'a str);

impl<'a> SnakeCaseIdent<'a> {
    pub fn new(s: &'a str) -> Option<Self> {
        (s.starts_with(|c: char| c.is_ascii_alphabetic())
            && !s.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '_')))
        .then_some(Self(s))
    }

    pub fn take(s: &'a str) -> KiwiFileResult<Self> {
        Self::new(s).ok_or_else(|| KiwiFileError::InvalidSnakeCase(s.to_string()))
    }

    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

pub struct TypesBuilder<'a> {
    lut: HashMap<&'a str, TypeId>,
    types: Vec<Type<'a>>,
}

impl<'a> TypesBuilder<'a> {
    pub fn new() -> Self {
        Self {
            lut: HashMap::new(),
            types: Vec::new(),
        }
    }

    pub fn new_with_builtins() -> Self {
        let mut builder = Self::new();

        use result::BitsDisplay as D;
        use result::BuiltinType::Integer as I;
        builder.push(
            SnakeCaseIdent::new("sint").unwrap(),
            TypeContent::Builtin(I(IntegerType {
                signed: true,
                shift: 0,
                display: D::Decimal,
            })),
        );
        builder.push(
            SnakeCaseIdent::new("uint").unwrap(),
            TypeContent::Builtin(I(IntegerType {
                signed: false,
                shift: 0,
                display: D::Decimal,
            })),
        );
        builder.push(
            SnakeCaseIdent::new("shex").unwrap(),
            TypeContent::Builtin(I(IntegerType {
                signed: true,
                shift: 0,
                display: D::Hexadecimal,
            })),
        );
        builder.push(
            SnakeCaseIdent::new("uhex").unwrap(),
            TypeContent::Builtin(I(IntegerType {
                signed: false,
                shift: 0,
                display: D::Hexadecimal,
            })),
        );

        builder
    }

    pub fn push(&mut self, name: SnakeCaseIdent<'a>, content: TypeContent<'a>) {
        let id = self.types.len();
        self.types.push(Type { name, content });
        let id = TypeId(id);

        self.lut.insert(name.as_str(), id);
    }

    pub fn get_id(&self, name: &str) -> Option<TypeId> {
        self.lut.get(name).cloned()
    }

    pub fn get(&self, name: &str) -> Option<&Type<'a>> {
        let id = self.lut.get(name)?;
        Some(self.types.get(id.0).unwrap())
    }

    pub fn get_id_with_error(&self, name: &str) -> KiwiFileResult<TypeId> {
        self.get_id(name)
            .ok_or_else(|| KiwiFileError::UnknownType(name.to_string()))
    }

    pub fn get_with_error(&self, name: &str) -> KiwiFileResult<&Type<'a>> {
        self.get(name)
            .ok_or_else(|| KiwiFileError::UnknownType(name.to_string()))
    }

    pub fn finalize(self) -> Vec<Type<'a>> {
        self.types
    }
}

pub struct FieldsBuilder<'a> {
    lut: HashMap<&'a str, FieldId>,
    fields: Vec<InstructionField<'a>>,
}

impl<'a> FieldsBuilder<'a> {
    pub fn new() -> Self {
        Self {
            lut: HashMap::new(),
            fields: Vec::new(),
        }
    }

    pub fn push(
        &mut self,
        name: SnakeCaseIdent<'a>,
        parts: Box<[InstructionFieldPart]>,
        type_id: TypeId,
    ) {
        let id = self.fields.len();
        self.fields.push(InstructionField {
            name,
            parts,
            type_id,
        });
        let id = FieldId(id);

        self.lut.insert(name.as_str(), id);
    }

    pub fn get(&self, name: &str) -> Option<&InstructionField<'a>> {
        let id = self.lut.get(name)?;
        Some(self.fields.get(id.0).unwrap())
    }

    pub fn get_with_error(&self, name: &str) -> KiwiFileResult<&InstructionField<'a>> {
        self.get(name)
            .ok_or_else(|| KiwiFileError::UnknownFormatField(name.to_string()))
    }

    pub fn get_id(&self, name: &str) -> Option<FieldId> {
        self.lut.get(name).cloned()
    }

    pub fn get_id_with_error(&self, name: &str) -> KiwiFileResult<FieldId> {
        self.get_id(name)
            .ok_or_else(|| KiwiFileError::UnknownFormatField(name.to_string()))
    }

    pub fn finalize(self) -> Vec<InstructionField<'a>> {
        self.fields
    }
}

#[derive(Debug)]
struct FormatInfo<'a> {
    fields: HashMap<SnakeCaseIdent<'a>, (BitRange, TypeId)>,
    subfields: HashMap<SnakeCaseIdent<'a>, (&'a [FieldOrBits], TypeId)>,
    num_bytes: u8,
}

impl<'a> FormatInfo<'a> {
    pub fn get_core_field_bitrange(&self, name: SnakeCaseIdent<'a>) -> KiwiFileResult<BitRange> {
        if let Some((bit_range, _)) = self.fields.get(&name) {
            return Ok(*bit_range);
        }

        Err(KiwiFileError::UnknownFormatField(name.as_str().to_string()))
    }

    pub fn get_field_parts_and_type(
        &self,
        name: SnakeCaseIdent<'a>,
    ) -> KiwiFileResult<(Box<[InstructionFieldPart]>, TypeId)> {
        if let Some((bit_range, tid)) = self.fields.get(&name) {
            return Ok((
                vec![InstructionFieldPart::Dynamic(*bit_range)].into_boxed_slice(),
                *tid,
            ));
        }

        if let Some((parts, tid)) = self.subfields.get(&name) {
            let mut field_parts = Vec::new();

            for part in parts.iter() {
                match part {
                    FieldOrBits::Bits(bits) => {
                        field_parts.push(InstructionFieldPart::Static(bits.clone()))
                    }
                    FieldOrBits::Field(field) => {
                        let field = SnakeCaseIdent::take(field)?;
                        let bit_range = self.get_core_field_bitrange(field)?;
                        field_parts.push(InstructionFieldPart::Dynamic(bit_range));
                    }
                }
            }

            let field_parts = field_parts.into_boxed_slice();

            return Ok((field_parts, *tid));
        }

        Err(KiwiFileError::UnknownFormatField(name.as_str().to_string()))
    }
}

impl new_raw::File {
    pub fn to_isa(&self) -> KiwiFileResult<result::Isa> {
        let name = SnakeCaseIdent::take(&self.meta.name)?;
        let description = &self.meta.description;

        let mut types_builder = TypesBuilder::new_with_builtins();

        if let Some(types) = &self.meta.types {
            for (name, ty) in types.iter() {
                let name = SnakeCaseIdent::take(name)?;

                if let Some(enumerate) = &ty.enumerate {
                    let variants = enumerate
                        .iter()
                        .map(|e| SnakeCaseIdent::take(e))
                        .collect::<Result<Vec<SnakeCaseIdent>, KiwiFileError>>()?
                        .into_boxed_slice();

                    let content = TypeContent::Enum(EnumType { variants });
                    types_builder.push(name, content);
                } else {
                    unimplemented!()
                }
            }
        }

        let mut formats = HashMap::new();

        for (name, format) in &self.meta.formats {
            let mut fields = HashMap::new();

            let mut offset = 0;
            for part in format.encoding.iter().rev() {
                if part.bits == 0 {
                    return Err(KiwiFileError::ZeroBitField(name.to_string()));
                }

                let ty = part.ty.as_ref().map_or("sint", |s| &s[..]);

                let range = BitRange {
                    msb: offset + part.bits - 1,
                    lsb: offset,
                };
                let type_id = types_builder.get_id_with_error(ty)?;

                let part_name = SnakeCaseIdent::take(&part.name)?;
                fields.insert(part_name, (range, type_id));

                offset += part.bits;
            }

            if offset % 8 != 0 {
                return Err(KiwiFileError::FormatEncodingSizeNotByteRound(offset));
            }

            if offset > 255 {
                return Err(KiwiFileError::FormatEncodingTooLarge(offset));
            }

            let num_bytes = offset as u8 / 8;

            let mut subfields = HashMap::new();

            if let Some(format_subfields) = &format.subfields {
                for (name, parts) in format_subfields {
                    let name = SnakeCaseIdent::take(name)?;

                    if fields.contains_key(&name) {
                        return Err(KiwiFileError::DuplicateFieldName(name.as_str().to_string()));
                    }

                    for part in parts {
                        match part {
                            FieldOrBits::Bits(_) => {}
                            FieldOrBits::Field(f) => {
                                if !fields.contains_key(&SnakeCaseIdent::take(f)?) {
                                    return Err(KiwiFileError::UnknownFormatField(f.to_string()));
                                }
                            }
                        }
                    }

                    // @TODO
                    subfields.insert(name, (&parts[..], types_builder.get_id_with_error("sint")?));
                }
            }

            formats.insert(name, FormatInfo { fields, subfields, num_bytes });
        }

        let mut instructions: Vec<Instruction> = Vec::new();

        for (name, encoding) in self.instructions.encoding.iter() {
            let asm = self
                .instructions
                .asm
                .get(name)
                .ok_or_else(|| KiwiFileError::MissingAssembly(name.to_string()))?;

            let name = SnakeCaseIdent::take(name)?;

            let mut fields_builder = FieldsBuilder::new();

            let format = formats
                .get(&encoding.format)
                .ok_or_else(|| KiwiFileError::UnknownFormat(encoding.format.to_string()))?;

            dbg!(&format);

            let mut static_fields = HashSet::new();
            let mut renamed_fields = HashSet::new();

            let mut encoding_mask = BytesMask::new(format.num_bytes as usize);


            for (format_field_name, field_value) in &encoding.fields {
                let format_field_name = SnakeCaseIdent::take(format_field_name)?;

                match field_value {
                    FieldOrBits::Bits(bits) => {
                        // @TODO: This should not be limited to core fields
                        let bit_range = format.get_core_field_bitrange(format_field_name)?;
                        encoding_mask.set_bits_at(bits, bit_range.lsb);
                        static_fields.insert(format_field_name);
                    },
                    FieldOrBits::Field(rename) => {
                        let (parts, tid) = format.get_field_parts_and_type(format_field_name)?;
                        let rename = SnakeCaseIdent::take(rename)?;
                        fields_builder.push(rename, parts, tid);
                        renamed_fields.insert(format_field_name);
                    }
                }
            }

            for (format_field_name, _) in &format.fields {
                if static_fields.contains(format_field_name) {
                    continue;
                }

                if renamed_fields.contains(format_field_name) {
                    continue;
                }

                let (parts, tid) = format.get_field_parts_and_type(*format_field_name)?;
                fields_builder.push(*format_field_name, parts, tid);
            }

            for (format_field_name, _) in &format.subfields {
                if static_fields.contains(format_field_name) {
                    continue;
                }

                if renamed_fields.contains(format_field_name) {
                    continue;
                }

                let (parts, tid) = format.get_field_parts_and_type(*format_field_name)?;
                fields_builder.push(*format_field_name, parts, tid);
            }


            let mut asm_fields = Vec::new();

            for field_name in &asm.fields {
                let fid = fields_builder.get_id_with_error(field_name)?;
                asm_fields.push(AssemblyField::Field(fid));
            }

            let asm = Assembly {
                mnemonic: &asm.mnemonic,
                fields: asm_fields,
            };

            let fields = fields_builder.finalize();

            instructions.push(Instruction {
                name,
                asm,
                encoding: encoding_mask,
                fields,
            })
        }

        if instructions.len() != self.instructions.asm.len() {
            for name in self.instructions.asm.keys() {
                if instructions
                    .iter()
                    .find(|instruction| instruction.name.as_str() == name)
                    .is_none()
                {
                    return Err(KiwiFileError::AdditionalAssembly(name.to_string()));
                }
            }
        }

        let types = types_builder.finalize();

        Ok(result::Isa {
            name,
            description,
            types,
            instructions,
        })
    }
}
