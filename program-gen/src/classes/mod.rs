macro_rules! define_instruction_class {
    (@internal $name:ident { $($instr_name:ident($args_name:ident)),+ $(,)? }) => {
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
    };
    ($name:ident { $($instr_name:ident($args_name:ident)),+ $(,)?  } |$ctx:ident| $is_available:expr) => {
        define_instruction_class!(@internal $name { $($instr_name($args_name)),+ });

        impl $crate::arbitrary::ArbitraryContextualInstruction for $name {
            #[inline]
            fn try_take($ctx: &mut $crate::arbitrary::ArbitraryGenerationContext) -> Option<::rvhwfuzzer_encoding::Instruction> {
                if (!$is_available) {
                    return None;
                }

                let r = ($ctx.params_mut().take_u32(Self::NUM_INSTRUCTIONS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_INSTRUCTIONS { r - Self::NUM_INSTRUCTIONS } else { r };

                static LUT: [fn(&mut $crate::arbitrary::ArbitraryGenerationContext) -> Option<::rvhwfuzzer_encoding::Instruction>; 0 $(+ { stringify!($args_name); 1 })+] = [
                    $(
                    <::rvhwfuzzer_encoding::$args_name as $crate::arbitrary::ArbitraryContextualInstruction>::try_take, 
                    )+
                ];

                (LUT[r])($ctx)
            }
        }

    };

    ($name:ident { $($instr_name:ident($args_name:ident)),+ $(,)?}) => {
        define_instruction_class!(@internal $name { $($instr_name($args_name)),+ });

        impl $crate::arbitrary::ArbitraryInstruction for $name {
            #[inline]
            fn take(ctx: &mut $crate::arbitrary::ArbitraryGenerationContext) -> ::rvhwfuzzer_encoding::Instruction {
                let r = (ctx.params_mut().take_u32(Self::NUM_INSTRUCTIONS.ilog2() + 1) as usize);
                let r = if r >= Self::NUM_INSTRUCTIONS { r - Self::NUM_INSTRUCTIONS } else { r };

                static LUT: [fn(&mut $crate::arbitrary::ArbitraryGenerationContext) -> ::rvhwfuzzer_encoding::Instruction; 0 $(+ { stringify!($args_name); 1 })+] = [
                    $(
                    <::rvhwfuzzer_encoding::$args_name as $crate::arbitrary::ArbitraryInstruction>::take, 
                    )+
                ];

                (LUT[r])(ctx)
            }
        }

    };
}

pub mod nonhopping_branches;
pub mod fpu32;
pub mod alu;
pub mod muldiv;
pub mod csr;
pub mod memory;
