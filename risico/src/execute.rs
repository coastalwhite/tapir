use std::cell::Cell;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::process::exit;

use rvhwfuzzer_encoding::{XRegIdent, FRegIdent};
use ::softfloat_wrapper::{Float, F32};

use crate::csr::ControlStatusRegisters;
use crate::csr::fcsr::ExceptionFlags;
use crate::device_config::Isa;
use crate::driver::cache::CacheResult;
use crate::memory::{BackingStore, MappedMemory};
use crate::repr::{Addr, Offset, Size, Word};
use crate::syscall::{SystemCallBehavior, SystemCallResult};
use crate::util::sign_extend;

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

#[derive(Clone)]
pub struct State<M: BackingStore> {
    registers: Registers,
    isa: Isa,
    syscall_behavior: SystemCallBehavior,
    memory: M,
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
    pub fn new(isa: Isa, syscall_behavior: SystemCallBehavior, entry: u32, memory: M) -> Self {
        let registers = Registers {
            pc: entry.into(),
            xregs: [0.into(); 31],
            fregs: [0.; 32],
            csr: ControlStatusRegisters::new(&crate::csr::CsrInitContext {  }),
        };

        State {
            registers,
            isa,
            syscall_behavior,
            memory,
            instruction_counter: 0,
        }
    }

    pub fn pc(&self) -> Addr {
        self.registers.pc
    }

    pub fn prepare_exception_flags(&self) {
        let fflags = self.registers.csr.fcsr.get_fflags();
        let fflags: ::softfloat_wrapper::ExceptionFlags = fflags.into();

        fflags.set();
    }

    pub fn store_exception_flags(&mut self) {
        let mut fflags = ::softfloat_wrapper::ExceptionFlags::default();
        fflags.get();

        let fflags = ExceptionFlags::from(fflags);

        self.registers.csr.fcsr.set_fflags(fflags);
    }
    
    pub fn get_rounding_mode(&mut self, rm: u8) -> ::softfloat_wrapper::RoundingMode {
        // Dynamic rounding mode
        let rm = if rm == 0b111 {
            self.registers.csr.fcsr.frm_read() as u8
        } else {
            rm
        };

        use ::softfloat_wrapper::RoundingMode as R;
        match rm {
            0b000 => R::TiesToEven,
            0b001 => R::TowardZero,
            0b010 => R::TowardNegative,
            0b011 => R::TowardPositive,
            0b100 => R::TiesToAway,

            // @TODO: properly handle error
            _ => {
                eprintln!("rm = {rm:03b}");
                unimplemented!()
            },
        }
    }

    pub fn execute(mut self) -> Self {
        self.execute_mut();
        self
    }

    pub fn instruction_fetch(&mut self) -> Word {
        Word::from(self.memory.get(self.pc()))
    }

    pub fn instruction_decode(&mut self, imemory: Word) -> rvhwfuzzer_encoding::Instruction {
        let mut imemory_bytes = &imemory.to_le_bytes()[..];
        let Some(instruction) = rvhwfuzzer_encoding::Instruction::decode(&mut imemory_bytes).unwrap() else {
            panic!(
                "unknown instruction 0x{:08x} at pc=0x{:08x}",
                imemory.as_addr(),
                self.pc()
            );
        };
        instruction
    }

    pub fn instruction_execute(&mut self, instruction: rvhwfuzzer_encoding::Instruction) {
        use rvhwfuzzer_encoding::Instruction::*;

        let mut next_pc = self.pc().offset(4);

        eprintln!("[PC={:08X}]: {}", self.pc(), &instruction);

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
                next_pc = rs1.offset(args.imm());
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

                let mem = self.memory.get_byte(src);
                let mem = Word::sign_extend_u8(mem);

                self.registers.set(rd, mem);
            }
            Lh(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.memory.get_halfword(src);
                let mem = Word::sign_extend_u16(mem);

                self.registers.set(rd, mem);
            }
            Lw(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.memory.get(src);
                let mem = Word::from(mem);

                self.registers.set(rd, mem);
            }
            Lbu(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.memory.get_byte(src);
                let mem = Word::from(mem);

                self.registers.set(rd, mem);
            }
            Lhu(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs).as_addr();
                let src = rs.offset(args.imm());

                let mem = self.memory.get_halfword(src);
                let mem = Word::from(mem);

                self.registers.set(rd, mem);
            }
            Sb(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1).as_addr();
                let rs2 = self.registers.get(rs2).to_le_bytes()[0];
                let dest = rs1.offset(args.imm());

                self.memory.set_byte(dest, rs2);
            }
            Sh(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1).as_addr();
                let rs2 = self.registers.get(rs2).to_le_halfwords()[0];
                let dest = rs1.offset(args.imm());

                self.memory.set_halfword(dest, rs2);
            }
            Sw(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get(rs1).as_addr();
                let rs2 = self.registers.get(rs2).as_u32();
                let dest = rs1.offset(args.imm());

                self.memory.set(dest, rs2);
            }
            Addi(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_i32().wrapping_add(args.imm()));
            }
            Slti(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, u32::from(rs.as_i32() < args.imm()));
            }
            Sltiu(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, u32::from(rs.as_u32() < args.imm()));
            }
            Xori(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_i32() ^ args.imm());
            }
            Andi(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_i32() & args.imm());
            }
            Ori(args) => {
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers.get(rs);

                self.registers.set(rd, rs.as_i32() | args.imm());
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

                self.prepare_exception_flags();
                let result = rs1.fused_mul_add(rs2, rs3, rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            FnmsubS(args) => {
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

                self.prepare_exception_flags();
                let result = (rs1.neg()).fused_mul_add(rs2, rs3.neg(), rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            FmsubS(args) => {
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

                self.prepare_exception_flags();
                let result = rs1.fused_mul_add(rs2, rs3.neg(), rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            FnmaddS(args) => {
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

                self.prepare_exception_flags();
                let result = (rs1.neg()).fused_mul_add(rs2, rs3, rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            Fsw(args) => {
                let rs1 = args.rs1();
                let rs2 = args.rs2();
                let imm = args.imm();

                let rs1 = self.registers.get(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let dest = rs1.as_addr().offset(imm);

                self.memory.set(dest, rs2 as u32);
            }
            Flw(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let imm = args.imm();

                let rs1 = self.registers.get(rs1);

                let src = rs1.as_addr().offset(imm);
                let value = self.memory.get(src);

                self.registers.set_freg(rd, value as f32);
            }
            FmulS(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                self.prepare_exception_flags();
                let result = rs1.mul(rs2, rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            FdivS(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                self.prepare_exception_flags();
                let result = rs1.div(rs2, rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            FaddS(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                self.prepare_exception_flags();
                let result = rs1.add(rs2, rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            FsubS(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let rs1 = F32::from_f32(rs1);
                let rs2 = F32::from_f32(rs2);

                self.prepare_exception_flags();
                let result = rs1.sub(rs2, rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            FltS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set(rd, u32::from(rs1 < rs2));
            }
            FminS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set_freg(rd, f32::min(rs1, rs2));
            }
            FsgnjS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set_freg(rd, rs1.copysign(rs2));
            }
            FsgnjnS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set_freg(rd, rs1.copysign(-rs2));
            }
            FsgnjxS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                let xor = f32::from_bits(rs1.to_bits() ^ rs2.to_bits());

                self.registers.set_freg(rd, rs1.copysign(xor));
            }
            FleS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set(rd, u32::from(rs1 <= rs2));
            }
            FmaxS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set_freg(rd, f32::max(rs1, rs2));
            }
            FeqS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();
                let rs2 = args.rs2();

                let rs1 = self.registers.get_freg(rs1);
                let rs2 = self.registers.get_freg(rs2);

                self.registers.set(rd, u32::from(rs1 == rs2));
            }
            FcvtSW(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get(rs1);
                self.prepare_exception_flags();
                let result = F32::from_i32(rs1.as_i32(), rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));

            },
            FcvtWS(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs1 = F32::from_f32(rs1);
                self.prepare_exception_flags();
                let result = rs1.to_i32(rm, false);
                self.store_exception_flags();

                self.registers.set(rd, result);
            },
            FsqrtS(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs1 = F32::from_f32(rs1);

                self.prepare_exception_flags();
                let result = Float::sqrt(&rs1, rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            }
            FcvtSWu(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get(rs1);
                self.prepare_exception_flags();
                let result = F32::from_u32(rs1.as_u32(), rm);
                self.store_exception_flags();

                self.registers.set_freg(rd, f32::from_bits(result.to_bits()));
            },
            FcvtWuS(args) => {
                let rm = args.rm();
                let rd = args.rd();
                let rs1 = args.rs1();

                let rm = self.get_rounding_mode(rm as u8);

                let rs1 = self.registers.get_freg(rs1);
                let rs1 = F32::from_f32(rs1);
                self.prepare_exception_flags();
                let result = rs1.to_u32(rm, false);
                self.store_exception_flags();

                self.registers.set(rd, result);
            },
            FclassS(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();

                let rs1 = self.registers.get_freg(rs1);

                let class = match rs1 {
                    f if f == f32::NEG_INFINITY => 0,
                    f if f.is_normal() && f.is_sign_negative() => 1,
                    f if f.is_subnormal() && f.is_sign_negative() => 2,
                    f if f == 0. && f.is_sign_negative() => 3,
                    f if f == 0. && f.is_sign_positive() => 4,
                    f if f.is_subnormal() && f.is_sign_positive() => 5,
                    f if f.is_normal() && f.is_sign_positive() => 6,
                    f if f == f32::INFINITY => 7,
                    // @Hack. There is also a quiet NaN
                    f if f.is_nan() => 8,
                    // f if f.is_nan() && f.is_quiet() => 9,
                    _ => unreachable!(),
                };

                self.registers.set(rd, class);
            }
            FmvWX(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();

                let rs1 = self.registers.get(rs1);

                self.registers.set_freg(rd, rs1.as_u32() as f32);
            }
            FmvXW(args) => {
                let rd = args.rd();
                let rs1 = args.rs1();

                let rs1 = self.registers.get_freg(rs1);

                self.registers.set(rd, rs1 as u32);
            }
            Csrrw(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers().get(rs);

                self.registers.csr.write(csr, rs.as_u32());

                let value = self.registers.csr.read(csr);
                
                self.registers.set(rd, value);
            }
            Csrrs(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers().get(rs);

                let value = self.registers.csr.read(csr);
                self.registers.csr.write(csr, value | rs.as_u32());

                let value = self.registers.csr.read(csr);
                
                self.registers.set(rd, value);
            }
            Csrrc(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let rs = args.rs1();

                let rs = self.registers().get(rs);

                let value = self.registers.csr.read(csr);
                self.registers.csr.write(csr, value & !rs.as_u32());

                let value = self.registers.csr.read(csr);
                
                self.registers.set(rd, value);
            }
            Csrrwi(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let uimm = args.uimm();

                self.registers.csr.write(csr, uimm.into());

                let value = self.registers.csr.read(csr);
                
                self.registers.set(rd, value);
            }
            Csrrsi(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let uimm = args.uimm();

                let value = self.registers.csr.read(csr);
                self.registers.csr.write(csr, value & !u32::from(uimm));

                let value = self.registers.csr.read(csr);
                
                self.registers.set(rd, value);
            }
            Csrrci(args) => {
                let csr = args.csr();
                let rd = args.rd();
                let uimm = args.uimm();

                let value = self.registers.csr.read(csr);
                self.registers.csr.write(csr, value | u32::from(uimm));

                let value = self.registers.csr.read(csr);
                
                self.registers.set(rd, value);
            }
            Ecall(_args) => {
                match self
                    .syscall_behavior
                    .handle(&mut self.registers, &mut self.memory)
                {
                    SystemCallResult::Return(_) => {}
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
                eprintln!("Environment break called!");
                std::process::exit(0);
            },
            Fence(_) => {},
            FenceI(_) => {},
            // Environment(variant) => {
            //     self.instruction_counter += 1;
            //     match variant {
            //         EnvironmentVariant::Call => {
            //             match self
            //                 .syscall_behavior
            //                 .handle(&mut self.registers, &mut self.memory)
            //             {
            //                 SystemCallResult::Return(_) => {}
            //                 SystemCallResult::Exit(error_code) => {
            //                     std::process::exit(error_code);
            //                 }
            //                 SystemCallResult::Abort => {
            //                     println!("Process Aborted through System Call.");
            //                     std::process::exit(1);
            //                 }
            //                 SystemCallResult::InvalidSystemCallNr => {
            //                     println!("Invalid System Call was called.");
            //                     std::process::exit(1);
            //                 }
            //             }
            //         }
            //         EnvironmentVariant::Break => self.environment_break(),
            //     }
            // }
            // CsrRegister(args) => {
            //     self.instruction_counter += 1;
            //
            //     match args.csr {
            //         // RDCYCLE
            //         3072 => {
            //             self.registers.set(args.rd, self.instruction_counter as u32);
            //         }
            //         _ => unimplemented!(),
            //     }
            // }
            // CsrImmediate(_args) => {
            //     self.instruction_counter += 1;
            //
            //     todo!()
            // }
        };

        self.registers.pc = next_pc;
    }

    pub fn memory(&mut self) -> &M {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut M {
        &mut self.memory
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
        // if Into::<Word>::into(self.pc()).as_u32() >= 0x8000_016C {
        //     self.environment_break();
        // }

        let imemory = self.instruction_fetch();
        let instruction = self.instruction_decode(imemory);
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
