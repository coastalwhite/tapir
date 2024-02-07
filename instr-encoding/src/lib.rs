pub mod asm {
	#[derive(Debug)]
	pub struct AsmDisplay<'a, T> {
		pub ctx: &'a AsmDisplayContext,
		pub instr: &'a T,
	}
	pub trait AsmField {
		fn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: &AsmDisplayContext) -> ::std::fmt::Result;
	}
	#[derive(Debug, Default)]
	pub struct AsmDisplayContext {
		xreg: 	XregDisplayVariant,
		freg: 	FregDisplayVariant,
	}
	#[derive(Debug, Default)]
	pub struct AsmSignedInt(u64);
	impl AsmField for AsmSignedInt {
		fn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, _: &AsmDisplayContext) -> ::std::fmt::Result {
			write!(f, "{}", self.0 as i64)?;
			Ok(())
		}
	}
	#[derive(Debug, Default)]
	pub struct AsmUnsignedInt(u64);
	impl AsmField for AsmUnsignedInt {
		fn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, _: &AsmDisplayContext) -> ::std::fmt::Result {
			write!(f, "{}", self.0 as u64)?;
			Ok(())
		}
	}
	#[derive(Debug, Default)]
	pub struct AsmSignedHex(u64);
	impl AsmField for AsmSignedHex {
		fn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, _: &AsmDisplayContext) -> ::std::fmt::Result {
			write!(f, "{:x}", self.0 as i64)?;
			Ok(())
		}
	}
	#[derive(Debug, Default)]
	pub struct AsmUnsignedHex(u64);
	impl AsmField for AsmUnsignedHex {
		fn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, _: &AsmDisplayContext) -> ::std::fmt::Result {
			write!(f, "{:x}", self.0 as u64)?;
			Ok(())
		}
	}
	#[derive(Debug, Default)]
	pub struct AsmRelLabel(u64);
	impl AsmField for AsmRelLabel {
		fn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, _: &AsmDisplayContext) -> ::std::fmt::Result {
			write!(f, "{}", self.0 as i64)?;
			Ok(())
		}
	}
	#[derive(Debug, Default)]
	pub enum XregDisplayVariant {
		#[default]
		Labeled,
		Numbered,
	}
	#[derive(Debug, Default)]
	pub struct AsmXreg(u8, );
	impl AsmField for AsmXreg {
		fn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: &AsmDisplayContext) -> ::std::fmt::Result {
			match ctx.xreg {
				XregDisplayVariant::Labeled => {
					static LUT: [&str; 32] = ["zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "fp", "s1", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6", ];
					f.write_str(LUT[(self.0 & 0b11111) as usize])?;
				},
				XregDisplayVariant::Numbered => {
					todo!();
				},
			}
			Ok(())
		}
	}
	#[derive(Debug, Default)]
	pub enum FregDisplayVariant {
		#[default]
		Labeled,
		Numbered,
	}
	#[derive(Debug, Default)]
	pub struct AsmFreg(u8, );
	impl AsmField for AsmFreg {
		fn fmt_field(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: &AsmDisplayContext) -> ::std::fmt::Result {
			match ctx.freg {
				FregDisplayVariant::Labeled => {
					static LUT: [&str; 32] = ["ft0", "ft1", "ft2", "ft3", "ft4", "ft5", "ft6", "ft7", "fs0", "fs1", "fa0", "fa1", "fa2", "fa3", "fa4", "fa5", "fa6", "fa7", "fs2", "fs3", "fs4", "fs5", "fs6", "fs7", "fs8", "fs9", "fs10", "fs11", "ft8", "ft9", "ft10", "ft11", ];
					f.write_str(LUT[(self.0 & 0b11111) as usize])?;
				},
				FregDisplayVariant::Numbered => {
					todo!();
				},
			}
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::AuipcArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("auipc      ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedHex(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::JalArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("jal        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmRelLabel(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::LuiArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("lui        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedHex(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FmsubSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fmsub.s    ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs3.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FnmsubSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fnmsub.s   ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs3.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FnmaddSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fnmadd.s   ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs3.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FmaddSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fmadd.s    ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs3.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SbArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("sb         ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::LwArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("lw         ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::BgeArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("bge        ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmRelLabel(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::BneArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("bne        ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmRelLabel(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::LbuArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("lbu        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::BltuArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("bltu       ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmRelLabel(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::CsrrwArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("csrrw      ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.csr.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::XoriArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("xori       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::AndiArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("andi       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SwArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("sw         ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::CsrrsArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("csrrs      ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.csr.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::ShArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("sh         ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SltiuArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("sltiu      ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::LbArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("lb         ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::LhArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("lh         ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::CsrrwiArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("csrrwi     ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.uimm.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.csr.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SltiArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("slti       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::CsrrcArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("csrrc      ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.csr.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::LhuArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("lhu        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FswArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fsw        ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::AddiArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("addi       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::BltArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("blt        ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmRelLabel(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::BgeuArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("bgeu       ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmRelLabel(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::CsrrsiArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("csrrsi     ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.uimm.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.csr.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::CsrrciArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("csrrci     ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.uimm.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.csr.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::JalrArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("jalr       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmRelLabel(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FlwArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("flw        ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::BeqArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("beq        ")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmRelLabel(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::OriArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("ori        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmSignedInt(self.instr.imm().into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FdivSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fdiv.s     ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FsubSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fsub.s     ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FaddSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fadd.s     ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FmulSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fmul.s     ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SraiArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("srai       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.shamt.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FeqSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("feq.s      ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::AddArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("add        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SlliArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("slli       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.shamt.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SltuArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("sltu       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FsgnjxSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fsgnjx.s   ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FltSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("flt.s      ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FsgnjSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fsgnj.s    ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FmaxSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fmax.s     ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FsgnjnSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fsgnjn.s   ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FminSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fmin.s     ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::XorArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("xor        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SrliArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("srli       ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmUnsignedInt(self.instr.shamt.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SltArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("slt        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SubArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("sub        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SllArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("sll        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SrlArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("srl        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::SraArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("sra        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FleSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fle.s      ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs2.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::AndArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("and        ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::OrArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("or         ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FcvtWuSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fcvt.wu.s  ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FcvtSWArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fcvt.s.w   ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FcvtSWuArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fcvt.s.wu  ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FsqrtSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fsqrt.s    ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FcvtWSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fcvt.w.s   ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FmvXWArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fmv.x.w    ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FmvWXArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fmv.w.x    ")?;
			AsmFreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmXreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::FclassSArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("fclass.s   ")?;
			AsmXreg(self.instr.rd.into(), ).fmt_field(f, self.ctx)?;
			f.write_str(",")?;
			AsmFreg(self.instr.rs1.into(), ).fmt_field(f, self.ctx)?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::EcallArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("ecall      ")?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::EbreakArgs> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			f.write_str("ebreak     ")?;
			Ok(())
		}
	}
	impl<'a> ::std::fmt::Display for AsmDisplay<'a, super::Instruction> {
		fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
			use super::Instruction as I;
			match self.instr {
				I::Auipc(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Jal(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Lui(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FmsubS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FnmsubS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FnmaddS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FmaddS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Sb(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Lw(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Bge(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Bne(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Lbu(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Bltu(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Csrrw(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Xori(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Andi(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Sw(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Csrrs(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Sh(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Sltiu(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Lb(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Lh(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Csrrwi(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Slti(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Csrrc(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Lhu(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Fsw(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Addi(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Blt(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Bgeu(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Csrrsi(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Csrrci(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Jalr(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Flw(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Beq(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Ori(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FdivS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FsubS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FaddS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FmulS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Srai(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FeqS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Add(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Slli(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Sltu(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FsgnjxS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FltS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FsgnjS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FmaxS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FsgnjnS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FminS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Xor(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Srli(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Slt(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Sub(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Sll(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Srl(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Sra(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FleS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::And(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Or(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FcvtWuS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FcvtSW(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FcvtSWu(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FsqrtS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FcvtWS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FmvXW(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FmvWX(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::FclassS(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Ecall(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
				I::Ebreak(ref args) => AsmDisplay { ctx: self.ctx, instr: args }.fmt(f),
			}
		}
	}
}
#[derive(Debug)]
pub struct AuipcArgs {
	pub rd: u8,
	pub imm31_12: u32,
}
impl AuipcArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm31_12: ((encoded >> 12) & 0b11111111111111111111) as u32,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000000010111;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm31_12 as u32) << 12;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm31_12 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm31_12 as u32) << 12) | 0
	}
}
#[derive(Debug)]
pub struct JalArgs {
	pub imm20: u8,
	pub imm10_1: u16,
	pub imm11: u8,
	pub rd: u8,
	pub imm19_12: u8,
}
impl JalArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			imm20: ((encoded >> 31) & 0b1) as u8,
			imm10_1: ((encoded >> 21) & 0b1111111111) as u16,
			imm11: ((encoded >> 20) & 0b1) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm19_12: ((encoded >> 12) & 0b11111111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001101111;
		encoded |= (self.imm20 as u32) << 31;
		encoded |= (self.imm10_1 as u32) << 21;
		encoded |= (self.imm11 as u32) << 20;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm19_12 as u32) << 12;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm20 @ imm19_12 @ imm11 @ imm10_1 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm10_1 as u32) << 1) | ((self.imm11 as u32) << 11) | ((self.imm19_12 as u32) << 12) | ((self.imm20 as u32) << 20) | 0
	}
}
#[derive(Debug)]
pub struct LuiArgs {
	pub rd: u8,
	pub imm31_12: u32,
}
impl LuiArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm31_12: ((encoded >> 12) & 0b11111111111111111111) as u32,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000000110111;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm31_12 as u32) << 12;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm31_12 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm31_12 as u32) << 12) | 0
	}
}
#[derive(Debug)]
pub struct FmsubSArgs {
	pub rm: u8,
	pub rs3: u8,
	pub rs2: u8,
	pub rd: u8,
	pub rs1: u8,
}
impl FmsubSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rm: ((encoded >> 12) & 0b111) as u8,
			rs3: ((encoded >> 27) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001000111;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rs3 as u32) << 27;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs1 as u32) << 15;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FnmsubSArgs {
	pub rm: u8,
	pub rs3: u8,
	pub rs2: u8,
	pub rd: u8,
	pub rs1: u8,
}
impl FnmsubSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rm: ((encoded >> 12) & 0b111) as u8,
			rs3: ((encoded >> 27) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001001011;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rs3 as u32) << 27;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs1 as u32) << 15;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FnmaddSArgs {
	pub rm: u8,
	pub rs3: u8,
	pub rs2: u8,
	pub rd: u8,
	pub rs1: u8,
}
impl FnmaddSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rm: ((encoded >> 12) & 0b111) as u8,
			rs3: ((encoded >> 27) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001001111;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rs3 as u32) << 27;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs1 as u32) << 15;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FmaddSArgs {
	pub rm: u8,
	pub rs3: u8,
	pub rs2: u8,
	pub rd: u8,
	pub rs1: u8,
}
impl FmaddSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rm: ((encoded >> 12) & 0b111) as u8,
			rs3: ((encoded >> 27) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001000011;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rs3 as u32) << 27;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs1 as u32) << 15;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SbArgs {
	pub rs1: u8,
	pub imm11_5: u8,
	pub imm4_0: u8,
	pub rs2: u8,
}
impl SbArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm11_5: ((encoded >> 25) & 0b1111111) as u8,
			imm4_0: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000000100011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm11_5 as u32) << 25;
		encoded |= (self.imm4_0 as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_5 @ imm4_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm4_0 as u32) << 0) | ((self.imm11_5 as u32) << 5) | 0
	}
}
#[derive(Debug)]
pub struct LwArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl LwArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000010000000000011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct BgeArgs {
	pub imm10_5: u8,
	pub rs1: u8,
	pub imm4_1: u8,
	pub imm12: u8,
	pub rs2: u8,
	pub imm11: u8,
}
impl BgeArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			imm10_5: ((encoded >> 25) & 0b111111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm4_1: ((encoded >> 8) & 0b1111) as u8,
			imm12: ((encoded >> 31) & 0b1) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			imm11: ((encoded >> 7) & 0b1) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000101000001100011;
		encoded |= (self.imm10_5 as u32) << 25;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm4_1 as u32) << 8;
		encoded |= (self.imm12 as u32) << 31;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.imm11 as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm12 @ imm11 @ imm10_5 @ imm4_1 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm4_1 as u32) << 1) | ((self.imm10_5 as u32) << 5) | ((self.imm11 as u32) << 11) | ((self.imm12 as u32) << 12) | 0
	}
}
#[derive(Debug)]
pub struct BneArgs {
	pub imm10_5: u8,
	pub rs1: u8,
	pub imm4_1: u8,
	pub imm12: u8,
	pub rs2: u8,
	pub imm11: u8,
}
impl BneArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			imm10_5: ((encoded >> 25) & 0b111111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm4_1: ((encoded >> 8) & 0b1111) as u8,
			imm12: ((encoded >> 31) & 0b1) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			imm11: ((encoded >> 7) & 0b1) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000001000001100011;
		encoded |= (self.imm10_5 as u32) << 25;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm4_1 as u32) << 8;
		encoded |= (self.imm12 as u32) << 31;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.imm11 as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm12 @ imm11 @ imm10_5 @ imm4_1 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm4_1 as u32) << 1) | ((self.imm10_5 as u32) << 5) | ((self.imm11 as u32) << 11) | ((self.imm12 as u32) << 12) | 0
	}
}
#[derive(Debug)]
pub struct LbuArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl LbuArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000100000000000011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct BltuArgs {
	pub imm10_5: u8,
	pub rs1: u8,
	pub imm4_1: u8,
	pub imm12: u8,
	pub rs2: u8,
	pub imm11: u8,
}
impl BltuArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			imm10_5: ((encoded >> 25) & 0b111111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm4_1: ((encoded >> 8) & 0b1111) as u8,
			imm12: ((encoded >> 31) & 0b1) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			imm11: ((encoded >> 7) & 0b1) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000110000001100011;
		encoded |= (self.imm10_5 as u32) << 25;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm4_1 as u32) << 8;
		encoded |= (self.imm12 as u32) << 31;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.imm11 as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm12 @ imm11 @ imm10_5 @ imm4_1 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm4_1 as u32) << 1) | ((self.imm10_5 as u32) << 5) | ((self.imm11 as u32) << 11) | ((self.imm12 as u32) << 12) | 0
	}
}
#[derive(Debug)]
pub struct CsrrwArgs {
	pub rs1: u8,
	pub rd: u8,
	pub csr: u16,
}
impl CsrrwArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			csr: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000001000001110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.csr as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct XoriArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl XoriArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000100000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct AndiArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl AndiArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000111000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct SwArgs {
	pub rs1: u8,
	pub imm11_5: u8,
	pub imm4_0: u8,
	pub rs2: u8,
}
impl SwArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm11_5: ((encoded >> 25) & 0b1111111) as u8,
			imm4_0: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000010000000100011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm11_5 as u32) << 25;
		encoded |= (self.imm4_0 as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_5 @ imm4_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm4_0 as u32) << 0) | ((self.imm11_5 as u32) << 5) | 0
	}
}
#[derive(Debug)]
pub struct CsrrsArgs {
	pub rs1: u8,
	pub rd: u8,
	pub csr: u16,
}
impl CsrrsArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			csr: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000010000001110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.csr as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct ShArgs {
	pub rs1: u8,
	pub imm11_5: u8,
	pub imm4_0: u8,
	pub rs2: u8,
}
impl ShArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm11_5: ((encoded >> 25) & 0b1111111) as u8,
			imm4_0: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000001000000100011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm11_5 as u32) << 25;
		encoded |= (self.imm4_0 as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_5 @ imm4_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm4_0 as u32) << 0) | ((self.imm11_5 as u32) << 5) | 0
	}
}
#[derive(Debug)]
pub struct SltiuArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl SltiuArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000011000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct LbArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl LbArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000000000011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct LhArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl LhArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000001000000000011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct CsrrwiArgs {
	pub uimm: u8,
	pub rd: u8,
	pub csr: u16,
}
impl CsrrwiArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			uimm: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			csr: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000101000001110011;
		encoded |= (self.uimm as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.csr as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SltiArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl SltiArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000010000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct CsrrcArgs {
	pub rs1: u8,
	pub rd: u8,
	pub csr: u16,
}
impl CsrrcArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			csr: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000011000001110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.csr as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct LhuArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl LhuArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000101000000000011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct FswArgs {
	pub rs1: u8,
	pub imm11_5: u8,
	pub imm4_0: u8,
	pub rs2: u8,
}
impl FswArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm11_5: ((encoded >> 25) & 0b1111111) as u8,
			imm4_0: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000010000000100111;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm11_5 as u32) << 25;
		encoded |= (self.imm4_0 as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_5 @ imm4_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm4_0 as u32) << 0) | ((self.imm11_5 as u32) << 5) | 0
	}
}
#[derive(Debug)]
pub struct AddiArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl AddiArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct BltArgs {
	pub imm10_5: u8,
	pub rs1: u8,
	pub imm4_1: u8,
	pub imm12: u8,
	pub rs2: u8,
	pub imm11: u8,
}
impl BltArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			imm10_5: ((encoded >> 25) & 0b111111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm4_1: ((encoded >> 8) & 0b1111) as u8,
			imm12: ((encoded >> 31) & 0b1) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			imm11: ((encoded >> 7) & 0b1) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000100000001100011;
		encoded |= (self.imm10_5 as u32) << 25;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm4_1 as u32) << 8;
		encoded |= (self.imm12 as u32) << 31;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.imm11 as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm12 @ imm11 @ imm10_5 @ imm4_1 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm4_1 as u32) << 1) | ((self.imm10_5 as u32) << 5) | ((self.imm11 as u32) << 11) | ((self.imm12 as u32) << 12) | 0
	}
}
#[derive(Debug)]
pub struct BgeuArgs {
	pub imm10_5: u8,
	pub rs1: u8,
	pub imm4_1: u8,
	pub imm12: u8,
	pub rs2: u8,
	pub imm11: u8,
}
impl BgeuArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			imm10_5: ((encoded >> 25) & 0b111111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm4_1: ((encoded >> 8) & 0b1111) as u8,
			imm12: ((encoded >> 31) & 0b1) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			imm11: ((encoded >> 7) & 0b1) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000111000001100011;
		encoded |= (self.imm10_5 as u32) << 25;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm4_1 as u32) << 8;
		encoded |= (self.imm12 as u32) << 31;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.imm11 as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm12 @ imm11 @ imm10_5 @ imm4_1 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm4_1 as u32) << 1) | ((self.imm10_5 as u32) << 5) | ((self.imm11 as u32) << 11) | ((self.imm12 as u32) << 12) | 0
	}
}
#[derive(Debug)]
pub struct CsrrsiArgs {
	pub uimm: u8,
	pub rd: u8,
	pub csr: u16,
}
impl CsrrsiArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			uimm: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			csr: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000110000001110011;
		encoded |= (self.uimm as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.csr as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct CsrrciArgs {
	pub uimm: u8,
	pub rd: u8,
	pub csr: u16,
}
impl CsrrciArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			uimm: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			csr: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000111000001110011;
		encoded |= (self.uimm as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.csr as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct JalrArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl JalrArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001100111;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct FlwArgs {
	pub rs1: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl FlwArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000010000000000111;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct BeqArgs {
	pub imm10_5: u8,
	pub rs1: u8,
	pub imm4_1: u8,
	pub imm12: u8,
	pub rs2: u8,
	pub imm11: u8,
}
impl BeqArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			imm10_5: ((encoded >> 25) & 0b111111) as u8,
			rs1: ((encoded >> 15) & 0b11111) as u8,
			imm4_1: ((encoded >> 8) & 0b1111) as u8,
			imm12: ((encoded >> 31) & 0b1) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
			imm11: ((encoded >> 7) & 0b1) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001100011;
		encoded |= (self.imm10_5 as u32) << 25;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.imm4_1 as u32) << 8;
		encoded |= (self.imm12 as u32) << 31;
		encoded |= (self.rs2 as u32) << 20;
		encoded |= (self.imm11 as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm12 @ imm11 @ imm10_5 @ imm4_1 @ 0 @ 
	pub fn imm(&self) -> u32 {
		((0b0 as u32) << 0) | ((self.imm4_1 as u32) << 1) | ((self.imm10_5 as u32) << 5) | ((self.imm11 as u32) << 11) | ((self.imm12 as u32) << 12) | 0
	}
}
#[derive(Debug)]
pub struct OriArgs {
	pub rs: u8,
	pub rd: u8,
	pub imm11_0: u16,
}
impl OriArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			imm11_0: ((encoded >> 20) & 0b111111111111) as u16,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000110000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.imm11_0 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
	/// imm11_0 @ 
	pub fn imm(&self) -> u32 {
		((self.imm11_0 as u32) << 0) | 0
	}
}
#[derive(Debug)]
pub struct FdivSArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FdivSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00011000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FsubSArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FsubSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00001000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FaddSArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FaddSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FmulSArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FmulSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00010000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SraiArgs {
	pub rs: u8,
	pub rd: u8,
	pub shamt: u8,
}
impl SraiArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			shamt: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b01000000000000000101000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.shamt as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FeqSArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FeqSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b10100000000000000010000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct AddArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl AddArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SlliArgs {
	pub rs: u8,
	pub rd: u8,
	pub shamt: u8,
}
impl SlliArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			shamt: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000001000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.shamt as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SltuArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl SltuArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000011000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FsgnjxSArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FsgnjxSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00100000000000000010000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FltSArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FltSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b10100000000000000001000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FsgnjSArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FsgnjSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00100000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FmaxSArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FmaxSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00101000000000000001000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FsgnjnSArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FsgnjnSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00100000000000000001000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FminSArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FminSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00101000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct XorArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl XorArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000100000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SrliArgs {
	pub rs: u8,
	pub rd: u8,
	pub shamt: u8,
}
impl SrliArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			shamt: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000101000000010011;
		encoded |= (self.rs as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.shamt as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SltArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl SltArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000010000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SubArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl SubArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b01000000000000000000000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SllArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl SllArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000001000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SrlArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl SrlArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000101000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct SraArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl SraArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b01000000000000000101000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FleSArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl FleSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b10100000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct AndArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl AndArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000111000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct OrArgs {
	pub rs1: u8,
	pub rd: u8,
	pub rs2: u8,
}
impl OrArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
			rs2: ((encoded >> 20) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000110000000110011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;
		encoded |= (self.rs2 as u32) << 20;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FcvtWuSArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
}
impl FcvtWuSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b11000000000100000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FcvtSWArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
}
impl FcvtSWArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b11010000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FcvtSWuArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
}
impl FcvtSWuArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b11010000000100000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FsqrtSArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
}
impl FsqrtSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b01011000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FcvtWSArgs {
	pub rs1: u8,
	pub rm: u8,
	pub rd: u8,
}
impl FcvtWSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rm: ((encoded >> 12) & 0b111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b11000000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rm as u32) << 12;
		encoded |= (self.rd as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FmvXWArgs {
	pub rs1: u8,
	pub rd: u8,
}
impl FmvXWArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b11100000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FmvWXArgs {
	pub rs1: u8,
	pub rd: u8,
}
impl FmvWXArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b11110000000000000000000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct FclassSArgs {
	pub rs1: u8,
	pub rd: u8,
}
impl FclassSArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
			rs1: ((encoded >> 15) & 0b11111) as u8,
			rd: ((encoded >> 7) & 0b11111) as u8,
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b11100000000000000001000001010011;
		encoded |= (self.rs1 as u32) << 15;
		encoded |= (self.rd as u32) << 7;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct EcallArgs {
}
impl EcallArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000000000000000001110011;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub struct EbreakArgs {
}
impl EbreakArgs {
	pub fn take_args(encoded: u32) -> Self {
		Self {
		}
	}
	pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
		let mut encoded: u32 = 0b00000000000100000000000001110011;

		writer.write_all(&encoded.to_le_bytes())
	}
}
#[derive(Debug)]
pub enum Instruction {
	Auipc(AuipcArgs),
	Jal(JalArgs),
	Lui(LuiArgs),
	FmsubS(FmsubSArgs),
	FnmsubS(FnmsubSArgs),
	FnmaddS(FnmaddSArgs),
	FmaddS(FmaddSArgs),
	Sb(SbArgs),
	Lw(LwArgs),
	Bge(BgeArgs),
	Bne(BneArgs),
	Lbu(LbuArgs),
	Bltu(BltuArgs),
	Csrrw(CsrrwArgs),
	Xori(XoriArgs),
	Andi(AndiArgs),
	Sw(SwArgs),
	Csrrs(CsrrsArgs),
	Sh(ShArgs),
	Sltiu(SltiuArgs),
	Lb(LbArgs),
	Lh(LhArgs),
	Csrrwi(CsrrwiArgs),
	Slti(SltiArgs),
	Csrrc(CsrrcArgs),
	Lhu(LhuArgs),
	Fsw(FswArgs),
	Addi(AddiArgs),
	Blt(BltArgs),
	Bgeu(BgeuArgs),
	Csrrsi(CsrrsiArgs),
	Csrrci(CsrrciArgs),
	Jalr(JalrArgs),
	Flw(FlwArgs),
	Beq(BeqArgs),
	Ori(OriArgs),
	FdivS(FdivSArgs),
	FsubS(FsubSArgs),
	FaddS(FaddSArgs),
	FmulS(FmulSArgs),
	Srai(SraiArgs),
	FeqS(FeqSArgs),
	Add(AddArgs),
	Slli(SlliArgs),
	Sltu(SltuArgs),
	FsgnjxS(FsgnjxSArgs),
	FltS(FltSArgs),
	FsgnjS(FsgnjSArgs),
	FmaxS(FmaxSArgs),
	FsgnjnS(FsgnjnSArgs),
	FminS(FminSArgs),
	Xor(XorArgs),
	Srli(SrliArgs),
	Slt(SltArgs),
	Sub(SubArgs),
	Sll(SllArgs),
	Srl(SrlArgs),
	Sra(SraArgs),
	FleS(FleSArgs),
	And(AndArgs),
	Or(OrArgs),
	FcvtWuS(FcvtWuSArgs),
	FcvtSW(FcvtSWArgs),
	FcvtSWu(FcvtSWuArgs),
	FsqrtS(FsqrtSArgs),
	FcvtWS(FcvtWSArgs),
	FmvXW(FmvXWArgs),
	FmvWX(FmvWXArgs),
	FclassS(FclassSArgs),
	Ecall(EcallArgs),
	Ebreak(EbreakArgs),
}
impl Instruction {
	pub fn decode(encoded: &[u8]) -> Option<Instruction> {
		let bits: u32 = u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3], ]);
		if bits & 0b00000000000000000000000001111111 == 0b00000000000000000000000000010111 {
			return Some(Self::Auipc(AuipcArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000000000001111111 == 0b00000000000000000000000001101111 {
			return Some(Self::Jal(JalArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000000000001111111 == 0b00000000000000000000000000110111 {
			return Some(Self::Lui(LuiArgs::take_args(bits)));
		}
		if bits & 0b00000110000000000000000001111111 == 0b00000000000000000000000001000111 {
			return Some(Self::FmsubS(FmsubSArgs::take_args(bits)));
		}
		if bits & 0b00000110000000000000000001111111 == 0b00000000000000000000000001001011 {
			return Some(Self::FnmsubS(FnmsubSArgs::take_args(bits)));
		}
		if bits & 0b00000110000000000000000001111111 == 0b00000000000000000000000001001111 {
			return Some(Self::FnmaddS(FnmaddSArgs::take_args(bits)));
		}
		if bits & 0b00000110000000000000000001111111 == 0b00000000000000000000000001000011 {
			return Some(Self::FmaddS(FmaddSArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000000000000100011 {
			return Some(Self::Sb(SbArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000010000000000011 {
			return Some(Self::Lw(LwArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000101000001100011 {
			return Some(Self::Bge(BgeArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000001000001100011 {
			return Some(Self::Bne(BneArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000100000000000011 {
			return Some(Self::Lbu(LbuArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000110000001100011 {
			return Some(Self::Bltu(BltuArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000001000001110011 {
			return Some(Self::Csrrw(CsrrwArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000100000000010011 {
			return Some(Self::Xori(XoriArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000111000000010011 {
			return Some(Self::Andi(AndiArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000010000000100011 {
			return Some(Self::Sw(SwArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000010000001110011 {
			return Some(Self::Csrrs(CsrrsArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000001000000100011 {
			return Some(Self::Sh(ShArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000011000000010011 {
			return Some(Self::Sltiu(SltiuArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000000000000000011 {
			return Some(Self::Lb(LbArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000001000000000011 {
			return Some(Self::Lh(LhArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000101000001110011 {
			return Some(Self::Csrrwi(CsrrwiArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000010000000010011 {
			return Some(Self::Slti(SltiArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000011000001110011 {
			return Some(Self::Csrrc(CsrrcArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000101000000000011 {
			return Some(Self::Lhu(LhuArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000010000000100111 {
			return Some(Self::Fsw(FswArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000000000000010011 {
			return Some(Self::Addi(AddiArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000100000001100011 {
			return Some(Self::Blt(BltArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000111000001100011 {
			return Some(Self::Bgeu(BgeuArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000110000001110011 {
			return Some(Self::Csrrsi(CsrrsiArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000111000001110011 {
			return Some(Self::Csrrci(CsrrciArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000000000001100111 {
			return Some(Self::Jalr(JalrArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000010000000000111 {
			return Some(Self::Flw(FlwArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000000000001100011 {
			return Some(Self::Beq(BeqArgs::take_args(bits)));
		}
		if bits & 0b00000000000000000111000001111111 == 0b00000000000000000110000000010011 {
			return Some(Self::Ori(OriArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000000000001111111 == 0b00011000000000000000000001010011 {
			return Some(Self::FdivS(FdivSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000000000001111111 == 0b00001000000000000000000001010011 {
			return Some(Self::FsubS(FsubSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000000000001111111 == 0b00000000000000000000000001010011 {
			return Some(Self::FaddS(FaddSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000000000001111111 == 0b00010000000000000000000001010011 {
			return Some(Self::FmulS(FmulSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b01000000000000000101000000010011 {
			return Some(Self::Srai(SraiArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b10100000000000000010000001010011 {
			return Some(Self::FeqS(FeqSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000000000000110011 {
			return Some(Self::Add(AddArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000001000000010011 {
			return Some(Self::Slli(SlliArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000011000000110011 {
			return Some(Self::Sltu(SltuArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00100000000000000010000001010011 {
			return Some(Self::FsgnjxS(FsgnjxSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b10100000000000000001000001010011 {
			return Some(Self::FltS(FltSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00100000000000000000000001010011 {
			return Some(Self::FsgnjS(FsgnjSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00101000000000000001000001010011 {
			return Some(Self::FmaxS(FmaxSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00100000000000000001000001010011 {
			return Some(Self::FsgnjnS(FsgnjnSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00101000000000000000000001010011 {
			return Some(Self::FminS(FminSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000100000000110011 {
			return Some(Self::Xor(XorArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000101000000010011 {
			return Some(Self::Srli(SrliArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000010000000110011 {
			return Some(Self::Slt(SltArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b01000000000000000000000000110011 {
			return Some(Self::Sub(SubArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000001000000110011 {
			return Some(Self::Sll(SllArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000101000000110011 {
			return Some(Self::Srl(SrlArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b01000000000000000101000000110011 {
			return Some(Self::Sra(SraArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b10100000000000000000000001010011 {
			return Some(Self::FleS(FleSArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000111000000110011 {
			return Some(Self::And(AndArgs::take_args(bits)));
		}
		if bits & 0b11111110000000000111000001111111 == 0b00000000000000000110000000110011 {
			return Some(Self::Or(OrArgs::take_args(bits)));
		}
		if bits & 0b11111111111100000000000001111111 == 0b11000000000100000000000001010011 {
			return Some(Self::FcvtWuS(FcvtWuSArgs::take_args(bits)));
		}
		if bits & 0b11111111111100000000000001111111 == 0b11010000000000000000000001010011 {
			return Some(Self::FcvtSW(FcvtSWArgs::take_args(bits)));
		}
		if bits & 0b11111111111100000000000001111111 == 0b11010000000100000000000001010011 {
			return Some(Self::FcvtSWu(FcvtSWuArgs::take_args(bits)));
		}
		if bits & 0b11111111111100000000000001111111 == 0b01011000000000000000000001010011 {
			return Some(Self::FsqrtS(FsqrtSArgs::take_args(bits)));
		}
		if bits & 0b11111111111100000000000001111111 == 0b11000000000000000000000001010011 {
			return Some(Self::FcvtWS(FcvtWSArgs::take_args(bits)));
		}
		if bits & 0b11111111111100000111000001111111 == 0b11100000000000000000000001010011 {
			return Some(Self::FmvXW(FmvXWArgs::take_args(bits)));
		}
		if bits & 0b11111111111100000111000001111111 == 0b11110000000000000000000001010011 {
			return Some(Self::FmvWX(FmvWXArgs::take_args(bits)));
		}
		if bits & 0b11111111111100000111000001111111 == 0b11100000000000000001000001010011 {
			return Some(Self::FclassS(FclassSArgs::take_args(bits)));
		}
		if bits & 0b11111111111111111111111111111111 == 0b00000000000000000000000001110011 {
			return Some(Self::Ecall(EcallArgs::take_args(bits)));
		}
		if bits & 0b11111111111111111111111111111111 == 0b00000000000100000000000001110011 {
			return Some(Self::Ebreak(EbreakArgs::take_args(bits)));
		}
		None
	}
}
