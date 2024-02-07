use std::io;
use std::path::Path;
use std::process::Stdio;
use std::sync::Mutex;

use crate::memory::BackingStore;
use crate::repr::{Addr, Size};

#[derive(Debug)]
pub struct DriverProcess {
    size: u32,
    process: std::process::Child,
    stdin: Mutex<std::process::ChildStdin>,
    stdout: Mutex<std::process::ChildStdout>,
    stderr: Mutex<std::process::ChildStderr>,
}

impl Clone for DriverProcess {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
enum ProcessCommand {
    Initialize,
    Ack,
    Size(Size),
    Get(Addr),
    Set(Addr, u32),
    Result(u32),
    Error(String),
}

impl DriverProcess {
    pub fn new(size: u32, path: &Path) -> io::Result<Self> {
        // TODO: Use args
        // TODO: Make shell configurable
        // TODO: Remove unwrap
        let mut process = std::process::Command::new("/bin/sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["-c", &path.to_str().unwrap()])
            .spawn()?;
        let mut stdin = process.stdin.take().ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Unable to grab ProcessDriver STDIN",
        ))?;
        let mut stdout = process.stdout.take().ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Unable to grab ProcessDriver STDOUT",
        ))?;
        let stderr = process.stderr.take().ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Unable to grab ProcessDriver STDERR",
        ))?;

        ProcessCommand::Initialize.serialize(&mut stdin)?;
        let ack = ProcessCommand::deserialize(&mut stdout)?;
        assert!(matches!(ack, ProcessCommand::Ack));
        ProcessCommand::Size(size.into()).serialize(&mut stdin)?;
        assert!(matches!(
            ProcessCommand::deserialize(&mut stdout)?,
            ProcessCommand::Ack
        ));

        let stdin = Mutex::new(stdin);
        let stdout = Mutex::new(stdout);
        let stderr = Mutex::new(stderr);

        Ok(DriverProcess {
            size,
            process,
            stdin,
            stdout,
            stderr,
        })
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

impl ProcessCommand {
    const fn to_tag(&self) -> u16 {
        match self {
            Self::Initialize => 0,
            Self::Ack => 1,
            Self::Size(..) => 2,
            Self::Get(..) => 8,
            Self::Set(..) => 9,
            Self::Result(..) => 10,
            Self::Error(..) => 256,
        }
    }

    fn deserialize<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut tag_buffer = [0u8; 2];
        reader.read_exact(&mut tag_buffer)?;

        let tag = u16::from_be_bytes(tag_buffer);

        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::Ack,
            2 => {
                let mut size_buffer = [0u8; 4];
                reader.read_exact(&mut size_buffer)?;
                let size = u32::from_be_bytes(size_buffer);

                Self::Size(size.into())
            }
            8 => {
                let mut addr_buffer = [0u8; 4];
                reader.read_exact(&mut addr_buffer)?;
                let addr = u32::from_be_bytes(addr_buffer);

                Self::Get(addr.into())
            }
            9 => {
                let mut addr_buffer = [0u8; 4];
                reader.read_exact(&mut addr_buffer)?;
                let addr = u32::from_be_bytes(addr_buffer);

                let mut value_buffer = [0u8; 4];
                reader.read_exact(&mut value_buffer)?;
                let value = u32::from_be_bytes(value_buffer);

                Self::Set(addr.into(), value)
            }
            10 => {
                let mut value_buffer = [0u8; 4];
                reader.read_exact(&mut value_buffer)?;
                let value = u32::from_be_bytes(value_buffer);

                Self::Result(value)
            }
            256 => {
                let mut size_buffer = [0u8; 4];
                reader.read_exact(&mut size_buffer)?;
                let size = u32::from_be_bytes(size_buffer);

                let mut string_buffer = vec![0; size as usize];
                reader.read_exact(&mut string_buffer)?;
                let string = String::from_utf8(string_buffer).unwrap();

                Self::Error(string)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid ProcessCommand Tag",
                ))
            }
        })
    }

    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let tag = self.to_tag();

        writer.write_all(&tag.to_be_bytes())?;

        match self {
            Self::Initialize | Self::Ack => {}
            Self::Size(size) => {
                writer.write_all(&size.to_be_bytes())?;
            }
            Self::Get(addr) => {
                writer.write_all(&addr.to_be_bytes())?;
            }
            Self::Set(addr, value) => {
                writer.write_all(&addr.to_be_bytes())?;
                writer.write_all(&value.to_be_bytes())?;
            }
            Self::Result(value) => {
                writer.write_all(&value.to_be_bytes())?;
            }
            Self::Error(string) => {
                writer.write_all(&(string.len() as u32).to_be_bytes())?;
                writer.write_all(string.as_bytes())?;
            }
        }

        writer.flush()?;

        Ok(())
    }
}

impl BackingStore for DriverProcess {
    fn get(&self, at: Addr) -> u32 {
        let mut stdin = self.stdin.lock().unwrap();
        ProcessCommand::Get(at).serialize(&mut *stdin).unwrap();

        let mut stdout = self.stdout.lock().unwrap();
        assert!(matches!(ProcessCommand::deserialize(&mut *stdout).unwrap(), ProcessCommand::Ack));
        match ProcessCommand::deserialize(&mut *stdout).unwrap() {
            ProcessCommand::Result(result) => result,
            ProcessCommand::Error(err) => {
                eprintln!("Received Process Communication error: {err}");
                std::process::exit(1);
            }
            cmd => {
                eprintln!("Weird message received: {cmd:?}");
                std::process::exit(1);
            }
        }
    }

    fn set(&mut self, at: Addr, value: u32) {
        let mut stdin = self.stdin.lock().unwrap();
        ProcessCommand::Set(at, value)
            .serialize(&mut *stdin)
            .unwrap();

        let mut stdout = self.stdout.lock().unwrap();
        assert!(matches!(ProcessCommand::deserialize(&mut *stdout).unwrap(), ProcessCommand::Ack));
    }
}
