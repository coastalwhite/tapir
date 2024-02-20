use std::iter::FusedIterator;

use risico::syscall::ECallBehavior;
use risico::trap::TrapBehavior;

static BASIC_USAGE: &str = r#"
Usage: risico <FILE> [flags]
"#;

static USAGE: &str = r#"
Usage: risico <FILE> [flags]

RISICO is a RISC-V emulator that allows to models side-channels and fault
injection techniques. It can run binaries in 2 modes.

1. Syscall Emulation: Run an ELF binary and emulate the system calls as Linux
2. Raw Image Emulation: Run raw binary instructions
3. Device Config: Run with a device config file

Flags:
 -d / --dump: instead of executing, dump the instructions
 -R / --raw: run the file as a raw image
 -D / --device-config: use a device config file
 -h / --help: show the usage string
 -S / --syscalls <system call handler>: how to handle system calls
 -t / --traps <trap handler>: how to handle traps
 -L / --logging <list of logging topics>: which logging to perform
 -T / --trace <output file>: output a trace to a file

Raw Image Flags:
 -E / --entry <entry address: select the entry address

Device Config Flags:
 -l / --load: load a file into memory
"#;

#[derive(Debug)]
pub enum RunType {
    SyscallEmulation,
    RawImage(RawImageFlags),
    DeviceConfig(DeviceConfigFlags),
}

#[derive(Debug)]
pub struct RawImageFlags {
    entry: u32,
}

#[derive(Debug)]
pub struct CliFlags {
    file: String,
    do_dump: bool,
    run_type: RunType,
    system_call_behavior: ECallBehavior,
    trap_behavior: TrapBehavior,
    logging: LoggingConfiguration,
    trace: Option<String>,
}

#[derive(Debug)]
pub struct LoggingConfiguration {
    pub show_instructions: bool,
    pub show_cycles: bool,
}

#[derive(Debug)]
pub struct DeviceConfigFlags {}

impl Default for LoggingConfiguration {
    fn default() -> Self {
        Self {
            show_instructions: false,
            show_cycles: false,
        }
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    if s.starts_with("0x") {
        u32::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse().ok()
    }
}

fn parse_ecall_behavior(s: &str) -> Option<ECallBehavior> {
    match s.trim() {
        "testing" => Some(ECallBehavior::Testing),
        "linux" => Some(ECallBehavior::Linux),
        "trapvec" => Some(ECallBehavior::TrapVector),
        _ => None,
    }
}

fn parse_trap_cause(s: &str) -> Result<TrapBehavior, String> {
    macro_rules! causes {
        (@delimited $fst:literal) => {{ $f }};
        (@delimited $fst:literal$(, $after:literal)+) => {{ concat!($fst$(, ", ", $after)+) }};
        ($($trap_cause:literal => $behavior:expr),+ $(,)?) => {
            match s {
                $(
                $trap_cause => Ok($behavior),
                )+
                _ => Err(format!("Invalid trap cause specifier '{s}'. Allowed values: {}", causes!(@delimited $($trap_cause),+))),
            }
        };
    }

    causes! {
        "all" => TrapBehavior::abort_all(),
        "ecall" => TrapBehavior::ABORT_ON_ENV_CALL_FROM_U_MODE | TrapBehavior::ABORT_ON_ENV_CALL_FROM_S_MODE | TrapBehavior::ABORT_ON_ENV_CALL_FROM_M_MODE,
        "instr_misaligned" => TrapBehavior::ABORT_ON_INSTR_ADDRESS_MISALIGNED,
        "instr_access_fault" => TrapBehavior::ABORT_ON_INSTR_ACCESS_FAULT,
        "illegal_instr" => TrapBehavior::ABORT_ON_ILLEGAL_INSTRUCTION,
        "breakpoint" => TrapBehavior::ABORT_ON_BREAKPOINT,
        "load_misaligned" => TrapBehavior::ABORT_ON_LOAD_ADDR_MISALIGNED,
        "load_access_fault" => TrapBehavior::ABORT_ON_LOAD_ACCESS_FAULT,
        "store_amo_misaligned" => TrapBehavior::ABORT_ON_STORE_AMO_ADDRESS_MISALIGNED,
        "store_amo_access_fault" => TrapBehavior::ABORT_ON_STORE_AMO_ACCESS_FAULT,
        "smode_ecall" => TrapBehavior::ABORT_ON_ENV_CALL_FROM_S_MODE,
        "umode_ecall" => TrapBehavior::ABORT_ON_ENV_CALL_FROM_U_MODE,
        // "reserved10" => TrapBehavior::ABORT_ON_RESERVED_10,
        "mmode_ecall" => TrapBehavior::ABORT_ON_ENV_CALL_FROM_M_MODE,
        "instr_page_fault" => TrapBehavior::ABORT_ON_INSTR_PAGE_FAULT,
        "load_page_fault" => TrapBehavior::ABORT_ON_LOAD_PAGE_FAULT,
        // "reserved14" => TrapBehavior::ABORT_ON_RESERVED_14,
        "store_amo_page_fault" => TrapBehavior::ABORT_ON_STORE_AMO_PAGE_FAULT,

        "missing_csr" => TrapBehavior::ABORT_ON_MISSING_CSR,
    }
}

fn parse_logging_config(s: &str) -> Option<LoggingConfiguration> {
    let mut config = LoggingConfiguration::default();

    for item in s.split(',') {
        match item.trim() {
            "i" | "instruction" => config.show_instructions = true,
            "c" | "cycles" => config.show_cycles = true,
            _ => return None,
        }
    }

    Some(config)
}

impl Default for RawImageFlags {
    fn default() -> Self {
        Self { entry: 0 }
    }
}

impl RawImageFlags {
    pub fn entry(&self) -> u32 {
        self.entry
    }
}

impl CliFlags {
    pub fn take() -> CliFlags {
        let mut file = None;

        let mut do_dump = false;
        let mut do_fail = false;
        let mut raw_image_flags = RawImageFlags::default();
        let mut run_type = RunType::SyscallEmulation;
        let mut trap_behavior = TrapBehavior::empty();
        let mut system_call_behavior = ECallBehavior::Linux;
        let mut logging = LoggingConfiguration {
            show_instructions: false,
            show_cycles: false,
        };
        let mut trace = None;

        let mut args = std::env::args();
        args.next().expect("Failed to skip over binary");
        loop {
            let Some(arg) = args.next() else {
                break;
            };

            match &arg[..] {
                "--help" | "-h" => {
                    println!("{USAGE}");
                    std::process::exit(0);
                }
                "--dump" | "-d" => {
                    do_dump = true;
                }
                "--raw" | "-R" => {
                    run_type = RunType::RawImage(RawImageFlags::default());
                }
                "--entry" | "-E" => {
                    let Some(entry_value) = args.next() else {
                        eprintln!("No entry value given");
                        do_fail = true;
                        continue;
                    };

                    let Some(entry) = parse_u32(&entry_value) else {
                        eprintln!("Entry value '{entry_value}' is invalid");
                        do_fail = true;
                        continue;
                    };

                    raw_image_flags.entry = entry;
                }
                "--syscalls" | "-S" => {
                    let Some(syscall_behavior) = args.next() else {
                        eprintln!("No system call behavior given");
                        do_fail = true;
                        continue;
                    };

                    let Some(syscall_behavior) = parse_ecall_behavior(&syscall_behavior) else {
                        eprintln!("System call behavior '{syscall_behavior}' is invalid");
                        do_fail = true;
                        continue;
                    };

                    system_call_behavior = syscall_behavior;
                }
                "--traps" | "-t" => {
                    let Some(set_trap_behavior) = args.next() else {
                        eprintln!("No trap behavior given");
                        do_fail = true;
                        continue;
                    };

                    let (is_abort, set_trap_behavior) = if let Some(set_trap_behavior) =
                        set_trap_behavior.strip_suffix("=allow")
                    {
                        (false, set_trap_behavior)
                    } else if let Some(set_trap_behavior) = set_trap_behavior.strip_suffix("=abort")
                    {
                        (true, set_trap_behavior)
                    } else {
                        (true, &set_trap_behavior[..])
                    };

                    match parse_trap_cause(&set_trap_behavior) {
                        Ok(set_trap_behavior) => {
                            if is_abort {
                                trap_behavior |= set_trap_behavior;
                            } else {
                                trap_behavior = trap_behavior.set_minus(set_trap_behavior);
                            }
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                            do_fail = true;
                        }
                    }
                }
                "--logging" | "-L" => {
                    let Some(logging_config) = args.next() else {
                        eprintln!("No logging behavior given");
                        do_fail = true;
                        continue;
                    };

                    let Some(logging_config) = parse_logging_config(&logging_config) else {
                        eprintln!("Logging '{logging_config}' is invalid");
                        do_fail = true;
                        continue;
                    };

                    logging = logging_config;
                }
                "--trace" | "-T" => {
                    let Some(trace_value) = args.next() else {
                        eprintln!("No trace file given");
                        do_fail = true;
                        continue;
                    };

                    trace = Some(trace_value);
                }
                "--device-config" | "-D" => {
                    run_type = RunType::DeviceConfig(DeviceConfigFlags {});
                }

                arg if arg.starts_with('-') => {
                    eprintln!("Unrecognized flag '{arg}'");
                    do_fail = true;
                }
                _ => {
                    if file.is_some() {
                        eprintln!("Unknown additional argument '{arg}'");
                        do_fail = true;
                    } else {
                        file = Some(arg);
                    }
                }
            }
        }

        let Some(file) = file else {
            eprintln!("No file given!");
            eprintln!("");
            eprintln!("{BASIC_USAGE}");
            std::process::exit(1);
        };

        if do_fail {
            std::process::exit(1);
        }

        if matches!(run_type, RunType::RawImage(_)) {
            run_type = RunType::RawImage(raw_image_flags);
        }

        CliFlags {
            file,
            do_dump,
            run_type,
            system_call_behavior,
            trap_behavior,
            logging,
            trace,
        }
    }

    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn do_dump(&self) -> bool {
        self.do_dump
    }

    pub fn run_type(&self) -> &RunType {
        &self.run_type
    }

    pub fn system_call_behavior(&self) -> ECallBehavior {
        self.system_call_behavior
    }

    pub fn trap_behavior(&self) -> TrapBehavior {
        self.trap_behavior
    }

    pub fn logging(&self) -> &LoggingConfiguration {
        &self.logging
    }

    pub fn trace(&self) -> Option<&str> {
        self.trace.as_ref().map(|s| &s[..])
    }
}
