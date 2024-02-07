use std::fmt::Display;
use std::io;

mod result;

use kiwi_toml::{FieldDisplay, FieldVariant, Instruction, KiwiFile};

pub struct BitContainerType(u32);

impl Display for BitContainerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num_bits = self.0;

        if num_bits <= 8 {
            return f.write_str("u8");
        }

        if num_bits <= 16 {
            return f.write_str("u16");
        }

        if num_bits <= 32 {
            return f.write_str("u32");
        }

        if num_bits <= 64 {
            return f.write_str("u64");
        }

        todo!();

        // let num_cells = num_bits.div_ceil(64);
        //
        // write!(f, "[u64; {num_cells}]")
    }
}

pub fn kebab_to_camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());

    let mut is_uppercase = true;

    for c in name.chars() {
        if c == '-' {
            is_uppercase = true;
            continue;
        }

        let c = if is_uppercase {
            c.to_ascii_uppercase()
        } else {
            c
        };
        is_uppercase = false;

        out.push(c);
    }

    out
}

pub fn to_args_struct_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());

    let mut is_uppercase = true;

    for c in name.chars() {
        if c == '_' || c == '-' {
            is_uppercase = true;
            continue;
        }

        let c = if is_uppercase {
            c.to_ascii_uppercase()
        } else {
            c
        };
        is_uppercase = false;

        out.push(c);
    }

    out
}

pub fn generate_args_struct(
    kiwi: &KiwiFile,
    instruction: &Instruction,
    writer: &mut impl io::Write,
) -> io::Result<()> {
    let struct_name = format!("{}Args", to_args_struct_name(instruction.name()));

    writeln!(writer, "#[derive(Debug)]")?;
    writeln!(writer, "pub struct {struct_name} {{")?;
    let format = &kiwi.formats()[instruction.encoding().format()];

    let format_bittype = BitContainerType(format.num_bits());

    for (part_name, part) in format.parts() {
        if instruction.encoding().predefined().contains(&part_name) {
            continue;
        }

        let part_name = instruction
            .encoding()
            .renamings()
            .get(part_name)
            .unwrap_or(part_name);

        let part_type = BitContainerType(part.num_bits);
        writeln!(writer, "\tpub {part_name}: {part_type},",)?;
    }

    writeln!(writer, "}}")?;

    writeln!(writer, "impl {struct_name} {{")?;
    writeln!(
        writer,
        "\tpub fn take_args(encoded: {format_bittype}) -> Self {{"
    )?;
    writeln!(writer, "\t\tSelf {{")?;

    for (part_name, part) in format.parts() {
        if instruction.encoding().predefined().contains(&part_name) {
            continue;
        }

        let part_name = instruction
            .encoding()
            .renamings()
            .get(part_name)
            .unwrap_or(part_name);

        let mask_ones = "1".repeat(part.num_bits as usize);
        let offset = part.offset;

        let part_type = BitContainerType(part.num_bits);

        writeln!(
            writer,
            "\t\t\t{part_name}: ((encoded >> {offset}) & 0b{mask_ones}) as {part_type},"
        )?;
    }

    writeln!(writer, "\t\t}}")?;

    writeln!(writer, "\t}}")?;

    writeln!(
        writer,
        "\tpub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {{"
    )?;

    let mask = instruction.encoding().mask().as_u32().unwrap();
    writeln!(
        writer,
        "\t\tlet mut encoded: {format_bittype} = 0b{mask:00$b};",
        format.num_bits() as usize
    )?;

    for (part_name, part) in format.parts() {
        if instruction.encoding().predefined().contains(&part_name) {
            continue;
        }

        let part_name = instruction
            .encoding()
            .renamings()
            .get(part_name)
            .unwrap_or(part_name);

        let offset = part.offset;
        writeln!(
            writer,
            "\t\tencoded |= (self.{part_name} as {format_bittype}) << {offset};"
        )?;
    }

    writeln!(writer)?;
    writeln!(writer, "\t\twriter.write_all(&encoded.to_le_bytes())")?;

    writeln!(writer, "\t}}")?;

    for (added_field_name, added_field_parts) in format.added_fields() {
        let mut is_variable = false;
        let mut is_overwritten = false;

        for part in added_field_parts.iter() {
            match part {
                kiwi_toml::raw::FormatFieldPart::Bits(_) => {}
                kiwi_toml::raw::FormatFieldPart::Field(field) => {
                    if !instruction.encoding().predefined().contains(field) {
                        is_variable = true;
                    }

                    if instruction.encoding().renamings().contains_key(field) {
                        is_overwritten = true;
                    }
                }
            }
        }

        if !is_variable || is_overwritten {
            continue;
        }

        write!(writer, "\t/// ")?;
        for part in added_field_parts.iter() {
            match part {
                kiwi_toml::raw::FormatFieldPart::Bits(bits) => write!(
                    writer,
                    "{:0.*b}",
                    bits.num_bits() as usize,
                    bits.as_u32().unwrap()
                )?,
                kiwi_toml::raw::FormatFieldPart::Field(field) => {
                    write!(writer, "{}", field.as_str())?
                }
            }

            write!(writer, " @ ")?;
        }
        writeln!(writer)?;
        writeln!(writer, "\tpub fn {added_field_name}(&self) -> u32 {{")?;

        write!(writer, "\t\t")?;

        let mut offset = 0u32;
        for part in added_field_parts.iter().rev() {
            match part {
                kiwi_toml::raw::FormatFieldPart::Bits(bits) => {
                    let bbits = bits.as_u32().unwrap();
                    write!(writer, "((0b{bbits:b} as u32) << {offset}) | ")?;
                    offset += bits.num_bits();
                }
                kiwi_toml::raw::FormatFieldPart::Field(field_name) => {
                    // dbg!(field_name);
                    let part_field = format.parts().get(field_name).unwrap();
                    write!(writer, "((self.{field_name} as u32) << {offset}) | ")?;
                    offset += part_field.num_bits;
                }
            }
        }

        writeln!(writer, "0")?;
        writeln!(writer, "\t}}")?;
    }

    writeln!(writer, "}}")?;

    Ok(())
}

fn field_display_intrinsic_fmt_codegen(
    writer: &mut impl io::Write,
    field_variant: &str,
    field_display: &str,
    indent: &str,
) -> io::Result<()> {
    match (field_variant, field_display) {
        ("signed-int", "default") => {
            writeln!(writer, "{indent}write!(f, \"{{}}\", self.0 as i64)?;")?;
        }
        ("unsigned-int", "default") => {
            writeln!(writer, "{indent}write!(f, \"{{}}\", self.0 as u64)?;")?;
        }
        ("signed-hex", "default") => {
            writeln!(writer, "{indent}write!(f, \"{{:x}}\", self.0 as i64)?;")?;
        }
        ("unsigned-hex", "default") => {
            writeln!(writer, "{indent}write!(f, \"{{:x}}\", self.0 as u64)?;")?;
        }
        ("rel-label", "imm") => {
            writeln!(writer, "{indent}write!(f, \"{{}}\", self.0 as i64)?;")?;
        }
        _ => panic!(),
    }

    Ok(())
}

fn field_display_fmt_codegen(
    writer: &mut impl io::Write,
    field_variant: &FieldVariant,
    field_display: &FieldDisplay,
    indent: &str,
) -> io::Result<()> {
    match field_display {
        FieldDisplay::Template(_) => {
            writeln!(writer, "{indent}todo!();")?;
        }
        FieldDisplay::Lut(lookup) => {
            assert_eq!(field_variant.arg_max_sizes().len(), 1);
            let bitsize = field_variant.arg_max_sizes()[0];
            let size = 1 << bitsize;

            let mask = size - 1;

            write!(writer, "{indent}static LUT: [&str; {size}] = [")?;

            for elem in lookup {
                // FIX. This needs to be escaped.
                write!(writer, r#""{elem}", "#)?;
            }

            writeln!(writer, "];")?;
            writeln!(
                writer,
                "{indent}f.write_str(LUT[(self.0 & 0b{mask:b}) as usize])?;"
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use kiwi_toml::FieldDisplay;

    use super::*;

    #[test]
    fn it_works() -> io::Result<()> {
        use io::Write;

        let file = KiwiFile::from_str(include_str!("../../example.kiwi.toml")).unwrap();
        dbg!(&file.formats());
        let mut out = std::fs::File::create("test.rs").unwrap();

        writeln!(&mut out, "pub mod asm {{")?;
        writeln!(&mut out, "\t#[derive(Debug)]")?;
        writeln!(&mut out, "\tpub struct AsmDisplay<'a, T> {{")?;
        writeln!(&mut out, "\t\tpub ctx: &'a AsmDisplayContext,")?;
        writeln!(&mut out, "\t\tpub instr: &'a T,")?;
        writeln!(&mut out, "\t}}")?;

        writeln!(&mut out, "\tpub trait AsmField {{")?;
        writeln!(&mut out, "\t\tfn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: &AsmDisplayContext) -> ::std::fmt::Result;")?;
        writeln!(&mut out, "\t}}")?;

        // writeln!(&mut out, "\timpl ::std::fmt::Display for dyn AsmField<Context = ()> {{")?;
        // writeln!(&mut out, "\t\tfn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {{")?;
        // writeln!(&mut out, "\t\t\tself.fmt_field(f, &())")?;
        // writeln!(&mut out, "\t\t}}")?;
        // writeln!(&mut out, "\t}}")?;

        writeln!(&mut out, "\t#[derive(Debug, Default)]")?;
        writeln!(&mut out, "\tpub struct AsmDisplayContext {{")?;
        for field_variant in file.field_variants() {
            if field_variant.display().len() == 1 {
                continue;
            }

            let name = field_variant.name();
            let struct_name = to_args_struct_name(field_variant.name());
            writeln!(&mut out, "\t\t{name}: \t{struct_name}DisplayVariant,")?;
        }
        writeln!(&mut out, "\t}}")?;

        for field_variant in file.field_variants() {
            let name = field_variant.name();
            let struct_name = to_args_struct_name(field_variant.name());

            if field_variant.display().len() > 1 {
                writeln!(&mut out, "\t#[derive(Debug, Default)]")?;
                writeln!(&mut out, "\tpub enum {struct_name}DisplayVariant {{")?;

                for (display_name, _) in field_variant.display() {
                    if field_variant.default_display() == display_name {
                        writeln!(&mut out, "\t\t#[default]")?;
                    }

                    let variant_name = to_args_struct_name(display_name);
                    writeln!(&mut out, "\t\t{variant_name},")?;
                }

                writeln!(&mut out, "\t}}")?;
            }

            writeln!(&mut out, "\t#[derive(Debug, Default)]")?;
            write!(&mut out, "\tpub struct Asm{struct_name}(")?;

            // NOTE temporary solution for intrinsics

            if field_variant.display()[0].1.is_none() {
                write!(&mut out, "u64")?;
            } else {
                for max_size in field_variant.arg_max_sizes() {
                    let container = BitContainerType(*max_size);
                    write!(&mut out, "{container}, ")?;
                }
            }

            writeln!(&mut out, ");")?;

            writeln!(&mut out, "\timpl AsmField for Asm{struct_name} {{")?;
            if field_variant.display().len() > 1 {
                writeln!(&mut out, "\t\tfn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: &AsmDisplayContext) -> ::std::fmt::Result {{")?;
                writeln!(&mut out, "\t\t\tmatch ctx.{name} {{")?;

                for (display_name, display) in field_variant.display() {
                    let variant_name = to_args_struct_name(display_name);
                    writeln!(
                        &mut out,
                        "\t\t\t\t{struct_name}DisplayVariant::{variant_name} => {{"
                    )?;

                    if let Some(display) = display {
                        field_display_fmt_codegen(&mut out, field_variant, display, "\t\t\t\t\t")?;
                    } else {
                        field_display_intrinsic_fmt_codegen(
                            &mut out,
                            &variant_name,
                            display_name,
                            "\t\t\t\t\t",
                        )?;
                    }

                    writeln!(&mut out, "\t\t\t\t}},")?;
                }

                writeln!(&mut out, "\t\t\t}}")?;
            } else {
                writeln!(&mut out, "\t\tfn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, _: &AsmDisplayContext) -> ::std::fmt::Result {{")?;
                if let Some(display) = &field_variant.display()[0].1 {
                    field_display_fmt_codegen(&mut out, field_variant, display, "\t\t\t")?;
                } else {
                    field_display_intrinsic_fmt_codegen(
                        &mut out,
                        field_variant.name(),
                        &field_variant.display()[0].0,
                        "\t\t\t",
                    )?;
                }
            }

            writeln!(&mut out, "\t\t\tOk(())")?;
            writeln!(&mut out, "\t\t}}")?;
            writeln!(&mut out, "\t}}")?;
        }

        let max_mnemonic_length = file
            .instructions()
            .iter()
            .map(|i| i.assembly().mnemonic().len())
            .max()
            .unwrap_or(0);

        for instruction in file.instructions() {
            let struct_name = to_args_struct_name(instruction.name());

            let asm = instruction.assembly();

            let mnemonic = asm.mnemonic();

            writeln!(
                &mut out,
                "\timpl<'a> ::std::fmt::Display for AsmDisplay<'a, super::{struct_name}Args> {{"
            )?;
            writeln!(
                &mut out,
                "\t\tfn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {{"
            )?;
            writeln!(&mut out, "\t\t\tf.write_str(\"{mnemonic:<0$}\")?;", max_mnemonic_length + 2)?;

            for (i, field) in asm.fields().iter().enumerate() {
                let variant = field.variant(&file);
                let field_name = to_args_struct_name(variant.name());
                write!(&mut out, "\t\t\tAsm{field_name}(")?;

                for arg in field.arguments() {
                    let format = instruction.format(&file);

                    if format.parts().contains_key(arg) {
                        write!(&mut out, "self.instr.{arg}.into(), ")?;
                    } else {
                        let f = format.added_fields().iter().find(|(s, _)| s == arg);

                        if f.is_some() {
                            write!(&mut out, "self.instr.{arg}().into(), ")?;
                        } else {
                            write!(&mut out, "self.instr.{arg}.into(), ")?;
                        }
                    }
                }

                writeln!(&mut out, ").fmt_field(f, self.ctx)?;")?;

                if i != asm.fields().len() - 1 {
                    writeln!(&mut out, "\t\t\tf.write_str(\",\")?;")?;
                }
            }

            writeln!(&mut out, "\t\t\tOk(())")?;
            writeln!(&mut out, "\t\t}}")?;
            writeln!(&mut out, "\t}}")?;
        }

        writeln!(
            &mut out,
            "\timpl<'a> ::std::fmt::Display for AsmDisplay<'a, super::Instruction> {{"
        )?;
        writeln!(
            &mut out,
            "\t\tfn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {{"
        )?;
        writeln!(&mut out, "\t\t\tuse super::Instruction as I;")?;
        writeln!(&mut out, "\t\t\tmatch self.instr {{")?;

        for instruction in file.instructions() {
            let struct_name = to_args_struct_name(instruction.name());
            writeln!(&mut out, "\t\t\t\tI::{struct_name}(ref args) => AsmDisplay {{ ctx: self.ctx, instr: args }}.fmt(f),")?;
        }

        writeln!(&mut out, "\t\t\t}}")?;
        writeln!(&mut out, "\t\t}}")?;
        writeln!(&mut out, "\t}}")?;

        writeln!(&mut out, "}}")?;

        // assert!(false);

        for i in file.instructions() {
            generate_args_struct(&file, &i, &mut out)?;
        }

        writeln!(&mut out, "#[derive(Debug)]")?;
        writeln!(&mut out, "pub enum Instruction {{")?;
        for i in file.instructions() {
            let struct_name = to_args_struct_name(i.name());
            writeln!(&mut out, "\t{struct_name}({struct_name}Args),")?;
        }
        writeln!(&mut out, "}}")?;

        writeln!(&mut out, "impl Instruction {{")?;
        writeln!(
            &mut out,
            "\tpub fn decode(encoded: &[u8]) -> Option<Instruction> {{"
        )?;

        let mut bit_size = 0u32;

        for i in file.instructions() {
            let struct_name = to_args_struct_name(i.name());

            let format = &file.formats()[i.encoding().format()];

            assert!(format.num_bits() % 8 == 0);

            let format_bittype = BitContainerType(format.num_bits());
            if format.num_bits() != bit_size {
                // FIX
                write!(
                    &mut out,
                    "\t\tlet bits: {format_bittype} = {format_bittype}::from_le_bytes(["
                )?;
                for i in 0..format.num_bits() / 8 {
                    write!(&mut out, "encoded[{i}], ")?;
                }
                writeln!(&mut out, "]);")?;

                bit_size = format.num_bits();
            }

            let activated = i.encoding().activated().as_u32().unwrap();
            let mask = i.encoding().mask().as_u32().unwrap();

            writeln!(
                &mut out,
                "\t\tif bits & 0b{activated:00$b} == 0b{mask:00$b} {{",
                format.num_bits() as usize
            )?;
            writeln!(
                &mut out,
                "\t\t\treturn Some(Self::{struct_name}({struct_name}Args::take_args(bits)));"
            )?;
            writeln!(&mut out, "\t\t}}")?;
        }
        writeln!(&mut out, "\t\tNone")?;
        writeln!(&mut out, "\t}}")?;
        writeln!(&mut out, "}}")?;

        assert!(false);

        Ok(())
    }
}
