use rvhwfuzzer_encoding::{CsrIndex, Instruction, XRegIdent};

pub struct CsrRead;
pub struct CsrWrite;
pub struct CsrImmWrite;

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
            fn take<P: $crate::arbitrary::ArbitraryParameterProvider>(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext<P>) -> Instruction {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let rd = ctx.params_mut().take_register_dest();

                let mut i = 0;
                $(
                    if i == r {
                        return ::rvhwfuzzer_encoding::Csrrc::new(rd, CsrIndex($id), XRegIdent::Zero).into();
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
            fn take<P: $crate::arbitrary::ArbitraryParameterProvider>(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext<P>) -> Instruction {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let rs = ctx.params_mut().take_register_src();

                let mut i = 0;
                $($(
                    if i == r {
                        stringify!($write_ident);
                        return ::rvhwfuzzer_encoding::Csrrw::new(XRegIdent::Zero, CsrIndex($id), rs).into();
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
            fn take<P: $crate::arbitrary::ArbitraryParameterProvider>(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext<P>) -> Instruction {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let uimm = ctx.params_mut().take_u8(5);

                let mut i = 0;
                $($(
                    if i == r {
                        stringify!($write_ident);
                        return ::rvhwfuzzer_encoding::Csrrwi::new(XRegIdent::Zero, CsrIndex($id), uimm).into();
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

csrs! {
    Frm       = 0x001 (write),
    Fflags    = 0x002 (write),
    Fcsr      = 0x003 (write),

    MvendorId = 0xF11,
    MarchId   = 0xF12,
    MimpId    = 0xF13,
    MhartId   = 0xF14,

    Mstatus   = 0x300 (write),
    Misa      = 0x301 (write),
    Mstatush  = 0x310 (write),
}
