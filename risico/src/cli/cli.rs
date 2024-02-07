use risico::syscall::SystemCallBehavior;

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
    system_call_behavior: SystemCallBehavior,
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

fn parse_syscall_behavior(s: &str) -> Option<SystemCallBehavior> {
    match s.trim() {
        "abort" => Some(SystemCallBehavior::Abort),
        "testing" => Some(SystemCallBehavior::Testing),
        "linux" => Some(SystemCallBehavior::Linux),
        _ => None,
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
        let mut system_call_behavior = SystemCallBehavior::Linux;
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

                    let Some(syscall_behavior) = parse_syscall_behavior(&syscall_behavior) else {
                        eprintln!("System call behavior '{syscall_behavior}' is invalid");
                        do_fail = true;
                        continue;
                    };

                    system_call_behavior = syscall_behavior;
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

    pub fn system_call_behavior(&self) -> SystemCallBehavior {
        self.system_call_behavior
    }

    pub fn logging(&self) -> &LoggingConfiguration {
        &self.logging
    }

    pub fn trace(&self) -> Option<&str> {
        self.trace.as_ref().map(|s| &s[..])
    }
}
