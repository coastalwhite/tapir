use rvhwfuzzer_program_gen::{generate_binary, MemoryArea};

fn main() -> std::io::Result<()> {
    use std::io::Write;

    // let mut stdout = std::io::stdout().lock();
    // let stdout = &mut stdout;

    let mut num_instructions = 0u64;

    for i in 0..100000 {
        let binary = generate_binary(0x8000_0000, &[0x7000_0000..0x7000_1000])?;

        // for (i, b) in binary.iter().enumerate() {
        //     if i != 0 && i % 8 == 0 {
        //         writeln!(stdout)?;
        //     }
        //
        //     write!(stdout, "{b:02X} ")?;
        // }
        //
        // writeln!(stdout)?;
        //
        // std::fs::write("test.bin", &binary)?;
        //
        // writeln!(stdout)?;

        num_instructions += (binary.bin.len().as_usize() / 4) as u64;

        // writeln!(stdout, "Bytes: {}", binary.len())?;
        // writeln!(stdout, "Instructions: ~{}", binary.len() / 4)?;
        //
        // writeln!(stdout)?;
    }

    println!("# of instructions: {num_instructions}");

    Ok(())
}
