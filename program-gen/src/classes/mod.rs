macro_rules! define_instruction_class {
    ($name:ident { $($instr_name:ident($args_name:ident)),+ $(,)? }) => {
        #[derive(Debug)]
        pub enum $name {
            $(
            $instr_name(::rvhwfuzzer_encoding::$args_name),
            )+
        }

        impl $name {
            const NUM_INSTRUCTIONS: usize = {
                0
                $(+ 1 + (::std::mem::size_of::<::rvhwfuzzer_encoding::$args_name>() * 0))+
            };

            pub fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
                match self {
                    $(
                    Self::$instr_name(v) => v.encode(writer),
                    )+
                }
            }
        }

        impl From<$name> for ::rvhwfuzzer_encoding::Instruction {
            fn from(value: $name) -> Self {
                match value {
                    $(
                    $name::$instr_name(v) => Self::$instr_name(v),
                    )+
                }
            }
        }

        impl $crate::arbitrary::ArbitraryInstruction for $name {
            fn take<P: $crate::arbitrary::ArbitraryParameterProvider>(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext<P>) -> ::rvhwfuzzer_encoding::Instruction {
                let r = (ctx.params_mut().take_u32(Self::NUM_INSTRUCTIONS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_INSTRUCTIONS { r - Self::NUM_INSTRUCTIONS } else { r };

                let mut i = 0;
                $(
                    if i == r {
                        return <::rvhwfuzzer_encoding::$args_name as $crate::arbitrary::ArbitraryInstruction>::take(ctx).into();
                    }
                    #[allow(unused_assignments)]
                    {
                        i += 1;
                    }
                )+

                unreachable!();
            }
        }

    };
}

pub mod nonhopping_branches;
pub mod fpu32;
pub mod alu;
pub mod csr;
