use std::fmt::Display;
use std::io;

mod fence_order;
mod register;

struct UpperHex(u32);

impl std::fmt::Debug for UpperHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("0x")?;
        <u32 as std::fmt::UpperHex>::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CsrIndex(pub u16);
#[derive(Debug, Clone, Copy)]
pub struct FenceMode(pub u8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RoundingMode {
    /// Round to Nearest, Ties to Even,
    #[default]
    TiesToEven = 0b000,

    /// Round to Zero
    ToZero = 0b001,
    /// Round Down (towards -Infinity)
    Down = 0b010,
    /// Round Up (towards Infinity)
    Up = 0b011,
    /// Round to Nearest, Ties to Max Magnitude
    TiesToMaxMagnitude = 0b100,

    Reserved101,
    Reserved110,

    Dynamic,
}

impl std::fmt::Display for RoundingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::TiesToEven => "rne",
            Self::ToZero => "rtz",
            Self::Down => "rdn",
            Self::Up => "rup",
            Self::TiesToMaxMagnitude => "rmm",
            Self::Reserved101 => "invalid",
            Self::Reserved110 => "invalid",
            Self::Dynamic => "dyn",
        })
    }
}

impl RoundingMode {
    pub fn take_masked(bits: u32) -> Self {
        match bits & 0b111 {
            0b000 => Self::TiesToEven,
            0b001 => Self::ToZero,
            0b010 => Self::Down,
            0b011 => Self::Up,
            0b100 => Self::TiesToMaxMagnitude,
            0b101 => Self::Reserved101,
            0b110 => Self::Reserved110,
            0b111 => Self::Dynamic,
            _ => unreachable!(),
        }
    }
}

impl Display for CsrIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub use fence_order::FenceOrder;
pub use register::{FRegIdent, XRegIdent};

#[inline(always)]
const fn funct3(bits: u32) -> u32 {
    (bits >> 12) & 0b111
}

#[inline(always)]
const fn funct7(bits: u32) -> u32 {
    (bits >> 25) & 0b111_1111
}

macro_rules! format_enable {
    (
        r
        $(, funct7 = $funct7:literal)?
        $(, funct2 = $funct2:literal)?
        $(, rs2    = $rs2:literal   )?
        $(, rs1    = $rs1:literal   )?
        $(, funct3 = $funct3:literal)?
        $(, rd     = $rd:literal    )?
    ) => {
        0u32
            $( | { $funct7; 0b111_1111u32 << 25 } )?
            $( | { $funct2;       0b11u32 << 25 } )?
            $( | { $rs2   ;   0b1_1111u32 << 20 } )?
            $( | { $rs1   ;   0b1_1111u32 << 15 } )?
            $( | { $funct3;      0b111u32 << 12 } )?
            $( | { $rd    ;   0b1_1111u32 << 07 } )?
    };
    (
        s
        $(, imm11_0 = $imm11_0:literal )?
        $(, rs2     = $rs2:literal    )?
        $(, rs1     = $rs1:literal    )?
        $(, funct3  = $funct3:literal )?
    ) => {
        0u32
            $( | compile_error!($imm11_0); )?
            $( | { $rs2   ; 0b1_1111u32 << 20 } )?
            $( | { $rs1   ; 0b1_1111u32 << 15 } )?
            $( | { $funct3;    0b111u32 << 12 } )?
    };
    (
        i
        $(, funct7_6_3 = $funct7_6_3:literal)?
        $(, funct7     = $funct7:literal    )?
        $(, imm11_0    = $imm11_0:literal   )?
        $(, rs1        = $rs1:literal       )?
        $(, funct3     = $funct3:literal    )?
        $(, rd         = $rd:literal        )?
    ) => {
        0u32
            $( | { $funct7_6_3 ; 0b1111u32     << 28 } )?
            $( | { $funct7     ; 0b111_1111u32 << 25 } )?
            $( | { $imm11_0    ; 0b1_1111u32   << 20 } )?
            $( | { $rs1        ; 0b1_1111u32   << 15 } )?
            $( | { $funct3     ;    0b111u32   << 12 } )?
            $( | { $rd         ; 0b1_1111u32   << 07 } )?
    };
    (
        b
        $(, imm12_1 = $imm12_1:literal )?
        $(, rs2     = $rs2:literal    )?
        $(, rs1     = $rs1:literal    )?
        $(, funct3  = $funct3:literal )?
    ) => {
        0u32
            $( | compile_error!($imm12_1); )?
            $( | { $rs2   ; 0b1_1111u32 << 20 } )?
            $( | { $rs1   ; 0b1_1111u32 << 15 } )?
            $( | { $funct3;    0b111u32 << 12 } )?
    };
    (
        u
        $(, imm31_12 = $imm31_12:literal)?
        $(, rd       = $rd:literal    )?
    ) => {
        0u32
            $( | { $imm31_12; 0xFFFF_Fu32 << 12 } )?
            $( | { $rd      ; 0b1111_1u32 << 07 } )?
    };
    (
        j
        $(, imm20_1 = $imm20_1:literal )?
        $(, rd      = $rd:literal      )?
    ) => {
        0u32
            $( | { $imm20_1; 0xFFFF_Fu32 << 12 } )?
            $( | { $rd     ; 0b1111_1u32 << 07 } )?
    };
}

macro_rules! format_mask {
    (
        r
        $(, funct7 = $funct7:literal)?
        $(, funct2 = $funct2:literal)?
        $(, rs2    = $rs2:literal   )?
        $(, rs1    = $rs1:literal   )?
        $(, funct3 = $funct3:literal)?
        $(, rd     = $rd:literal    )?
    ) => {
        0u32
            $( | { const FUNCT7: u32 = $funct7; FUNCT7 << 25 } )?
            $( | { const FUNCT2: u32 = $funct2; FUNCT2 << 25 } )?
            $( | { const RS2:    u32 = $rs2   ; RS2    << 20 } )?
            $( | { const RS1:    u32 = $rs1   ; RS1    << 15 } )?
            $( | { const FUNCT3: u32 = $funct3; FUNCT3 << 12 } )?
            $( | { const RD:     u32 = $rd    ; RD     << 07 } )?
    };
    (
        i
        $(, funct7_6_3 = $funct7_6_3:literal)?
        $(, funct7     = $funct7:literal    )?
        $(, imm11_0    = $imm11_0:literal   )?
        $(, rs1        = $rs1:literal       )?
        $(, funct3     = $funct3:literal    )?
        $(, rd         = $rd:literal        )?
    ) => {
        0u32
            $( | { const FUNCT7_6_3: u32 = $funct7_6_3; FUNCT7_6_3 << 28 } )?
            $( | { const FUNCT7:     u32 = $funct7    ; FUNCT7     << 25 } )?
            $( | { const IMM11_0:    u32 = $imm11_0   ; IMM11_0    << 20 } )?
            $( | { const RS1:        u32 = $rs1       ; RS1        << 15 } )?
            $( | { const FUNCT3:     u32 = $funct3    ; FUNCT3     << 12 } )?
            $( | { const RD:         u32 = $rd        ; RD         << 07 } )?
    };
    (
        s
        $(, imm11_0 = $imm11_0:literal )?
        $(, rs2     = $rs2:literal    )?
        $(, rs1     = $rs1:literal    )?
        $(, funct3  = $funct3:literal )?
    ) => {
        0u32
            $( | compile_error!($imm11_0); )?
            $( | { const RS2:    u32 = $rs2   ; RS2    << 20 } )?
            $( | { const RS1:    u32 = $rs1   ; RS2    << 15 } )?
            $( | { const FUNCT3: u32 = $funct3; FUNCT3 << 12 } )?
    };
    (
        b
        $(, imm12_1 = $imm12_1:literal )?
        $(, rs2     = $rs2:literal    )?
        $(, rs1     = $rs1:literal    )?
        $(, funct3  = $funct3:literal )?
    ) => {
        0u32
            $( | compile_error!($imm12_1); )?
            $( | { const RS2:    u32 = $rs2   ; RS2    << 20 } )?
            $( | { const RS1:    u32 = $rs1   ; RS2    << 15 } )?
            $( | { const FUNCT3: u32 = $funct3; FUNCT3 << 12 } )?
    };
    (
        u
        $(, imm31_12 = $imm31_12:literal)?
        $(, rd       = $rd:literal    )?
    ) => {
        0u32
            $( | { const IMM31_12: u32 = $imm31_12; IMM31_12 << 12 } )?
            $( | { const RD:       u32 = $rd      ; RD       << 07 } )?
    };
    (
        j
        $(, imm20_1 = $imm20_1:literal )?
        $(, rd      = $rd:literal      )?
    ) => {
        0u32
            $( | compile_error!($imm20_1); )?
            $( | { const RD:       u32 = $rd      ; RD       << 07 } )?
    };
}

#[rustfmt::skip]
macro_rules! format_num_bytes {
    (r) => { 4 };
    (i) => { 4 };
    (s) => { 4 };
    (b) => { 4 };
    (u) => { 4 };
    (j) => { 4 };
}

#[rustfmt::skip]
macro_rules! field_type {
    (rd: freg) => { FRegIdent };
    (rs1: freg) => { FRegIdent };
    (rs2: freg) => { FRegIdent };
    (rs3: freg) => { FRegIdent };
    (rm) => { RoundingMode };
    (rd:  xreg) => { XRegIdent };
    (rs1: xreg) => { XRegIdent };
    (rs2: xreg) => { XRegIdent };
    (shamt) => { u8 };
    (csr) => { CsrIndex };
    (uimm: csr) => { u8 };
    (imm: itype_unsigned) => { u16 };
    (imm: itype_signed) => { i16 };
    (imm: stype) => { i16 };
    (imm: btype) => { i16 };
    (imm: jtype) => { i32 };
    (imm: utype) => { u32 };
    (fm) => { FenceMode };
    (pred) => { FenceOrder };
    (succ) => { FenceOrder };
}

#[rustfmt::skip]
macro_rules! field_encode {
    ($v:ident, rd: freg) =>            { ($v as u32) << 7 };
    ($v:ident, rs1: freg) =>           { ($v as u32) << 15 };
    ($v:ident, rs2: freg) =>           { ($v as u32) << 20 };
    ($v:ident, rs3: freg) =>           { ($v as u32) << 27 };
    ($v:ident, rm) =>                  { ($v as u32) << 12 };
    ($v:ident, rd: xreg) =>            { ($v as u32) << 7 };
    ($v:ident, rs1: xreg) =>           { ($v as u32) << 15 };
    ($v:ident, rs2: xreg) =>           { ($v as u32) << 20 };
    ($v:ident, shamt) =>               { ($v as u32) << 20 };
    ($v:ident, csr) =>                 { ($v.0 as u32) << 20 };
    ($v:ident, uimm: csr) =>           { ($v as u32) << 15 };
    ($v:ident, imm: itype_unsigned) => { ($v as u32) << 20 };
    ($v:ident, imm: itype_signed) =>   { (($v as u32) & 0xFFF) << 20 };
    ($v:ident, imm: stype) =>          {{
        let imm4_0  = ($v as u32) & 0x01F;
        let imm11_5 = ($v as u32) & 0xFE0;

        (imm4_0 << 7) | (imm11_5 << (25 - 5))
    }};
    ($v:ident, imm: btype) => {{
        let imm4_1  = ($v as u32) & 0x001E;
        let imm10_5 = ($v as u32) & 0x07E0;
        let imm11   = ($v as u32) & 0x0800;
        let imm12   = ($v as u32) & 0x1000;

        (imm4_1  << ( 8 -  1)) | 
        (imm10_5 << (25 -  5)) |
        (imm11   >> (11 -  7)) | // @NOTE: Intentional right shift
        (imm12   << (31 - 12))
    }};
    ($v:ident, imm: jtype) => {{
        let imm10_1  = ($v as u32) & 0x0000_07FE;
        let imm11    = ($v as u32) & 0x0000_0800;
        let imm19_12 = ($v as u32) & 0x000F_F000;
        let imm20    = ($v as u32) & 0x0010_0000;

        (imm10_1  << (21 -  1)) | 
        (imm11    << (20 - 11)) |
        (imm19_12 << (12 - 12)) |
        (imm20    << (31 - 20))
    }};
    ($v:ident, imm: utype) => { $v & 0xFFFF_F000 };
    ($v:ident, fm) => { ($v.0 as u32) << 28 };
    ($v:ident, pred) => { ($v.encode() as u32) << 24 };
    ($v:ident, succ) => { ($v.encode() as u32) << 20 };
}

#[rustfmt::skip]
macro_rules! field_decode {
    ($bs:expr, rd: freg)            => { FRegIdent::take_masked($bs >> 7) };
    ($bs:expr, rs1: freg)           => { FRegIdent::take_masked($bs >> 15) };
    ($bs:expr, rs2: freg)           => { FRegIdent::take_masked($bs >> 20) };
    ($bs:expr, rs3: freg)           => { FRegIdent::take_masked($bs >> 27) };
    ($bs:expr, rm)                  => { RoundingMode::take_masked($bs >> 12) };
    ($bs:expr, rd: xreg)            => { XRegIdent::take_masked($bs >> 7) };
    ($bs:expr, rs1: xreg)           => { XRegIdent::take_masked($bs >> 15) };
    ($bs:expr, rs2: xreg)           => { XRegIdent::take_masked($bs >> 20) };
    ($bs:expr, shamt)               => { (($bs >> 20) & 0x1F) as u8 };
    ($bs:expr, csr)                 => { CsrIndex(($bs >> 20) as u16) };
    ($bs:expr, uimm: csr)           => { (($bs >> 15) & 0x1F) as u8 };
    ($bs:expr, imm: itype_unsigned) => { ($bs >> 20) as u16 };
    ($bs:expr, imm: itype_signed)   => { ((($bs & 0xFFF0_0000) as i32) >> 20) as i16 };
    ($bs:expr, imm: stype)          => { ((($bs & 0xFE00_0000) as i32) >> 20) as i16 | (($bs >> 7) & 0x1F) as i16 };
    ($bs:expr, imm: btype)          => {{
            // Not straight forward because we need to sign extend

            let imm = $bs as i32;
            let imm = imm >> (31 - 12);
            let imm = imm & !0xFFF;

            let imm11   = ($bs >>  7) & 0x01;
            let imm10_5 = ($bs >> 25) & 0x3F;
            let imm4_1  = ($bs >>  8) & 0x0F;

            let imm = imm | (imm11 << 11) as i32 | (imm10_5 << 5) as i32 | (imm4_1 << 1) as i32;

            imm as i16
    }};
    ($bs:expr, imm: jtype) => {{
        // Not straight forward because we need to sign extend

        let imm = $bs as i32;
        let imm = imm >> (31 - 20);
        let imm = imm & !0xF_FFFF;

        let imm19_12 =  $bs & 0xF_F000;
        let imm11    = ($bs >> 20) & 1;
        let imm10_1  = ($bs >> 21) & 0x3FF;

        let imm = imm | imm19_12 as i32 | (imm11 << 11) as i32 | (imm10_1 << 1) as i32;

        imm
    }};
    ($bs:expr, imm: utype) => { $bs & 0xFFFF_F000 };
    ($bs:expr, fm) => { FenceMode((($bs >> 28) & 0xF) as u8) };
    ($bs:expr, pred) => { FenceOrder::take_masked($bs >> 24) };
    ($bs:expr, succ) => { FenceOrder::take_masked($bs >> 20) };
}

macro_rules! instructions {
    (
        $(
            $name:ident
            (
                $mnemonic:literal,
                $opcode:literal,
                $format:ident $(, $field:ident = $value:literal)* $(,)?
            )
            (
                $($method_ident:ident$(: $method_extra:ident)?),* $(,)?
            )
        ),+
        $(,)?
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum InstructionVariant {
            $($name,)+
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum Instruction {
            $($name($name),)+
        }

        impl InstructionVariant {
            pub const NUM_INSTRUCTIONS: usize = 0 $( + { $mnemonic; 1 } )+;
            const MNEMONIC_LUT: [&'static str; Self::NUM_INSTRUCTIONS] = [
                $($mnemonic,)+
            ];

            pub const fn mnemonic(self) -> &'static str {
                Self::MNEMONIC_LUT[self as u16 as usize]
            }
        }

        $(
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(u32);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    $(
                    .field(stringify!($method_ident), &self.$method_ident())
                    )*
                    .field("encoding", &UpperHex(self.0))
                    .finish()
            }
        }

        impl From<$name> for Instruction {
            #[inline(always)]
            fn from(i: $name) -> Self {
                Self::$name(i)
            }
        }

        impl $name {
            pub const NUM_BYTES: usize = format_num_bytes!($format);
            pub const MNEMONIC: &'static str = $mnemonic;

            const ENABLE: u32 = format_enable!($format$(, $field = $value)*) | $opcode;
            const MASK: u32 = format_mask!($format$(, $field = $value)*) | $opcode;

            #[inline]
            pub fn new($($method_ident: field_type!($method_ident$(: $method_extra)?)),*) -> Self {
                let mut bits = 0;

                $(
                bits |= field_encode!($method_ident, $method_ident$(: $method_extra)?);
                )*

                debug_assert!(bits & Self::ENABLE == 0);

                bits |= Self::MASK;

                Self(bits)
            }

            #[inline(always)]
            pub fn matches(bits: u32) -> bool {
                bits & Self::ENABLE == Self::MASK
            }

            #[inline]
            pub fn take(bits: u32) -> Option<Self> {
                Self::matches(bits).then_some(Self(bits))
            }

            #[inline(always)]
            pub fn take_unchecked(bits: u32) -> Self {
                debug_assert!(Self::matches(bits), "Invalid take for {} (0x{:08x})", Self::MNEMONIC, bits);
                Self(bits)
            }

            #[inline(always)]
            pub fn encode_as_u32(self) -> u32 {
                self.0
            }

            #[inline]
            pub fn encode(self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
                writer.write_all(&self.0.to_le_bytes()[..Self::NUM_BYTES])
            }

            $(
            #[inline(always)]
            pub fn $method_ident(self) -> field_type!($method_ident$(: $method_extra)?) {
                field_decode!(self.0, $method_ident$(: $method_extra)?)
            }
            )*
        }
        )+

        impl InstructionVariant {
            pub fn into_instruction(self, bits: u32) -> Option<Instruction> {
                match self {
                    $(
                    Self::$name => {
                        let encoding = $name::take(bits)?;
                        Some(Instruction::$name(encoding))
                    },
                    )+
                }
            }

            pub fn into_instruction_unchecked(self, bits: u32) -> Instruction {
                match self {
                    $(
                    Self::$name => {
                        let encoding = $name::take_unchecked(bits);
                        Instruction::$name(encoding)
                    },
                    )+
                }
            }
        }

        impl std::fmt::Display for Instruction {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                    Self::$name(i) => i.fmt(f),
                    )+
                }
            }
        }

        impl Instruction {
            #[inline(always)]
            pub fn encode_as_u32(self) -> u32 {
                match self {
                    $(
                    Self::$name(args) => args.encode_as_u32(),
                    )+
                }
            }

            #[inline]
            pub fn encode(self, writer: &mut impl io::Write) -> io::Result<()> {
                let bits = self.encode_as_u32();

                // Use 2 bytes for compressed instructions
                let num_bytes = 2 << usize::from(bits & 0b11 == 0b11);

                writer.write_all(&bits.to_le_bytes()[..num_bytes])
            }
        }
    };
}

fn decode_compressed(_bits: u16) -> Option<InstructionVariant> {
    None
}

impl Instruction {
    pub fn decode(reader: &mut impl io::Read) -> io::Result<Option<Instruction>> {
        let mut fst = 0;
        reader.read_exact(std::slice::from_mut(&mut fst))?;

        // Compressed instruction
        if fst & 0b11 != 0b11 {
            let mut snd = 0;
            reader.read_exact(std::slice::from_mut(&mut snd))?;

            let bits = u16::from_le_bytes([fst, snd]);
            let Some(variant) = decode_compressed(bits) else {
                return Ok(None);
            };

            return Ok(Some(variant.into_instruction_unchecked(bits as u32)));
        }

        let mut bytes = [0u8; 3];
        reader.read_exact(&mut bytes)?;

        let bits = u32::from_le_bytes([fst, bytes[0], bytes[1], bytes[2]]);

        #[rustfmt::skip]
        static DECODE_LUT: [fn(u32) -> Option<InstructionVariant>; 32] = [
            decode_load,     // 00 - 000
            decode_load_fp,    // 00 - 001
            opcode_00010,    // 00 - 010
            decode_misc_mem, // 00 - 011
            decode_op_imm,   // 00 - 100
            decode_auipc,    // 00 - 101
            opcode_00110,    // 00 - 110
            opcode_00111,    // 00 - 111
            decode_store,    // 01 - 000
            decode_store_fp,    // 01 - 001
            opcode_01010,    // 01 - 010
            opcode_01011,    // 01 - 011
            decode_op,       // 01 - 100
            decode_lui,      // 01 - 101
            opcode_01110,    // 01 - 110
            opcode_01111,    // 01 - 111
            decode_madd,    // 10 - 000
            decode_msub,    // 10 - 001
            decode_nmsub,    // 10 - 010
            decode_nmadd,    // 10 - 011
            decode_op_fp,    // 10 - 100
            opcode_10101,    // 10 - 101
            opcode_10110,    // 10 - 110
            opcode_10111,    // 10 - 111
            decode_branch,   // 11 - 000
            decode_jalr,     // 11 - 001
            opcode_11010,    // 11 - 010
            decode_jal,      // 11 - 011
            decode_system,   // 11 - 100
            opcode_11101,    // 11 - 101
            opcode_11110,    // 11 - 110
            opcode_11111,    // 11 - 111
        ];
        let lut_offset = (fst >> 2) & 0b11111;
        let decode_fn = DECODE_LUT[lut_offset as usize];

        let Some(variant) = decode_fn(bits) else {
            return Ok(None);
        };

        let instruction = variant.into_instruction_unchecked(bits);

        Ok(Some(instruction))
    }
}

fn decode_load(bits: u32) -> Option<InstructionVariant> {
    use InstructionVariant as V;

    let funct3 = funct3(bits);

    #[rustfmt::skip]
    static LUT: [Option<InstructionVariant>; 8] = [
        Some(V::Lb),  Some(V::Lh),  Some(V::Lw), None, // 000
        Some(V::Lbu), Some(V::Lhu), None,        None, // 100
    ];

    LUT[funct3 as usize]
}

fn decode_load_fp(bits: u32) -> Option<InstructionVariant> {
    let funct3 = funct3(bits);

    match funct3 {
        0b010 => Some(InstructionVariant::Flw),
        _ => None,
    }
}
fn opcode_00010(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn decode_misc_mem(bits: u32) -> Option<InstructionVariant> {
    let funct3 = funct3(bits);

    match funct3 {
        0b000 => Some(InstructionVariant::Fence),
        0b001 => Some(InstructionVariant::FenceI),
        _ => None,
    }
}
fn decode_op_imm(bits: u32) -> Option<InstructionVariant> {
    use InstructionVariant as V;

    let funct3 = funct3(bits);
    let funct7 = funct7(bits);

    #[rustfmt::skip]
    static LUT: [InstructionVariant; 8] = [
        V::Addi, V::Slli, V::Slti, V::Sltiu, // 000
        V::Xori, V::Srli, V::Ori,  V::Andi,  // 100
    ];

    let instr = LUT[funct3 as usize];

    if matches!(instr, V::Slli) && funct7 != 0b000_0000 {
        return None;
    } else if matches!(instr, V::Srli) {
        if funct7 == 0b000_0000 {
            return Some(instr);
        } else if funct7 == 0b010_0000 {
            return Some(V::Srai);
        } else {
            return None;
        }
    }

    Some(LUT[funct3 as usize])
}
fn decode_auipc(_: u32) -> Option<InstructionVariant> {
    Some(InstructionVariant::Auipc)
}
fn opcode_00110(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn opcode_00111(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn decode_store(bits: u32) -> Option<InstructionVariant> {
    use InstructionVariant as V;

    let funct3 = funct3(bits);

    #[rustfmt::skip]
    static LUT: [Option<InstructionVariant>; 8] = [
        Some(V::Sb),  Some(V::Sh),  Some(V::Sw), None, // 000
        None,         None,         None,        None, // 100
    ];

    LUT[funct3 as usize]
}
fn decode_store_fp(bits: u32) -> Option<InstructionVariant> {
    let funct3 = funct3(bits);

    match funct3 {
        0b010 => Some(InstructionVariant::Fsw),
        _ => None,
    }
}
fn opcode_01010(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn opcode_01011(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn decode_op(bits: u32) -> Option<InstructionVariant> {
    use InstructionVariant as V;

    let funct3 = funct3(bits);
    let funct7 = funct7(bits);

    match funct7 {
        0b000_0000 => {
            #[rustfmt::skip]
            static LUT: [InstructionVariant; 8] = [
                V::Add, V::Sll, V::Slt, V::Sltu, // 000
                V::Xor, V::Srl, V::Or,  V::And,  // 100
            ];
            Some(LUT[funct3 as usize])
        }
        0b010_0000 => {
            #[rustfmt::skip]
            static LUT: [Option<InstructionVariant>; 8] = [
                Some(V::Sub), None,         None, None, // 000
                None,         Some(V::Sra), None, None, // 100
            ];
            LUT[funct3 as usize]
        }
        _ => None,
    }
}
fn decode_lui(_: u32) -> Option<InstructionVariant> {
    Some(InstructionVariant::Lui)
}
fn opcode_01110(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn opcode_01111(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn decode_madd(bits: u32) -> Option<InstructionVariant> {
    if (bits >> 25) & 0b11 == 0b00 {
        return Some(InstructionVariant::FmaddS);
    }

    None
}
fn decode_msub(bits: u32) -> Option<InstructionVariant> {
    if (bits >> 25) & 0b11 == 0b00 {
        return Some(InstructionVariant::FmsubS);
    }

    None
}
fn decode_nmsub(bits: u32) -> Option<InstructionVariant> {
    if (bits >> 25) & 0b11 == 0b00 {
        return Some(InstructionVariant::FnmsubS);
    }

    None
}
fn decode_nmadd(bits: u32) -> Option<InstructionVariant> {
    if (bits >> 25) & 0b11 == 0b00 {
        return Some(InstructionVariant::FnmaddS);
    }

    None
}
fn decode_op_fp(bits: u32) -> Option<InstructionVariant> {
    let funct7 = funct7(bits);
    let funct3 = funct3(bits);
    let rs2 = (bits >> 20) & 0b11111;

    match (funct7, rs2, funct3) {
        (0b000_0000, _, _) => Some(InstructionVariant::FaddS),
        (0b000_0100, _, _) => Some(InstructionVariant::FsubS),
        (0b000_1000, _, _) => Some(InstructionVariant::FmulS),
        (0b000_1100, _, _) => Some(InstructionVariant::FdivS),
        (0b010_1100, 0b00000, _) => Some(InstructionVariant::FsqrtS),
        (0b001_0000, _, 0b000) => Some(InstructionVariant::FsgnjS),
        (0b001_0000, _, 0b001) => Some(InstructionVariant::FsgnjnS),
        (0b001_0000, _, 0b010) => Some(InstructionVariant::FsgnjxS),
        (0b001_0100, _, 0b000) => Some(InstructionVariant::FminS),
        (0b001_0100, _, 0b001) => Some(InstructionVariant::FmaxS),
        (0b110_0000, 0b00000, _) => Some(InstructionVariant::FcvtWS),
        (0b110_0000, 0b00001, _) => Some(InstructionVariant::FcvtWuS),
        (0b111_0000, 0b00000, 0b000) => Some(InstructionVariant::FmvXW),
        (0b101_0000, _, 0b010) => Some(InstructionVariant::FeqS),
        (0b101_0000, _, 0b001) => Some(InstructionVariant::FltS),
        (0b101_0000, _, 0b000) => Some(InstructionVariant::FleS),
        (0b111_0000, 0b00000, 0b001) => Some(InstructionVariant::FclassS),
        (0b110_1000, 0b00000, _) => Some(InstructionVariant::FcvtSW),
        (0b110_1000, 0b00001, _) => Some(InstructionVariant::FcvtSWu),
        (0b111_1000, 0b00000, 0b000) => Some(InstructionVariant::FmvWX),
        _ => None,
    }
}
fn opcode_10101(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn opcode_10110(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn opcode_10111(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn decode_branch(bits: u32) -> Option<InstructionVariant> {
    use InstructionVariant as V;

    let funct3 = funct3(bits);

    #[rustfmt::skip]
    static LUT: [Option<InstructionVariant>; 8] = [
        Some(V::Beq), Some(V::Bne), None,          None,          // 000
        Some(V::Blt), Some(V::Bge), Some(V::Bltu), Some(V::Bgeu), // 000
    ];

    LUT[funct3 as usize]
}
fn decode_jalr(bits: u32) -> Option<InstructionVariant> {
    if (bits >> 12) & 0b111 != 0 {
        return None;
    }

    Some(InstructionVariant::Jalr)
}
fn opcode_11010(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn decode_jal(_: u32) -> Option<InstructionVariant> {
    Some(InstructionVariant::Jal)
}
fn decode_system(bits: u32) -> Option<InstructionVariant> {
    let funct3 = funct3(bits);

    match funct3 {
        0b000 if Ecall::matches(bits) => Some(InstructionVariant::Ecall),
        0b000 if Ebreak::matches(bits) => Some(InstructionVariant::Ebreak),
        0b000 if MRet::matches(bits) => Some(InstructionVariant::MRet),
        0b001 => Some(InstructionVariant::Csrrw),
        0b010 => Some(InstructionVariant::Csrrs),
        0b011 => Some(InstructionVariant::Csrrc),
        0b101 => Some(InstructionVariant::Csrrwi),
        0b110 => Some(InstructionVariant::Csrrsi),
        0b111 => Some(InstructionVariant::Csrrci),
        _ => None,
    }
}
fn opcode_11101(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn opcode_11110(_bits: u32) -> Option<InstructionVariant> {
    None
}
fn opcode_11111(_bits: u32) -> Option<InstructionVariant> {
    None
}

instructions! {
    Lui   ("lui",   0b011_0111, u                                     ) (rd: xreg, imm: utype),
    Auipc ("auipc", 0b001_0111, u                                     ) (rd: xreg, imm: utype),

    Jal   ("jal",   0b110_1111, j                                     ) (rd: xreg, imm: jtype),
    Jalr  ("jalr",  0b110_0111, i, funct3 = 0b000                     ) (rd: xreg, rs1: xreg, imm: itype_signed),

    Beq   ("beq",   0b110_0011, b, funct3 = 0b000                     ) (rs1: xreg, rs2: xreg, imm: btype),
    Bne   ("bne",   0b110_0011, b, funct3 = 0b001                     ) (rs1: xreg, rs2: xreg, imm: btype),
    Blt   ("blt",   0b110_0011, b, funct3 = 0b100                     ) (rs1: xreg, rs2: xreg, imm: btype),
    Bge   ("bge",   0b110_0011, b, funct3 = 0b101                     ) (rs1: xreg, rs2: xreg, imm: btype),
    Bltu  ("bltu",  0b110_0011, b, funct3 = 0b110                     ) (rs1: xreg, rs2: xreg, imm: btype),
    Bgeu  ("bgeu",  0b110_0011, b, funct3 = 0b111                     ) (rs1: xreg, rs2: xreg, imm: btype),

    Lb    ("lb",    0b000_0011, i, funct3 = 0b000                     ) (rd: xreg, rs1: xreg, imm: itype_signed),
    Lh    ("lh",    0b000_0011, i, funct3 = 0b001                     ) (rd: xreg, rs1: xreg, imm: itype_signed),
    Lw    ("lw",    0b000_0011, i, funct3 = 0b010                     ) (rd: xreg, rs1: xreg, imm: itype_signed),
    Lbu   ("lbu",   0b000_0011, i, funct3 = 0b100                     ) (rd: xreg, rs1: xreg, imm: itype_signed),
    Lhu   ("lhu",   0b000_0011, i, funct3 = 0b101                     ) (rd: xreg, rs1: xreg, imm: itype_signed),

    Sb    ("sb",    0b010_0011, s, funct3 = 0b000                     ) (rs1: xreg, rs2: xreg, imm: stype),
    Sh    ("sh",    0b010_0011, s, funct3 = 0b001                     ) (rs1: xreg, rs2: xreg, imm: stype),
    Sw    ("sw",    0b010_0011, s, funct3 = 0b010                     ) (rs1: xreg, rs2: xreg, imm: stype),

    Addi  ("addi",  0b001_0011, i,                      funct3 = 0b000) (rd: xreg, rs1: xreg, imm: itype_signed),
    Slti  ("slti",  0b001_0011, i,                      funct3 = 0b010) (rd: xreg, rs1: xreg, imm: itype_signed),
    Sltiu ("sltiu", 0b001_0011, i,                      funct3 = 0b011) (rd: xreg, rs1: xreg, imm: itype_unsigned),
    Xori  ("xori",  0b001_0011, i,                      funct3 = 0b100) (rd: xreg, rs1: xreg, imm: itype_signed),
    Ori   ("ori",   0b001_0011, i,                      funct3 = 0b110) (rd: xreg, rs1: xreg, imm: itype_signed),
    Andi  ("andi",  0b001_0011, i,                      funct3 = 0b111) (rd: xreg, rs1: xreg, imm: itype_signed),
    Slli  ("slli",  0b001_0011, i, funct7 = 0b000_0000, funct3 = 0b001) (rd: xreg, rs1: xreg, shamt),
    Srli  ("srli",  0b001_0011, i, funct7 = 0b000_0000, funct3 = 0b101) (rd: xreg, rs1: xreg, shamt),
    Srai  ("srai",  0b001_0011, i, funct7 = 0b010_0000, funct3 = 0b101) (rd: xreg, rs1: xreg, shamt),

    Add   ("add",   0b011_0011, r, funct7 = 0b000_0000, funct3 = 0b000) (rd: xreg, rs1: xreg, rs2: xreg),
    Sub   ("sub",   0b011_0011, r, funct7 = 0b010_0000, funct3 = 0b000) (rd: xreg, rs1: xreg, rs2: xreg),
    Sll   ("sll",   0b011_0011, r, funct7 = 0b000_0000, funct3 = 0b001) (rd: xreg, rs1: xreg, rs2: xreg),
    Slt   ("slt",   0b011_0011, r, funct7 = 0b000_0000, funct3 = 0b010) (rd: xreg, rs1: xreg, rs2: xreg),
    Sltu  ("sltu",  0b011_0011, r, funct7 = 0b000_0000, funct3 = 0b011) (rd: xreg, rs1: xreg, rs2: xreg),
    Xor   ("xor",   0b011_0011, r, funct7 = 0b000_0000, funct3 = 0b100) (rd: xreg, rs1: xreg, rs2: xreg),
    Srl   ("srl",   0b011_0011, r, funct7 = 0b000_0000, funct3 = 0b101) (rd: xreg, rs1: xreg, rs2: xreg),
    Sra   ("sra",   0b011_0011, r, funct7 = 0b010_0000, funct3 = 0b101) (rd: xreg, rs1: xreg, rs2: xreg),
    Or    ("or",    0b011_0011, r, funct7 = 0b000_0000, funct3 = 0b110) (rd: xreg, rs1: xreg, rs2: xreg),
    And   ("and",   0b011_0011, r, funct7 = 0b000_0000, funct3 = 0b111) (rd: xreg, rs1: xreg, rs2: xreg),

    Fence ("fence", 0b000_1111, i,
        funct7_6_3 = 0b0000  ,
        rs1        = 0b0_0000,
        funct3     = 0b000   ,
        rd         = 0b0_0000,
    ) (fm, pred, succ),

    Ecall ("ecall", 0b111_0011, i,
        imm11_0    = 0x000   ,
        rs1        = 0b0_0000,
        funct3     = 0b000   ,
        rd         = 0b0_0000,
    ) (),
    Ebreak ("ebreak", 0b111_0011, i,
        imm11_0    = 0x001   ,
        rs1        = 0b0_0000,
        funct3     = 0b000   ,
        rd         = 0b0_0000,
    ) (),

    // Zfencei
    FenceI ("fence.i", 0b000_1111, i, funct3 = 0b001) (rd: xreg, rs1: xreg, imm: itype_unsigned),

    // Zcsr
    Csrrw  ("csrrw",   0b111_0011, i, funct3 = 0b001) (rd: xreg, csr, rs1: xreg),
    Csrrs  ("csrrs",   0b111_0011, i, funct3 = 0b010) (rd: xreg, csr, rs1: xreg),
    Csrrc  ("csrrc",   0b111_0011, i, funct3 = 0b011) (rd: xreg, csr, rs1: xreg),
    Csrrwi ("csrrwi",  0b111_0011, i, funct3 = 0b101) (rd: xreg, csr, uimm: csr),
    Csrrsi ("csrrsi",  0b111_0011, i, funct3 = 0b110) (rd: xreg, csr, uimm: csr),
    Csrrci ("csrrci",  0b111_0011, i, funct3 = 0b111) (rd: xreg, csr, uimm: csr),

    // RV32F
    Flw     ("flw",       0b000_0111, i, funct3 = 0b010)                                     (rd: freg, rs1: xreg, imm: itype_signed),
    Fsw     ("fsw",       0b010_0111, s, funct3 = 0b010)                                     (rs1: xreg, rs2: freg, imm: stype),

    FmaddS  ("fmadd.s",   0b100_0011, r, funct2 = 0b00)                                      (rd: freg, rs1: freg, rs2: freg, rs3: freg, rm),
    FmsubS  ("fmsub.s",   0b100_0111, r, funct2 = 0b00)                                      (rd: freg, rs1: freg, rs2: freg, rs3: freg, rm),
    FnmsubS ("fnmsub.s",  0b100_1011, r, funct2 = 0b00)                                      (rd: freg, rs1: freg, rs2: freg, rs3: freg, rm),
    FnmaddS ("fnmadd.s",  0b100_1111, r, funct2 = 0b00)                                      (rd: freg, rs1: freg, rs2: freg, rs3: freg, rm),

    FaddS   ("fadd.s",    0b101_0011, r, funct7 = 0b000_0000)                                (rd: freg, rs1: freg, rs2: freg, rm),
    FsubS   ("fsub.s",    0b101_0011, r, funct7 = 0b000_0100)                                (rd: freg, rs1: freg, rs2: freg, rm),
    FmulS   ("fmul.s",    0b101_0011, r, funct7 = 0b000_1000)                                (rd: freg, rs1: freg, rs2: freg, rm),
    FdivS   ("fdiv.s",    0b101_0011, r, funct7 = 0b000_1100)                                (rd: freg, rs1: freg, rs2: freg, rm),
    FsqrtS  ("fsqrt.s",   0b101_0011, r, funct7 = 0b010_1100, rs2 = 0b00000)                 (rd: freg, rs1: freg, rm),

    FsgnjS  ("fsgnj.s",   0b101_0011, r, funct7 = 0b001_0000, funct3 = 0b000)                (rd: freg, rs1: freg, rs2: freg),
    FsgnjnS ("fsgnjn.s",  0b101_0011, r, funct7 = 0b001_0000, funct3 = 0b001)                (rd: freg, rs1: freg, rs2: freg),
    FsgnjxS ("fsgnjx.s",  0b101_0011, r, funct7 = 0b001_0000, funct3 = 0b010)                (rd: freg, rs1: freg, rs2: freg),

    FminS   ("fmin.s",    0b101_0011, r, funct7 = 0b001_0100, funct3 = 0b000)                (rd: freg, rs1: freg, rs2: freg),
    FmaxS   ("fmax.s",    0b101_0011, r, funct7 = 0b001_0100, funct3 = 0b001)                (rd: freg, rs1: freg, rs2: freg),

    FcvtWS  ("fcvt.w.s",  0b101_0011, r, funct7 = 0b110_0000, rs2 = 0b00000)                 (rd: xreg, rs1: freg, rm),
    FcvtWuS ("fcvt.wu.s", 0b101_0011, r, funct7 = 0b110_0000, rs2 = 0b00001)                 (rd: xreg, rs1: freg, rm),

    FmvXW   ("fmv.x.w",   0b101_0011, r, funct7 = 0b111_0000, rs2 = 0b00000, funct3 = 0b000) (rd: xreg, rs1: freg),

    FeqS    ("feq.s",     0b101_0011, r, funct7 = 0b101_0000, funct3 = 0b010)                (rd: xreg, rs1: freg, rs2: freg),
    FltS    ("flt.s",     0b101_0011, r, funct7 = 0b101_0000, funct3 = 0b001)                (rd: xreg, rs1: freg, rs2: freg),
    FleS    ("fle.s",     0b101_0011, r, funct7 = 0b101_0000, funct3 = 0b000)                (rd: xreg, rs1: freg, rs2: freg),

    FclassS ("fclass.s",  0b101_0011, r, funct7 = 0b111_0000, rs2 = 0b00000, funct3 = 0b001) (rd: xreg, rs1: freg),

    FcvtSW  ("fcvt.s.w",  0b101_0011, r, funct7 = 0b110_1000, rs2 = 0b00000)                 (rd: freg, rs1: xreg, rm),
    FcvtSWu ("fcvt.s.wu", 0b101_0011, r, funct7 = 0b110_1000, rs2 = 0b00001)                 (rd: freg, rs1: xreg, rm),

    FmvWX   ("fmv.w.x",   0b101_0011, r, funct7 = 0b111_1000, rs2 = 0b00000, funct3 = 0b000) (rd: freg, rs1: xreg),

    // Privileged Instructions
    MRet    ("mret",      0b111_0011, r, funct7 = 0b001_1000, rs2 = 0b00010, rs1 = 0b00000, funct3 = 0b000, rd = 0b00000) (),
}

macro_rules! asm_display {
    (
        [$instr:ident]
        $( $name:ident $(($format:literal$(, $arg:expr)* $(,)?))? ),+ $(,)?
    ) => {
        $(
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #[allow(unused_variables)]
                let $instr = self;
                write!(f, "{:<12}", Self::MNEMONIC)?;
                $(
                write!(f, $format$(, $arg)*)?;
                )?

                Ok(())
            }
        }
        )+
    };
}

asm_display! {
    [i]
    Lui   ("{},0x{:05x}", i.rd(), i.imm() >> 12),
    Auipc ("{},0x{:05x}", i.rd(), i.imm() >> 12),

    Jal   ("{},{}", i.rd(), i.imm()),
    Jalr  ("{},{},{}", i.rd(), i.rs1(), i.imm()),

    Beq   ("{},{},{}", i.rs1(), i.rs2(), i.imm()),
    Bne   ("{},{},{}", i.rs1(), i.rs2(), i.imm()),
    Blt   ("{},{},{}", i.rs1(), i.rs2(), i.imm()),
    Bge   ("{},{},{}", i.rs1(), i.rs2(), i.imm()),
    Bltu  ("{},{},{}", i.rs1(), i.rs2(), i.imm()),
    Bgeu  ("{},{},{}", i.rs1(), i.rs2(), i.imm()),

    Lb    ("{},{}({})", i.rd(), i.imm(), i.rs1()),
    Lh    ("{},{}({})", i.rd(), i.imm(), i.rs1()),
    Lw    ("{},{}({})", i.rd(), i.imm(), i.rs1()),
    Lbu   ("{},{}({})", i.rd(), i.imm(), i.rs1()),
    Lhu   ("{},{}({})", i.rd(), i.imm(), i.rs1()),

    Sb    ("{},{}({})", i.rs1(), i.imm(), i.rs2()),
    Sh    ("{},{}({})", i.rs1(), i.imm(), i.rs2()),
    Sw    ("{},{}({})", i.rs1(), i.imm(), i.rs2()),

    Addi  ("{},{},{}", i.rd(), i.rs1(), i.imm()),
    Slti  ("{},{},{}", i.rd(), i.rs1(), i.imm()),
    Sltiu ("{},{},{}", i.rd(), i.rs1(), i.imm()),
    Xori  ("{},{},{}", i.rd(), i.rs1(), i.imm()),
    Ori   ("{},{},{}", i.rd(), i.rs1(), i.imm()),
    Andi  ("{},{},{}", i.rd(), i.rs1(), i.imm()),
    Slli  ("{},{},{}", i.rd(), i.rs1(), i.shamt()),
    Srli  ("{},{},{}", i.rd(), i.rs1(), i.shamt()),
    Srai  ("{},{},{}", i.rd(), i.rs1(), i.shamt()),

    Add   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    Sub   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    Sll   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    Slt   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    Sltu  ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    Xor   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    Srl   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    Sra   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    Or    ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    And   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),

    Fence ("{},{}", i.pred(), i.succ()),

    Ecall,
    Ebreak,

    // Zfencei
    FenceI,

    // Zcsr
    Csrrw  ("{},{},{}", i.rd(), i.csr(), i.rs1()),
    Csrrs  ("{},{},{}", i.rd(), i.csr(), i.rs1()),
    Csrrc  ("{},{},{}", i.rd(), i.csr(), i.rs1()),
    Csrrwi ("{},{},{}", i.rd(), i.csr(), i.uimm()),
    Csrrsi ("{},{},{}", i.rd(), i.csr(), i.uimm()),
    Csrrci ("{},{},{}", i.rd(), i.csr(), i.uimm()),

    // RV32F
    Flw    ("{},{}({})", i.rd(), i.imm(), i.rs1()),
    Fsw    ("{},{}({})", i.rs1(), i.imm(), i.rs2()),

    FmaddS  ("{},{},{},{},{}", i.rd(), i.rs1(), i.rs2(), i.rs3(), i.rm()),
    FmsubS  ("{},{},{},{},{}", i.rd(), i.rs1(), i.rs2(), i.rs3(), i.rm()),
    FnmsubS ("{},{},{},{},{}", i.rd(), i.rs1(), i.rs2(), i.rs3(), i.rm()),
    FnmaddS ("{},{},{},{},{}", i.rd(), i.rs1(), i.rs2(), i.rs3(), i.rm()),

    FaddS   ("{},{},{},{}", i.rd(), i.rs1(), i.rs2(), i.rm()),
    FsubS   ("{},{},{},{}", i.rd(), i.rs1(), i.rs2(), i.rm()),
    FmulS   ("{},{},{},{}", i.rd(), i.rs1(), i.rs2(), i.rm()),
    FdivS   ("{},{},{},{}", i.rd(), i.rs1(), i.rs2(), i.rm()),
    FsqrtS  ("{},{},{}", i.rd(), i.rs1(), i.rm()),

    FsgnjS  ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    FsgnjnS ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    FsgnjxS ("{},{},{}", i.rd(), i.rs1(), i.rs2()),

    FmaxS   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    FminS   ("{},{},{}", i.rd(), i.rs1(), i.rs2()),

    FcvtWS  ("{},{},{}", i.rd(), i.rs1(), i.rm()),
    FcvtWuS ("{},{},{}", i.rd(), i.rs1(), i.rm()),

    FmvXW   ("{},{}",    i.rd(), i.rs1()),

    FeqS    ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    FltS    ("{},{},{}", i.rd(), i.rs1(), i.rs2()),
    FleS    ("{},{},{}", i.rd(), i.rs1(), i.rs2()),

    FclassS ("{},{}", i.rd(), i.rs1()),

    FcvtSW  ("{},{},{}", i.rd(), i.rs1(), i.rm()),
    FcvtSWu ("{},{},{}", i.rd(), i.rs1(), i.rm()),

    FmvWX   ("{},{}", i.rd(), i.rs1()),

    // Privileged Instructions
    MRet,
}
