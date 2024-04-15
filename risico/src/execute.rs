use std::cell::Cell;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::process::exit;

use rsoftfloat::f32::F32;
use rvhwfuzzer_encoding::{FRegIdent, XRegIdent};
use rvisa::{MIsaExt, MIsa};

use crate::csr::fcsr::ExceptionFlags;
use crate::csr::mstatus::ExtStatus;
use crate::csr::mtvec::TrapCause;
use crate::csr::{ControlStatusRegisters, CsrWriteContext, Mode};
use crate::device_config::Isa;
use crate::driver::cache::CacheResult;
use crate::memory::{BackingStore, Endianness, MappedMemory};
use crate::repr::{Addr, Offset, Size, Word};
use crate::syscall::{ECallBehavior, SystemCallResult};
use crate::trap::TrapBehavior;
use crate::util::{is_signaling_nan, sign_extend};

#[repr(u8)]
enum RoundingMode {
    RoundToNearestTiesToEven = 0b000,
    RoundToZero = 0b001,
    RoundDown = 0b010,
    RoundUp = 0b0011,
    RoundToNearestTiesToMaxMagnitude = 0b0100,
    Dynamic = 0b0111,
}

impl TryFrom<u8> for RoundingMode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0b000 => Self::RoundToNearestTiesToEven,
            0b001 => Self::RoundToZero,
            0b010 => Self::RoundDown,
            0b011 => Self::RoundUp,
            0b100 => Self::RoundToNearestTiesToMaxMagnitude,
            0b111 => Self::Dynamic,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Registers {
    fregs: [f32; 32],
    xregs: [Word; 31],
    pc: Addr,
    csr: ControlStatusRegisters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    Machine,
    User,
}

impl From<PrivilegeLevel> for Mode {
    fn from(value: PrivilegeLevel) -> Self {
        match value {
            PrivilegeLevel::Machine => Mode::Machine,
            PrivilegeLevel::User => Mode::User,
        }
    }
}

impl TryFrom<Mode> for PrivilegeLevel {
    type Error = ();

    fn try_from(value: Mode) -> Result<Self, Self::Error> {
        match value {
            Mode::User => Ok(Self::User),
            Mode::Supervisor => Err(()),
            Mode::Reserved10 => Err(()),
            Mode::Machine => Ok(Self::Machine),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StateECallBehavior {
    pub machine: ECallBehavior,
    pub user: ECallBehavior,
}

#[derive(Clone)]
pub struct State<M: BackingStore> {
    isa: rvisa::MIsa,
    privilege: PrivilegeLevel,
    trap_behavior: TrapBehavior,
    ecall_behavior: StateECallBehavior,
    registers: Registers,
    memory: M,
    /// Host Target Interface
    ///
    /// Originally form the `riscv-fesvr`, now used by Spike to communicate with the host.
    htif: Option<Addr>,
    instruction_counter: u64,
}

impl Registers {
    #[inline]
    pub fn get_pc(&self) -> Word {
        self.pc.into()
    }

    #[inline]
    pub fn set_pc(&mut self, value: impl Into<Word>) -> Word {
        let old = self.pc;
        self.pc = value.into().as_addr();
        old.into()
    }

    #[inline]
    pub fn get(&self, ident: XRegIdent) -> Word {
        if ident == XRegIdent::Zero {
            return Word::default();
        }

        self.xregs[ident as usize - 1]
    }

    #[inline]
    pub fn set(&mut self, ident: XRegIdent, value: impl Into<Word>) -> Word {
        if ident == XRegIdent::Zero {
            return Word::default();
        }

        let old = self.xregs[ident as usize - 1];
        self.xregs[ident as usize - 1] = value.into();
        old
    }

    #[inline]
    pub fn get_freg(&self, ident: FRegIdent) -> f32 {
        self.fregs[ident as usize]
    }

    #[inline]
    pub fn set_freg(&mut self, ident: FRegIdent, value: f32) -> f32 {
        let old = self.fregs[ident as usize];
        self.fregs[ident as usize] = value;
        old
    }

    #[inline]
    pub fn csr(&self) -> &ControlStatusRegisters {
        &self.csr
    }
}

impl<M: BackingStore> Debug for State<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- START State Debug ---")?;
        // if self.pc().is_word_aligned() {
        //     let hex = self.memory.get(self.pc());
        //     let instruction = Instruction::try_from(hex)
        //         .map_or("<???>".to_string(), |instruction| instruction.to_string());
        //     writeln!(f, "instr:\t{instruction}\t(= 0x{hex:08x})")?;
        // }
        writeln!(f, "pc:\t{}\t(= 0x{:08x})", self.pc(), self.pc())?;
        writeln!(f, "")?;
        writeln!(f, "GP-Registers:")?;
        for i in 0..16 {
            let lhs_ident = XRegIdent::take_masked(i);
            let rhs_ident = XRegIdent::take_masked(i + 16);

            let lhs = self.registers.get(lhs_ident).as_i32();
            let rhs = self.registers.get(rhs_ident).as_i32();

            writeln!(
                f,
                "{lhs_ident}:\t{:+011} (0x{:08x})\t{rhs_ident}:\t{:+011} (0x{:08x})",
                lhs, lhs, rhs, rhs
            )?;
        }

        Ok(())
    }
}

impl<M: BackingStore> State<M> {
    pub fn new(
        isa: MIsa,
        ecall_behavior: StateECallBehavior,
        trap_behavior: TrapBehavior,
        entry: u32,
        memory: M,
        htif: Option<Addr>,
    ) -> Self {
        let registers = Registers {
            pc: entry.into(),
            xregs: [0.into(); 31],
            fregs: [0.; 32],
            csr: ControlStatusRegisters::new(&crate::csr::CsrInitContext {
                isa,
            }),
        };

        State {
            registers,
            isa,
            privilege: PrivilegeLevel::Machine,
            ecall_behavior,
            trap_behavior,
            memory,
            instruction_counter: 0,
            htif,
        }
    }

    pub fn pc(&self) -> Addr {
        self.registers.pc
    }

    pub fn get_rounding_mode(&mut self, rm: u8) -> rsoftfloat::RoundingMode {
        // Dynamic rounding mode
        let rm = if rm == 0b111 {
            self.registers.csr.fcsr.frm_read() as u8
        } else {
            rm
        };

        use rsoftfloat::RoundingMode as R;
        match rm {
            0b000 => R::TiesToEven,
            0b001 => R::ToZero,
            0b010 => R::Down,
            0b011 => R::Up,
            0b100 => R::TiesToMaxMagnitude,

            // @TODO: properly handle error
            _ => {
                eprintln!("rm = {rm:03b}");
                unimplemented!()
            }
        }
    }

    pub fn are_f_ext_instrs_available(&self) -> bool {
        self.registers()
            .csr
            .misa
            .isa()
            .contains(MIsaExt::SINGLE_PRECISION_FP)
            && self.registers().csr.mstatus.fs() != ExtStatus::Off
    }

    pub fn are_c_ext_instrs_available(&self) -> bool {
        self.registers()
            .csr
            .misa
            .isa()
            .contains(MIsaExt::COMPRESSED)
    }

    pub fn execute(mut self) -> Self {
        self.execute_mut();
        self
    }

    pub fn htif_to_host(&self, value: u32) {
        // Adapted from:
        // - https://github.com/riscv-software-src/riscv-isa-sim/issues/364#issuecomment-607657754

        let is_exit = value & 1 != 0;
        if is_exit {
            let return_code = value >> 1;
            std::process::exit(return_code as i32);
        } else {
            unimplemented!()
        }
    }

    pub fn current_privilege(&self) -> PrivilegeLevel {
        self.privilege
    }

    pub fn current_ecall_behavior(&self) -> ECallBehavior {
        match self.current_privilege() {
            PrivilegeLevel::Machine => self.ecall_behavior.machine,
            PrivilegeLevel::User => self.ecall_behavior.user,
        }
    }

    pub fn get_datamem_endianness(&self) -> Endianness {
        let mstatus = &self.registers().csr().mstatus;

        match self.current_privilege() {
            PrivilegeLevel::User => mstatus.ube(),
            // PrivilegeLevel::Supervisor => mstatus.sbe(),
            PrivilegeLevel::Machine => mstatus.mbe(),
        }
    }

    pub fn fetch_datamem_word(&self, at: Addr) -> Word {
        let endianness = self.get_datamem_endianness();
        self.memory().get(at, endianness).into()
    }

    pub fn fetch_datamem_halfword(&self, at: Addr) -> u16 {
        let endianness = self.get_datamem_endianness();
        self.memory().get_halfword(at, endianness)
    }

    pub fn fetch_datamem_byte(&self, at: Addr) -> u8 {
        self.memory().get_byte(at)
    }

    pub fn store_datamem_word(&mut self, at: Addr, value: impl Into<Word>) {
        let value = value.into();

        if self
            .htif
            .is_some_and(|htif_addr| htif_addr == at || htif_addr.offset(4) == at)
        {
            self.htif_to_host(value.as_u32());
            return;
        }

        let endianness = self.get_datamem_endianness();
        self.memory_mut().set(at, value.as_u32(), endianness)
    }

    pub fn store_datamem_halfword(&mut self, at: Addr, value: u16) {
        let endianness = self.get_datamem_endianness();
        self.memory_mut().set_halfword(at, value, endianness)
    }

    pub fn store_datamem_byte(&mut self, at: Addr, value: u8) {
        self.memory_mut().set_byte(at, value)
    }

    fn csr_write_context(&self) -> CsrWriteContext {
        CsrWriteContext {
            pc: self.registers().get_pc().as_addr(),
        }
    }

    pub fn trigger_trap(&mut self, trap: TrapCause) {
        if self
            .trap_behavior
            .contains(TrapBehavior::abort_on_trap(trap))
        {
            let instruction_mem = self.instruction_fetch();
            let instruction = self.instruction_decode(instruction_mem);
            let instruction = instruction
                .map_or(format!("<unknown 0x{instruction_mem:08X}>"), |i| {
                    format!("{i}")
                });

            eprintln!(
                "[ABORT][PC=0x{:08X}]: Trap '{trap:?}' on '{instruction}'",
                self.pc(),
            );
            std::process::exit(1);
        }

        eprintln!("[PC={:08X}]: Trap triggered {trap:?}", self.pc().as_u32(),);

        let ctx = self.csr_write_context();

        self.registers
            .csr
            .mstatus
            .trap_to_machine(self.current_privilege().into());
        self.registers.csr.mepc.write(self.pc().as_u32(), &ctx);
        self.registers.csr.mcause.write(trap as u32, &ctx);
        let addr = self.registers.csr.mtvec.cause_addr(trap).unwrap();

        self.registers_mut().csr.cycle.increment();
        self.registers_mut().csr.instret.increment();
        self.registers_mut().csr.time.increment();

        self.registers.pc = Addr::from(addr);
    }

    pub fn instruction_fetch(&mut self) -> Word {
        // @Note: According to the RISC-V Privileged Specification, we don't have to take into
        // account the MBE, SBE or UBE here.
        //
        // > The MBE, SBE, and UBE bits in mstatus and mstatush are WARL fields that control the
        // > endianness of memory accesses other than instruction fetches. Instruction fetches are
        // > always little- endian.
        Word::from(self.memory.get_le_bytes(self.pc()))
    }

    pub fn instruction_decode(
        &mut self,
        imemory: Word,
    ) -> Option<rvhwfuzzer_encoding::Instruction> {
        let mut imemory_bytes = &imemory.to_le_bytes()[..];
        let Some(instruction) =
            rvhwfuzzer_encoding::Instruction::decode(&mut imemory_bytes).unwrap()
        else {
            return None;
        };

        Some(instruction)
    }

    pub fn instruction_execute(&mut self, instruction: rvhwfuzzer_encoding::Instruction) {
        use rvhwfuzzer_encoding::Instruction::*;

        let mut next_pc = self.pc().offset(instruction.num_bytes() as i8);

        // println!("[PC={:08X}]: {}", self.pc(), &instruction);

        match instruction {
            Lui(args) => {
                self.instruction_counter += 1;
                self.registers.set(args.rd(), args.imm());
            }
            Auipc(args) => {
                self.instruction_counter += 1;

                let rd = args.rd();
                let offset = Offset::from(args.imm() as i32);

                self.registers.set(rd, self.pc().offset(offset));
            }
            Jal(args) => {
                self.instruction_counter += 1;

                let rd = args.rd();

                self.registers.set(rd, self.pc().offset(4));
                next_pc = self.pc().offset(args.imm());
            }
            Jalr(args) => {
                self.instruction_counter += 1;

                let rd = args.rd();
                let rs = args.rs1();

                let rs1 = self.registers.get(rs).as_addr();
                self.registers.set(rd, self.pc().offset(4));
                next_pc = rs1.offset(args.imm()).halfword_align();
            }
            Beq(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                if rs1 == rs2 {
                    next_pc = self.pc().offset(args.imm());
                }
            }
            Bne(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                if rs1 != rs2 {
                    next_pc = self.pc().offset(args.imm());
                }
            }
            Blt(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                if rs1.as_i32() < rs2.as_i32() {
                    next_pc = self.pc().offset(args.imm());
                }
            }
            Bge(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                if rs1.as_i32() >= rs2.as_i32() {
                    next_pc = self.pc().offset(args.imm());
                }
            }
            Bltu(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                if rs1.as_u32() < rs2.as_u32() {
                    next_pc = self.pc().offset(args.imm());
                }
            }
            Bgeu(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                if rs1.as_u32() >= rs2.as_u32() {
                    next_pc = self.pc().offset(args.imm());
                }
            }
            Lb(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.fetch_datamem_byte(src);
                let mem = Word::sign_extend_u8(mem);

                self.registers.set(rd, mem);
            }
            Lh(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.fetch_datamem_halfword(src);
                let mem = Word::sign_extend_u16(mem);

                self.registers.set(rd, mem);
            }
            Lw(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.fetch_datamem_word(src);

                self.registers.set(rd, mem);
            }
            Lbu(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.fetch_datamem_byte(src);
                let mem = Word::from(mem);

                self.registers.set(rd, mem);
            }
            Lhu(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.fetch_datamem_halfword(src);
                let mem = Word::from(mem);

                self.registers.set(rd, mem);
            }
            Sb(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1).as_addr();
                let rs2 = self.registers.get(rs2).to_le_bytes()[0];
                let dest = rs1.offset(args.imm());

                self.store_datamem_byte(dest, rs2);
            }
            Sh(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1).as_addr();
                let rs2 = self.registers.get(rs2).to_le_halfwords()[0];
                let dest = rs1.offset(args.imm());

                self.store_datamem_halfword(dest, rs2);
            }
            Sw(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1).as_addr();
                let rs2 = self.registers.get(rs2).as_u32();
                let dest = rs1.offset(args.imm());

                self.store_datamem_word(dest, rs2);
            }
            Addi(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers
                    .set(rd, rs.as_i32().wrapping_add(args.imm().into()));
            }
            Slti(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers
                    .set(rd, u32::from(rs.as_i32() < args.imm().into()));
            }
            Sltiu(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers
                    .set(rd, u32::from(rs.as_u32() < args.imm().into()));
            }
            Xori(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_i32() ^ i32::from(args.imm()));
            }
            Andi(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_i32() & i32::from(args.imm()));
            }
            Ori(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_i32() | i32::from(args.imm()));
            }
            Slli(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_u32() << args.shamt());
            }
            Srli(args) => {
                let rd = args.rd();
                let rs = args.rs1();
                let shamt = args.shamt();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_u32() >> shamt);
            }
            Srai(args) => {
                let rd = args.rd();
                let rs = args.rs1();
                let shamt = args.shamt();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_i32() >> shamt);
            }
            Add(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd, rs1 + rs2);
            }
            Sub(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd, rs1 - rs2);
            }
            Sll(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd, rs1 << rs2.as_shift());
            }
            Srl(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd, rs1 >> rs2.as_shift());
            }
            Sra(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers
                    .set(rd, rs1.as_i32() >> rs2.as_shift().as_usize());
            }
            Slt(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers
                    .set(rd, u32::from(rs1.as_i32() < rs2.as_i32()));
            }
            Sltu(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers
                    .set(rd, u32::from(rs1.as_u32() < rs2.as_u32()));
            }
            Xor(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd, rs1.as_u32() ^ rs2.as_u32());
            }
            And(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd, rs1.as_u32() & rs2.as_u32());
            }
            Or(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd, rs1.as_u32() | rs2.as_u32());
            }
            FmaddS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();
                let rs3 = args.rs3();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);
                let rs3 = self.registers.get_freg(rs3);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);
                let rs3 = F32::from_f32(rs3);

                let (result, flags) = F32::fmadd(rs1, rs2, rs3, rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FnmsubS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();
                let rs3 = args.rs3();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);
                let rs3 = self.registers.get_freg(rs3);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);
                let rs3 = F32::from_f32(rs3);

                let (result, flags) = F32::fnmsub(rs1, rs2, rs3, rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FmsubS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();
                let rs3 = args.rs3();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);
                let rs3 = self.registers.get_freg(rs3);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);
                let rs3 = F32::from_f32(rs3);

                let (result, flags) = F32::fmsub(rs1, rs2, rs3, rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FnmaddS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();
                let rs3 = args.rs3();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);
                let rs3 = self.registers.get_freg(rs3);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);
                let rs3 = F32::from_f32(rs3);

                let (result, flags) = F32::fnmadd(rs1, rs2, rs3, rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            Fsw(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs1 = args.rs1();
                let rs2 = args.rs2();
                let imm = args.imm();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let dest = rs1.as_addr().offset(imm);

                self.store_datamem_word(dest, rs2.to_bits());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            Flw(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let imm = args.imm();

                let rs1 = self.registers.get(rs1);

                let src = rs1.as_addr().offset(imm);
                let value = self.fetch_datamem_word(src);

                self.registers.set_freg(rd, value.as_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FmulS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = F32::mul(rs1, rs2, rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FdivS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = F32::div(rs1, rs2, rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FaddS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = F32::add(rs1, rs2, rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FsubS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = F32::sub(rs1, rs2, rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FltS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = F32::lt(rs1, rs2);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set(rd, u32::from(result));
                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FminS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = rs1.min(rs2);

                self.registers.csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FmaxS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = rs1.max(rs2);

                self.registers.csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FsgnjS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set_freg(rd, rs1.copysign(rs2));

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FsgnjnS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set_freg(rd, rs1.copysign(-rs2));

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FsgnjxS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let xor = f32::from_bits(rs1.to_bits() ^ rs2.to_bits());

                self.registers.set_freg(rd, rs1.copysign(xor));

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FleS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = F32::le(rs1, rs2);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set(rd, u32::from(result));

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FeqS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                let (result, flags) = rs1.eq(rs2);

                self.registers.csr.fcsr.add_fflags(flags);
                self.registers.set(rd, u32::from(result));

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FcvtSW(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get(rs1);

                let (result, flags) = F32::from_i32(rs1.as_i32(), rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FcvtWS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);

                let rs1 = F32::from_f32(rs1);

                let (result, flags) = rs1.to_i32(rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set(rd, result);

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FsqrtS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs1 = F32::from_f32(rs1);

                let (result, flags) = rs1.sqrt(rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FcvtSWu(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get(rs1);

                let (result, flags) = F32::from_u32(rs1.as_u32(), rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set_freg(rd, result.to_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FcvtWuS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);

                let rs1 = F32::from_f32(rs1);

                let (result, flags) = rs1.to_u32(rm);

                self.registers_mut().csr.fcsr.add_fflags(flags);
                self.registers.set(rd, result);

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FclassS(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();

                let rs1 = self.registers.get_freg(rs1);

                let rs1 = F32::from_f32(rs1);

                let class = rs1.class();
                let class = 1u32 << (class as u32);

                self.registers.set(rd, class);
            }
            FmvWX(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();

                let rs1 = self.registers.get(rs1);

                self.registers.set_freg(rd, rs1.as_f32());

                self.registers_mut().csr.mstatus.mark_fs_dirty();
            }
            FmvXW(args) => {
                if !self.are_f_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();

                let rs1 = self.registers.get_freg(rs1);

                self.registers.set(rd, Word::from_f32(rs1));
            }
            Csrrw(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers().get(rs);

                let ctx = self.csr_write_context();

                let Ok(value) = self.registers.csr.write(csr, rs.as_u32(), &ctx) else {
                    if self.trap_behavior.does_abort_on_missing_csr() {
                        let csr = csr.0;
                        eprintln!("[ERROR][CSRRW] Missing CSR: {csr} (0x{csr:03x}) ");
                        std::process::exit(1);
                    }
                    return self.trigger_trap(TrapCause::IllegalInstruction);
                };

                self.registers.set(rd, value);
            }
            Csrrs(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let rs = args.rs1();

                let value = if rs == XRegIdent::Zero {
                    self.registers.csr.read(csr)
                } else {
                    let ctx = self.csr_write_context();

                    let rs = self.registers().get(rs);
                    self.registers.csr.update(csr, |v| v | rs.as_u32(), &ctx)
                };

                let Ok(value) = value else {
                    if self.trap_behavior.does_abort_on_missing_csr() {
                        let csr = csr.0;
                        eprintln!("[ERROR][CSRRS] Missing CSR: {csr} (0x{csr:03x}) ");
                        std::process::exit(1);
                    }
                    return self.trigger_trap(TrapCause::IllegalInstruction);
                };

                self.registers.set(rd, value);
            }
            Csrrc(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let rs = args.rs1();

                let value = if rs == XRegIdent::Zero {
                    self.registers.csr.read(csr)
                } else {
                    let ctx = self.csr_write_context();

                    let rs = self.registers().get(rs);
                    self.registers.csr.update(csr, |v| v & !rs.as_u32(), &ctx)
                };

                let Ok(value) = value else {
                    if self.trap_behavior.does_abort_on_missing_csr() {
                        let csr = csr.0;
                        eprintln!("[ERROR][CSRRC] Missing CSR: {csr} (0x{csr:03x}) ");
                        std::process::exit(1);
                    }
                    return self.trigger_trap(TrapCause::IllegalInstruction);
                };

                self.registers.set(rd, value);
            }
            Csrrwi(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let uimm = args.uimm();

                let ctx = self.csr_write_context();

                let Ok(value) = self.registers.csr.write(csr, uimm.into(), &ctx) else {
                    if self.trap_behavior.does_abort_on_missing_csr() {
                        let csr = csr.0;
                        eprintln!("[ERROR][CSRRWI] Missing CSR: {csr} (0x{csr:03x}) ");
                        std::process::exit(1);
                    }
                    return self.trigger_trap(TrapCause::IllegalInstruction);
                };

                self.registers.set(rd, value);
            }
            Csrrsi(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let uimm = args.uimm();

                let ctx = self.csr_write_context();

                let Ok(value) = self
                    .registers
                    .csr
                    .update(csr, |v| v | u32::from(uimm), &ctx)
                else {
                    if self.trap_behavior.does_abort_on_missing_csr() {
                        let csr = csr.0;
                        eprintln!("[ERROR][CSRRSI] Missing CSR: {csr} (0x{csr:03x}) ");
                        std::process::exit(1);
                    }
                    return self.trigger_trap(TrapCause::IllegalInstruction);
                };

                self.registers.set(rd, value);
            }
            Csrrci(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let uimm = args.uimm();

                let ctx = self.csr_write_context();

                let Ok(value) = self
                    .registers
                    .csr
                    .update(csr, |v| v & !u32::from(uimm), &ctx)
                else {
                    if self.trap_behavior.does_abort_on_missing_csr() {
                        let csr = csr.0;
                        eprintln!("[ERROR][CSRRCI] Missing CSR: {csr} (0x{csr:03x}) ");
                        std::process::exit(1);
                    }
                    return self.trigger_trap(TrapCause::IllegalInstruction);
                };

                self.registers.set(rd, value);
            }
            Ecall(_args) => {
                if self.current_ecall_behavior() == ECallBehavior::TrapVector {
                    self.trigger_trap(match self.current_privilege() {
                        PrivilegeLevel::Machine => TrapCause::EcallMmode,
                        PrivilegeLevel::User => TrapCause::EcallUmode,
                    });
                    return;
                }

                match self
                    .current_ecall_behavior()
                    .handle(&mut self.registers, &mut self.memory)
                {
                    SystemCallResult::Return(_) => {}
                    SystemCallResult::Jump(target) => {
                        unreachable!();
                    }
                    SystemCallResult::Exit(error_code) => {
                        std::process::exit(error_code);
                    }
                    SystemCallResult::Abort => {
                        println!("Process Aborted through System Call.");
                        std::process::exit(1);
                    }
                    SystemCallResult::InvalidSystemCallNr => {
                        println!("Invalid System Call was called.");
                        std::process::exit(1);
                    }
                }
            }
            Ebreak(_args) => {
                return self.trigger_trap(TrapCause::Breakpoint);
            }
            Fence(_) => {}
            FenceI(_) => {}
            MRet(_) => {
                next_pc = Addr::from(self.registers.csr.mepc.read());

                if !self.are_c_ext_instrs_available() {
                    next_pc = next_pc.word_align();
                }

                self.privilege = self
                    .registers
                    .csr
                    .mstatus
                    .return_from_machine_trap()
                    .try_into()
                    .unwrap();
            }
            Mul(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                self.registers
                    .set(rd, rs1.as_u32().wrapping_mul(rs2.as_u32()));
            }
            MulH(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                let rs1 = i64::from(rs1.as_i32());
                let rs2 = i64::from(rs2.as_i32());

                let result = (rs1.wrapping_mul(rs2) >> 32) as u32;

                self.registers.set(rd, result);
            }
            MulHsu(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                let rs1 = i64::from(rs1.as_i32()) as u64;
                let rs2 = u64::from(rs2.as_u32());

                let result = (rs1.wrapping_mul(rs2) >> 32) as u32;

                self.registers.set(rd, result);
            }
            MulHu(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                let rs1 = u64::from(rs1.as_u32());
                let rs2 = u64::from(rs2.as_u32());

                let result = (rs1.wrapping_mul(rs2) >> 32) as u32;

                self.registers.set(rd, result);
            }
            Div(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                let result = if rs2.is_zero() {
                    -1
                } else {
                    rs1.as_i32().checked_div(rs2.as_i32()).unwrap_or(i32::MIN)
                };

                self.registers.set(rd, result);
            }
            DivU(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                let result = if rs2.is_zero() {
                    u32::MAX
                } else {
                    rs1.as_u32().wrapping_div(rs2.as_u32())
                };

                self.registers.set(rd, result);
            }
            Rem(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                let result = if rs2.is_zero() {
                    rs1.as_i32()
                } else {
                    rs1.as_i32().checked_rem(rs2.as_i32()).unwrap_or(0)
                };

                self.registers.set(rd, result);
            }
            RemU(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get(rs2);

                let result = if rs2.is_zero() {
                    rs1.as_u32()
                } else {
                    rs1.as_u32().wrapping_rem(rs2.as_u32())
                };

                self.registers.set(rd, result);
            }
            CAddi4SpN(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd().into();
                let imm = args.imm();

                let sp = self.registers.get(XRegIdent::Sp);
                let sp = sp.as_u32().wrapping_add(imm);

                self.registers.set(rd, sp);
            }
            CLw(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let offset = i16::from(args.imm());

                let rs1 = self.registers().get(rs1.into());
                let addr = rs1.as_addr().offset(offset);

                let value = self.fetch_datamem_word(addr);
                self.registers.set(rd.into(), value);
            }
            CFlw(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs1 = args.rs1();
                let offset = i16::from(args.imm());

                let rs1 = self.registers().get(rs1.into());
                let addr = rs1.as_addr().offset(offset);

                let value = self.fetch_datamem_word(addr);
                self.registers.set_freg(rd.into(), value.as_f32());
            }
            CSw(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs1 = args.rs1();
                let rs2 = args.rs2();
                let offset = i16::from(args.imm());

                let rs1 = self.registers().get(rs1.into());
                let rs2 = self.registers().get(rs2.into());
                let addr = rs1.as_addr().offset(offset);

                self.store_datamem_word(addr, rs2.as_u32());
            }
            CFsw(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs1 = args.rs1();
                let rs2 = args.rs2();
                let offset = i16::from(args.imm());

                let rs1 = self.registers().get(rs1.into());
                let rs2 = self.registers().get_freg(rs2.into());
                let addr = rs1.as_addr().offset(offset);

                self.store_datamem_word(addr, rs2.to_bits());
            }
            CNop(_) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

            }
            CAddi(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1();
                let imm = args.imm();

                let rs1 = self.registers().get(rd_rs1);
                let rs1 = rs1.as_i32().wrapping_add(i32::from(imm));

                self.registers.set(rd_rs1, rs1);
            }
            CJal(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                self.registers.set(XRegIdent::Ra, self.pc().offset(2));
                next_pc = self.pc().offset(args.imm());
            }
            CLi(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let imm = args.imm();

                self.registers.set(rd, imm);
            }
            CAddi16Sp(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let imm = args.imm();

                let sp = self.registers.get(XRegIdent::Sp);
                let sp = sp.as_i32().wrapping_add(i32::from(imm));

                self.registers.set(XRegIdent::Sp, sp);
            }
            CLui(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let imm = args.imm();

                self.registers.set(rd, imm);
            }
            CSrli(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1().into();
                let imm = args.imm();

                let rs1 = self.registers.get(rd_rs1);
                let value = rs1.as_u32() >> imm;

                self.registers.set(rd_rs1, value);
            }
            CSrai(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1().into();
                let imm = args.imm();

                let rs1 = self.registers.get(rd_rs1);
                let value = rs1.as_i32() >> imm;

                self.registers.set(rd_rs1, value);
            }
            CAndi(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1().into();
                let imm = i32::from(args.imm());

                let rs1 = self.registers.get(rd_rs1);
                let value = rs1.as_i32() & imm;

                self.registers.set(rd_rs1, value);
            }
            CSub(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1().into();
                let rs2 = args.rs2().into();

                let rs1 = self.registers.get(rd_rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd_rs1, rs1 - rs2);
            }
            CXor(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1().into();
                let rs2 = args.rs2().into();

                let rs1 = self.registers.get(rd_rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd_rs1, rs1 ^ rs2);
            }
            COr(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1().into();
                let rs2 = args.rs2().into();

                let rs1 = self.registers.get(rd_rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd_rs1, rs1 | rs2);
            }
            CAnd(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1().into();
                let rs2 = args.rs2().into();

                let rs1 = self.registers.get(rd_rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd_rs1, rs1 & rs2);
            }
            CJ(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                next_pc = self.pc().offset(args.imm());
            }
            CBeqz(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs1 = args.rs1().into();

                let rs1 = self.registers.get(rs1);

                if rs1.is_zero() {
                    next_pc = self.pc().offset(args.imm());
                }
            }
            CBnez(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs1 = args.rs1().into();

                let rs1 = self.registers.get(rs1);

                if !rs1.is_zero() {
                    next_pc = self.pc().offset(args.imm());
                }
            }
            CSlli(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1();
                let imm = args.imm();

                let rs1 = self.registers.get(rd_rs1);
                let value = rs1.as_u32() << imm;

                self.registers.set(rd_rs1, value);
            }
            CLwSp(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let offset = i16::from(args.imm());

                let addr = self.registers.get(XRegIdent::Sp).as_addr().offset(offset);
                let value = self.fetch_datamem_word(addr);

                self.registers.set(rd, value);
            }
            CFlwSp(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let offset = i16::from(args.imm());

                let addr = self.registers.get(XRegIdent::Sp).as_addr().offset(offset);
                let value = self.fetch_datamem_word(addr);

                self.registers.set_freg(rd, value.as_f32());
            }
            CJr(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs = args.rs1();
                let rs1 = self.registers.get(rs).as_addr();
                next_pc = rs1.halfword_align();
            }
            CMv(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd = args.rd();
                let rs2 = args.rs2();

                let value = self.registers.get(rs2);
                self.registers.set(rd, value);
            }
            CEBreak(_) => todo!(),
            CJalr(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs = args.rs1();

                let rs1 = self.registers.get(rs).as_addr();
                self.registers.set(XRegIdent::Ra, self.pc().offset(2));
                next_pc = rs1.halfword_align();
            }
            CAdd(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rd_rs1 = args.rd_rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rd_rs1);
                let rs2 = self.registers.get(rs2);

                self.registers.set(rd_rs1, rs1 + rs2);
            }
            CSwSp(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs2 = args.rs2();
                let offset = args.imm();

                let addr = self
                    .registers
                    .get(XRegIdent::Sp)
                    .as_addr()
                    .offset(i16::from(offset));
                let value = self.registers.get(rs2);

                self.store_datamem_word(addr, value.as_u32());
            }
            CFswSp(args) => {
                if !self.are_c_ext_instrs_available() {
                    self.trigger_trap(TrapCause::IllegalInstruction);
                    return;
                }

                let rs2 = args.rs2();
                let offset = args.imm();

                let addr = self
                    .registers
                    .get(XRegIdent::Sp)
                    .as_addr()
                    .offset(i16::from(offset));
                let value = self.registers.get_freg(rs2);

                self.store_datamem_word(addr, value.to_bits());
            }
        };

        self.registers_mut().csr.cycle.increment();
        self.registers_mut().csr.instret.increment();
        self.registers_mut().csr.time.increment();

        self.registers.pc = next_pc;
    }

    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut M {
        &mut self.memory
    }

    pub fn take_memory(self) -> M {
        self.memory
    }

    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    pub fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    pub fn run_while<F>(&mut self, f: F)
    where
        F: Fn(&Self) -> bool,
    {
        while f(&self) {
            self.execute_mut();
        }
    }

    fn set_pc(&mut self, addr: Addr) {
        self.registers.set_pc(addr);
    }

    fn offset_pc(&mut self, offset: impl Into<Offset>) {
        self.registers.set_pc(self.pc().offset(offset));
    }

    pub fn rdcycle(&self) -> u32 {
        self.instruction_counter as u32
    }

    pub fn execute_mut(&mut self) {
        let imemory = self.instruction_fetch();
        let Some(instruction) = self.instruction_decode(imemory) else {
            eprintln!(
                "[PC={:08X}]: <unknown 0x{:08X}>",
                self.pc(),
                imemory.as_u32()
            );
            return self.trigger_trap(TrapCause::IllegalInstruction);
        };
        self.instruction_execute(instruction)
    }

    // fn environment_call(&mut self) {
    //     let syscall = self.registers.get(RegIdent::GeneralPurpose(17)).as_u32();
    //
    //     match syscall {
    //         0 => {
    //             // Print
    //             let addr = self.registers.get(RegIdent::GeneralPurpose(11)).as_addr();
    //             let len = self.registers.get(RegIdent::GeneralPurpose(12)).as_size();
    //
    //             let s = self.memory.read_from(addr, len);
    //             let s = String::from_utf8(s).expect("syscall with not utf8 string");
    //
    //             print!("{s}");
    //         }
    //         1 => {
    //             println!("Requested exit...");
    //             let exit_code = self.registers.get(RegIdent::GeneralPurpose(11)).as_i32();
    //             std::process::exit(exit_code);
    //         }
    //         // Read
    //         63 => {
    //             let fd = self.registers.get(RegIdent::GeneralPurpose(10)).as_u32();
    //             let addr = self.registers.get(RegIdent::GeneralPurpose(11)).as_u32();
    //             let length = self.registers.get(RegIdent::GeneralPurpose(12)).as_u32();
    //
    //             assert_eq!(fd, 0);
    //
    //             use std::io::stdin;
    //
    //             let mut buf = vec![0; length as usize];
    //             stdin()
    //                 .lock()
    //                 .read_exact(&mut buf)
    //                 .expect("Failed to read from STDIN");
    //             self.memory.write_to(Addr::from(addr), &buf);
    //         }
    //         // Write
    //         64 => {
    //             let fd = self.registers.get(RegIdent::GeneralPurpose(10)).as_u32();
    //             let addr = self.registers.get(RegIdent::GeneralPurpose(11)).as_u32();
    //             let length = self.registers.get(RegIdent::GeneralPurpose(12)).as_u32();
    //
    //             assert_eq!(fd, 1);
    //
    //             use std::io::stdout;
    //
    //             let buf = self.memory.read_from(Addr::from(addr), Size::from(length));
    //             stdout()
    //                 .lock()
    //                 .write(&buf)
    //                 .expect("Failed to write to STDOUT");
    //             stdout().flush().expect("Failed to flush");
    //         }
    //         2 => {
    //             let addr = self.registers.get(RegIdent::GeneralPurpose(11)).as_addr();
    //             let len = self.registers.get(RegIdent::GeneralPurpose(12)).as_size();
    //
    //             use std::io::{stdin, stdout};
    //             let mut s = String::new();
    //             let _ = stdout().flush();
    //             stdin()
    //                 .read_line(&mut s)
    //                 .expect("Did not enter a correct string");
    //             if let Some('\n') = s.chars().next_back() {
    //                 s.pop();
    //             }
    //             if let Some('\r') = s.chars().next_back() {
    //                 s.pop();
    //             }
    //
    //             let read_length = usize::min(len.as_usize(), s.len());
    //
    //             for i in 0..read_length {
    //                 let offset = Offset::from(i as i32);
    //                 self.memory.set_byte(addr.offset(offset), s.as_bytes()[i]);
    //             }
    //
    //             self.registers
    //                 .set(RegIdent::GeneralPurpose(10), read_length as u32);
    //         }
    //         _ => panic!("unknown syscall"),
    //     }
    // }

    fn environment_break(&mut self) {
        println!("{:?}", self);
        use std::io::{stdin, stdout, Write};
        let mut s = String::new();
        print!("Continue (Y/n): ");
        let _ = stdout().flush();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }
        if !(s.is_empty() | s.starts_with(&['y', 'Y'])) {
            exit(0);
        }
    }
}
