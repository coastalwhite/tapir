use std::io;

use crate::RegisterId;

pub struct CsrRead {
    rd: RegisterId,
    id: u16,
}

pub struct CsrWrite {
    rs: RegisterId,
    id: u16,
}

pub struct CsrImmWrite {
    uimm: u8,
    id: u16,
}

macro_rules! csrs {
    (
    $($name:ident = $id:literal $(($write_ident:ident))?),+ $(,)?
    ) => {
        impl CsrRead {
            const NUM_CSRS: usize = 0 $(+ { stringify!($name); 1 })+;
        }

        impl CsrWrite {
            const NUM_CSRS: usize = 0 $($(+ { stringify!($write_ident); 1 })?)+;
        }

        impl CsrImmWrite {
            const NUM_CSRS: usize = 0 $($(+ { stringify!($write_ident); 1 })?)+;
        }

        impl $crate::arbitrary::ArbitraryInstruction for CsrRead {
            fn take<P: $crate::arbitrary::ArbitraryParameterProvider>(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext<P>) -> Self {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let rd = ctx.params_mut().take_register_dest();

                let mut i = 0;
                $(
                    if i == r {
                        return Self {
                            rd,
                            id: $id,
                        };
                    }
                    #[allow(unused_assignments)]
                    {
                        i += 1;
                    }
                )+

                unreachable!();
            }
        }

        impl $crate::arbitrary::ArbitraryInstruction for CsrWrite {
            fn take<P: $crate::arbitrary::ArbitraryParameterProvider>(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext<P>) -> Self {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let rs = ctx.params_mut().take_register_src();

                let mut i = 0;
                $($(
                    if i == r {
                        stringify!($write_ident);
                        return Self {
                            rs,
                            id: $id,
                        };
                    }
                    #[allow(unused_assignments)]
                    {
                        i += 1;
                    }
                )?)+

                unreachable!();
            }
        }

        impl $crate::arbitrary::ArbitraryInstruction for CsrImmWrite {
            fn take<P: $crate::arbitrary::ArbitraryParameterProvider>(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext<P>) -> Self {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let uimm = ctx.params_mut().take_u8(5);

                let mut i = 0;
                $($(
                    if i == r {
                        stringify!($write_ident);
                        return Self {
                            uimm,
                            id: $id,
                        };
                    }
                    #[allow(unused_assignments)]
                    {
                        i += 1;
                    }
                )?)+

                unreachable!();
            }
        }
    };
}

impl CsrRead {
    pub fn encode(&self, writer: &mut impl io::Write) -> io::Result<()> {
        ::rvhwfuzzer_encoding::CsrrcArgs { csr: self.id, rd: self.rd.0, rs1: 0 }.encode(writer)
    }
}

impl CsrWrite {
    pub fn encode(&self, writer: &mut impl io::Write) -> io::Result<()> {
        ::rvhwfuzzer_encoding::CsrrwArgs { csr: self.id, rd: 0, rs1: self.rs.0 }.encode(writer)
    }
}

impl CsrImmWrite {
    pub fn encode(&self, writer: &mut impl io::Write) -> io::Result<()> {
        ::rvhwfuzzer_encoding::CsrrwiArgs { csr: self.id, rd: 0, uimm: self.uimm }.encode(writer)
    }
}

csrs! {
    Frm    = 0x001 (write),
    Fflags = 0x002 (write),
    Fcsr   = 0x003 (write),
}
