use super::{ArbitraryInstruction, ArbitraryGenerationContext};

macro_rules! impl_arbitrary_args {
    ($name:ident { [$ctx:ident] $($arg:expr),* $(,)? }) => {
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
            fn take($ctx: &mut ArbitraryGenerationContext) -> ::rvhwfuzzer_encoding::Instruction {
                Self::new($( $arg, )+).into()
            }
        }
    }
}

macro_rules! impl_fpu32_r4_args {
    ($($name:ident),+ $(,)?) => {
        $( impl_arbitrary_args! {
            $name {
                [ctx]
                ctx.params_mut().take_fpu_register_dest(),
                ctx.params_mut().take_fpu_register_src(),
                ctx.params_mut().take_fpu_register_src(),
                ctx.params_mut().take_fpu_register_src(),
                ctx.params_mut().take_static_rounding_mode(),
            }
        } )+
    };
}

macro_rules! impl_fpu32_r_rm_args {
    ($($name:ident),+ $(,)?) => {
        $( impl_arbitrary_args! {
            $name {
                [ctx]
                ctx.params_mut().take_fpu_register_dest(),
                ctx.params_mut().take_fpu_register_src(),
                ctx.params_mut().take_fpu_register_src(),
                ctx.params_mut().take_static_rounding_mode(),
            }
        } )+
    };
}

macro_rules! impl_fpu32_r_no_rm_args {
    ($($name:ident),+ $(,)?) => {
        $( impl_arbitrary_args! {
            $name {
                [ctx]
                ctx.params_mut().take_fpu_register_dest(),
                ctx.params_mut().take_fpu_register_src(),
                ctx.params_mut().take_fpu_register_src(),
            }
        } )+
    };
}

macro_rules! impl_fpu32_cmp_args {
    ($($name:ident),+ $(,)?) => {
        $( impl_arbitrary_args! {
            $name {
                [ctx]
                ctx.params_mut().take_register_dest(),
                ctx.params_mut().take_fpu_register_src(),
                ctx.params_mut().take_fpu_register_src(),
            }
        } )+
    };
}

impl_fpu32_r4_args! {
    FmaddS,
    FmsubS,
    FnmaddS,
    FnmsubS,
}

impl_fpu32_r_rm_args! {
    FaddS,
    FsubS,
    FmulS,
    FdivS,
}

impl_fpu32_r_no_rm_args! {
    FminS,
    FmaxS,
    FsgnjS,
    FsgnjnS,
    FsgnjxS,
}

impl_fpu32_cmp_args! {
    FeqS,
    FltS,
    FleS,
}

impl_arbitrary_args! {
    FsqrtS {
        [ctx]
        ctx.params_mut().take_fpu_register_dest(),
        ctx.params_mut().take_fpu_register_src(),
        ctx.params_mut().take_static_rounding_mode(),
    }
}

impl_arbitrary_args! {
    FclassS {
        [ctx]
        ctx.params_mut().take_register_dest(),
        ctx.params_mut().take_fpu_register_src(),
    }
}

impl_arbitrary_args! {
    FmvWX {
        [ctx]
        ctx.params_mut().take_fpu_register_dest(),
        ctx.params_mut().take_register_src(),
    }
}

impl_arbitrary_args! {
    FmvXW {
        [ctx]
        ctx.params_mut().take_register_dest(),
        ctx.params_mut().take_fpu_register_src(),
    }
}

impl_arbitrary_args! {
    FcvtWS {
        [ctx]
        ctx.params_mut().take_register_dest(),
        ctx.params_mut().take_fpu_register_src(),
        ctx.params_mut().take_static_rounding_mode(),
    }
}

impl_arbitrary_args! {
    FcvtWuS {
        [ctx]
        ctx.params_mut().take_register_dest(),
        ctx.params_mut().take_fpu_register_src(),
        ctx.params_mut().take_static_rounding_mode(),
    }
}

impl_arbitrary_args! {
    FcvtSW {
        [ctx]
        ctx.params_mut().take_fpu_register_dest(),
        ctx.params_mut().take_register_src(),
        ctx.params_mut().take_static_rounding_mode(),
    }
}

impl_arbitrary_args! {
    FcvtSWu {
        [ctx]
        ctx.params_mut().take_fpu_register_dest(),
        ctx.params_mut().take_register_src(),
        ctx.params_mut().take_static_rounding_mode(),
    }
}
