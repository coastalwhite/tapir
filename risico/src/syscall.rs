use std::io::{stdin, stdout, Read, Write};

use crate::csr::mtvec::TrapCause;
use crate::execute::Registers;
use crate::memory::BackingStore;
use crate::repr::Addr;

use rvhwfuzzer_encoding::XRegIdent;

pub enum SystemCallResult {
    Return(Option<u32>),
    InvalidSystemCallNr,
    Exit(i32),
    Jump(Addr),
    Abort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ECallBehavior {
    TrapVector,
    Linux,
    Testing,
}

pub trait SystemCallConvention {
    type Args;

    fn get_syscall_number(regs: &Registers) -> u32 {
        regs.get(XRegIdent::A7).as_u32()
    }
    fn get_args(regs: &Registers) -> Self::Args;
    fn handle<M: BackingStore>(
        syscall_number: u32,
        args: Self::Args,
        regs: &mut Registers,
        memory: &mut M,
    ) -> SystemCallResult;
    fn write_result(regs: &mut Registers, result: u32) {
        regs.set(XRegIdent::A0, result);
    }
}

struct TestingConvention;
struct LinuxConvention;

enum AssertType {
    Unknown = 0,
    U32 = 1,
    I32 = 2,
    F32 = 3,
    Ptr = 4,
    Invalid,
}

impl From<u32> for AssertType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::U32,
            2 => Self::I32,
            3 => Self::F32,
            3 => Self::Ptr,
            _ => Self::Invalid,
        }
    }
}

impl SystemCallConvention for TestingConvention {
    type Args = [u32; 6];

    fn get_args(regs: &Registers) -> Self::Args {
        [
            regs.get(XRegIdent::A0).as_u32(),
            regs.get(XRegIdent::A1).as_u32(),
            regs.get(XRegIdent::A2).as_u32(),
            regs.get(XRegIdent::A3).as_u32(),
            regs.get(XRegIdent::A4).as_u32(),
            regs.get(XRegIdent::A5).as_u32(),
        ]
    }
    fn handle<M: BackingStore>(
        syscall_number: u32,
        args: Self::Args,
        _regs: &mut Registers,
        memory: &mut M,
    ) -> SystemCallResult {
        match syscall_number {
            // Exit
            0 => SystemCallResult::Exit(args[0] as i32),
            // Write Register
            1 => {
                println!("{:08x}", args[0]);
                SystemCallResult::Return(None)
            }
            // Memory to Stdout
            2 => {
                let start = args[0];
                let end = start + args[1];

                for addr in start..end {
                    todo!()
                    // let value = memory.get_without_cache((addr & 0xFFFF_FFFC).into());
                    // let value = (value >> ((3 - (addr & 0x3)) * 8)) as u8;

                    // println!("{value:02x}");
                }
                SystemCallResult::Return(None)
            }
            // Assert Zero Register
            3 => {
                if args[0] == 0 {
                    SystemCallResult::Return(None)
                } else {
                    SystemCallResult::Abort
                }
            }
            // Assert Eq Register
            4 => {
                let input_type = AssertType::from(args[0]);

                let lhs = args[1];
                let rhs = args[2];

                if lhs != rhs {
                    match input_type {
                        AssertType::Unknown => {
                            eprintln!("Assert failed (unknown type): {lhs} != {rhs}");
                        }
                        AssertType::U32 => {
                            eprintln!(
                                "Assert failed (u32): {lhs} (0x{lhs:08x}) != {rhs} (0x{rhs:08x})"
                            );
                        }
                        AssertType::I32 => {
                            eprintln!("Assert failed (i32): {} != {}", lhs as i32, rhs as i32);
                        }
                        AssertType::F32 => {
                            eprintln!(
                                "Assert failed (f32): {} (0x{lhs:08X}) != {} (0x{rhs:08X})",
                                f32::from_bits(lhs),
                                f32::from_bits(rhs)
                            );
                        }
                        AssertType::Ptr => {
                            eprintln!("Assert failed (ptr): 0x{lhs:08X}) != 0x{rhs:08X}");
                        }
                        AssertType::Invalid => {
                            eprintln!("Assert failed (invalid type): {lhs:08X} != {rhs:08X}");
                        }
                    }
                    SystemCallResult::Abort
                } else {
                    SystemCallResult::Return(None)
                }
            }
            // Panic
            5 => {
                if args[0] == 0 {
                    eprintln!("Code panicked at unknown location");
                    return SystemCallResult::Abort;
                }

                let filename = args[0];
                let filename_len = args[1];
                let line = args[2];

                let filename =
                    String::from_utf8(memory.read_from(filename.into(), filename_len.into()))
                        .unwrap();

                eprintln!("Code panicked at {filename}:{line}");
                SystemCallResult::Abort
            }
            _ => SystemCallResult::InvalidSystemCallNr,
        }
    }
}

impl SystemCallConvention for LinuxConvention {
    type Args = [u32; 6];

    fn get_args(regs: &Registers) -> Self::Args {
        [
            regs.get(XRegIdent::A0).as_u32(),
            regs.get(XRegIdent::A1).as_u32(),
            regs.get(XRegIdent::A2).as_u32(),
            regs.get(XRegIdent::A3).as_u32(),
            regs.get(XRegIdent::A4).as_u32(),
            regs.get(XRegIdent::A5).as_u32(),
        ]
    }
    fn handle<M: BackingStore>(
        syscall_number: u32,
        args: Self::Args,
        _regs: &mut Registers,
        memory: &mut M,
    ) -> SystemCallResult {
        match syscall_number {
            0..=62 => unimplemented!(),
            63 => {
                // Read System Call
                let fd = args[0] as i32;
                let buf = args[1];
                let count = args[2];

                // fd = STDIN
                if fd != 0 {
                    unimplemented!()
                }

                if count == 0 {
                    return SystemCallResult::Return(Some(0));
                }

                let mut tmp_buffer = vec![0; count as usize];
                let num_read_bytes = stdin().lock().read(&mut tmp_buffer).unwrap_or(0);

                memory.write_to(buf.into(), &tmp_buffer[..num_read_bytes]);

                SystemCallResult::Return(Some(num_read_bytes as u32))
            }
            64 => {
                // Write System Call
                let fd = args[0] as i32;
                let buf = args[1];
                let count = args[2];

                // fd = STDOUT
                if !(fd == 1 || fd == 2) {
                    unimplemented!();
                }

                if count == 0 {
                    return SystemCallResult::Return(Some(0));
                }

                let buf = memory.read_from(buf.into(), count.into());

                let num_written_bytes = stdout().lock().write(&buf).unwrap_or(0);

                SystemCallResult::Return(Some(num_written_bytes as u32))
            }
            65..=92 => unimplemented!(),
            93 => {
                // exit
                let error_code = args[0];
                dbg!(error_code);
                SystemCallResult::Exit(error_code as i32)
            }
            94..=440 => unimplemented!(),
            _ => SystemCallResult::InvalidSystemCallNr,
        }
    }
}

impl ECallBehavior {
    pub fn handle<M: BackingStore>(self, regs: &mut Registers, memory: &mut M) -> SystemCallResult {
        match self {
            Self::Testing => {
                let syscall_nr = TestingConvention::get_syscall_number(regs);
                let args = TestingConvention::get_args(regs);
                let result = TestingConvention::handle(syscall_nr, args, regs, memory);

                if let SystemCallResult::Return(Some(result)) = result {
                    LinuxConvention::write_result(regs, result);
                }

                result
            }
            Self::Linux => {
                let syscall_nr = LinuxConvention::get_syscall_number(regs);
                let args = LinuxConvention::get_args(regs);
                let result = LinuxConvention::handle(syscall_nr, args, regs, memory);

                if let SystemCallResult::Return(Some(result)) = result {
                    LinuxConvention::write_result(regs, result);
                }

                result
            }
            Self::TrapVector => {
                let cause_addr = regs
                    .csr()
                    .mtvec
                    .cause_addr(TrapCause::EcallMmode)
                    .expect("Invalid MTVEC");
                SystemCallResult::Jump(Addr::from(cause_addr))
            }
        }
    }
}
