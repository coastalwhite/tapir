use std::borrow::Cow;

use crate::bits::Bits;
use crate::bytemask::BytesMask;
use crate::to_result::SnakeCaseIdent;

#[derive(Debug)]
pub struct Isa<'a> {
    pub(crate) name: SnakeCaseIdent<'a>,
    pub(crate) description: &'a str,
    pub(crate) types: Vec<Type<'a>>,
    pub(crate) instructions: Vec<Instruction<'a>>,
}

impl<'a> Isa<'a> {
    pub fn name(&self) -> SnakeCaseIdent<'a> {
        self.name
    }
    
    pub fn description(&self) -> &str {
        self.description
    }

    pub fn types(&self) -> &[Type<'a>] {
        &self.types
    }

    pub fn instructions(&self) -> &[Instruction<'a>] {
        &self.instructions
    }
}

#[derive(Debug)]
pub struct Type<'a> {
    pub(crate) name: SnakeCaseIdent<'a>,
    pub(crate) content: TypeContent<'a>,
}

#[derive(Debug)]
pub enum TypeContent<'a> {
    Builtin(BuiltinType),
    Enum(EnumType<'a>),
}

#[derive(Debug)]
pub enum BuiltinType {
    Integer(IntegerType),
}

#[derive(Debug)]
pub struct EnumType<'a> {
    pub(crate) variants: Box<[SnakeCaseIdent<'a>]>,
}

impl<'a> Type<'a> {
    pub fn name(&self) -> SnakeCaseIdent<'a> {
        self.name
    }

    pub fn content(&self) -> &TypeContent<'a> {
        &self.content
    }
}

impl<'a> EnumType<'a> {
    pub fn max_bits(&self) -> u32 {
        debug_assert!(self.variants.len().is_power_of_two());
        self.variants.len().ilog2()
    }

    pub fn variants(&self) -> &[SnakeCaseIdent<'a>] {
        &self.variants
    }
}


#[derive(Debug)]
pub struct IntegerType {
    pub(crate) signed: bool,
    pub(crate) shift: u32,
    pub(crate) display: BitsDisplay,
}

#[derive(Debug)]
pub enum BitsDisplay {
    Decimal,
    Binary,
    Octal,
    Hexadecimal,
}

#[derive(Debug)]
pub struct Instruction<'a> {
    pub(crate) name: SnakeCaseIdent<'a>,
    pub(crate) asm: Assembly<'a>,
    pub(crate) encoding: BytesMask,
    pub(crate) fields: Vec<InstructionField<'a>>,
}

#[derive(Debug)]
pub struct Assembly<'a> {
    pub(crate) mnemonic: &'a str,
    pub(crate) fields: Vec<AssemblyField<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldId(pub(crate) usize);
#[derive(Debug, Clone, Copy)]
pub struct TypeId(pub(crate) usize);

#[derive(Debug)]
pub enum AssemblyField<'a> {
    Field(FieldId),
    FormatString(AssemblyFieldFormatString<'a>),
}

#[derive(Debug)]
pub struct AssemblyFieldFormatString<'a> {
    pub(crate) base: Cow<'a, str>,
    pub(crate) fields: Vec<(usize, FieldId)>,
}

#[derive(Debug)]
pub struct InstructionField<'a> {
    pub(crate) name: SnakeCaseIdent<'a>,
    pub(crate) parts: Box<[InstructionFieldPart]>,
    pub(crate) type_id: TypeId,
}

#[derive(Debug, Clone, Copy)]
pub struct BitRange {
    pub(crate) msb: u32,
    pub(crate) lsb: u32,
}

#[derive(Debug)]
pub enum InstructionFieldPart {
    Static(Bits),
    Dynamic(BitRange),
}