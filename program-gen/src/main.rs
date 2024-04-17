use rvhwfuzzer_program_gen::{generate_binary, GenerationConfig};

use std::io;

struct SpeedCommand {
    nr_tests: u32,
    min_bbs: usize,
    max_bbs: usize,
    min_instrs_per_bb: usize,
    max_instrs_per_bb: usize,
}

enum Args {
    Speed(SpeedCommand),
}

enum ArgsError {
    MissingBin,
    Usage,
    Conversion(String),
    InvalidCommand(String),
}

fn usage(writer: &mut impl io::Write) -> io::Result<()> {
    writeln!(writer, "Usage: {} <CMD> [..ARGS]", env!("CARGO_CRATE_NAME"))?;
    writeln!(writer,)?;
    writeln!(writer, "Commands:")?;
    writeln!(writer, "  - speed <# of tests> <min bbs> <max bbs> <min i per bb> <max i per bb>")?;

    Ok(())
}

fn args() -> Result<Args, ArgsError> {
    use ArgsError as E;

    let mut args = std::env::args();

    args.next().unwrap();

    let cmd = args.next().ok_or(E::Usage)?;

    match &cmd[..] {
        "speed" => {
            let nr_tests = args.next().ok_or(E::Usage)?;
            let min_bbs = args.next().ok_or(E::Usage)?;
            let max_bbs = args.next().ok_or(E::Usage)?;
            let min_instrs_per_bb = args.next().ok_or(E::Usage)?;
            let max_instrs_per_bb = args.next().ok_or(E::Usage)?;

            let nr_tests: u32 = nr_tests.parse().map_err(|_| E::Conversion(nr_tests))?;
            let min_bbs: usize = min_bbs.parse().map_err(|_| E::Conversion(min_bbs))?;
            let max_bbs: usize = max_bbs.parse().map_err(|_| E::Conversion(max_bbs))?;
            let min_instrs_per_bb: usize = min_instrs_per_bb
                .parse()
                .map_err(|_| E::Conversion(min_instrs_per_bb))?;
            let max_instrs_per_bb: usize = max_instrs_per_bb
                .parse()
                .map_err(|_| E::Conversion(max_instrs_per_bb))?;

            Ok(Args::Speed(SpeedCommand {
                nr_tests,
                min_bbs,
                max_bbs,
                min_instrs_per_bb,
                max_instrs_per_bb,
            }))
        }
        _ => Err(E::InvalidCommand(cmd)),
    }
}

fn main() -> std::io::Result<()> {
    let args = args().unwrap_or_else(|err| {
        use ArgsError as E;

        match err {
            E::Usage => {
                usage(&mut std::io::stderr()).unwrap();
            }
            E::MissingBin => eprintln!("Binary is missing somehow"),
            E::Conversion(v) => eprintln!("Conversion of '{v}' failed."),
            E::InvalidCommand(cmd) => {
                eprintln!("Invalid command '{cmd}')");
                eprintln!();
                usage(&mut std::io::stderr()).unwrap();
            }
        }

        std::process::exit(2)
    });

    match args {
        Args::Speed(cmd) => {
            let SpeedCommand { nr_tests, min_bbs, max_bbs, min_instrs_per_bb, max_instrs_per_bb } = cmd;
            let mut total_num_instructions = 0u64;

            let config = GenerationConfig {
                entry: 0x8000_0000,
                data_memory_ranges: &[0x7000_0000..0x8000_0000],
                num_basic_block_range: min_bbs..max_bbs,
                num_instrs_per_bb_range: min_instrs_per_bb..max_instrs_per_bb,
            };

            let start = std::time::Instant::now();

            for _ in 0..nr_tests {
                let binary = generate_binary(&config)?;
                total_num_instructions += binary.num_instructions();
            }

            let end = start.elapsed();

            let total_time = end.as_secs_f64();

            let avg_time = total_time / (nr_tests as f64);
            let avg_num_instructions = (total_num_instructions as f64) / (nr_tests as f64);

            println!("{total_num_instructions},{avg_num_instructions},{total_time},{avg_time},{nr_tests},{min_bbs},{max_bbs},{min_instrs_per_bb},{max_instrs_per_bb}");
        }
    }

    Ok(())
}
