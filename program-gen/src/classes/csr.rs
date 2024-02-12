use rvhwfuzzer_encoding::{CsrIndex, Instruction, XRegIdent};

pub struct CsrRead;
pub struct CsrWrite;
pub struct CsrImmWrite;

macro_rules! csrs {
    (
    $($name:ident = $id:literal $(($write_ident:ident))?),+ $(,)?
    ) => {
        static READ_CSR_IDS: [CsrIndex; 0 $(+ { stringify!($id); 1 })+] = [
            $(CsrIndex($id),)+
        ];

        static WRITE_CSR_IDS: [CsrIndex; 0 $($(+ { stringify!($write_ident); 1 })?)+] = [
            $($( { stringify!($write_ident); CsrIndex($id) }, )?)+
        ];

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
            fn take(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext) -> Instruction {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let rd = ctx.params_mut().take_register_dest();

                ::rvhwfuzzer_encoding::Csrrc::new(rd, READ_CSR_IDS[r], XRegIdent::Zero).into()
            }
        }

        impl $crate::arbitrary::ArbitraryInstruction for CsrWrite {
            fn take(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext) -> Instruction {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let rs = ctx.params_mut().take_register_src();

                ::rvhwfuzzer_encoding::Csrrw::new(XRegIdent::Zero, WRITE_CSR_IDS[r], rs).into()
            }
        }

        impl $crate::arbitrary::ArbitraryInstruction for CsrImmWrite {
            fn take(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext) -> Instruction {
                let r = (ctx.params_mut().take_u32(Self::NUM_CSRS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_CSRS { r - Self::NUM_CSRS } else { r };

                let uimm = ctx.params_mut().take_u8(5);

                ::rvhwfuzzer_encoding::Csrrwi::new(XRegIdent::Zero, WRITE_CSR_IDS[r], uimm).into()
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
