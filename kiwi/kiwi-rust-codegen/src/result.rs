use std::io;

use kiwi_toml::result::*;

const INDENT: &str = "    ";

#[inline]
fn indent(writer: &mut impl io::Write, indentation: u32) -> io::Result<()> {
    for _ in 0..indentation {
        writer.write_all(INDENT.as_bytes())?;
    }

    Ok(())
}

fn write_isa<'a>(isa: &Isa<'a>, writer: &mut impl io::Write) -> io::Result<()> {
    for ty in isa.types() {
        let Type::Enum(ty) = ty else {
            continue;
        };

        let pascal_case_name = ty.name().pascal_case();

        writeln!(writer, "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]")?;
        writeln!(writer, "pub struct {pascal_case_name} {{")?;
            
        for variant in ty.variants() {
            let pascal_case_variant = variant.pascal_case();

            indent(writer, 1)?;
            writeln!(writer, "{pascal_case_variant},")?;
        }

        writeln!(writer, "}}")?;
    }


    Ok(())
}
