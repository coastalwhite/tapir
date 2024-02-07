use super::{ArbitraryInstruction, ArbitraryParameterProvider, ArbitraryGenerationContext};

macro_rules! impl_arbitrary_args {
    ($name:ident { [$ctx:ident] $($arg:ident: $arg_expr:expr),* $(,)? }) => {
        impl ArbitraryInstruction for ::rvhwfuzzer_encoding::$name {
            fn take<P: ArbitraryParameterProvider>($ctx: &mut ArbitraryGenerationContext<P>) -> Self {
                Self { $( $arg: $arg_expr, )+ }
            }
        }
    }
}

macro_rules! impl_fpu32_r4_args {
    ($($name:ident),+ $(,)?) => {
        $( impl_arbitrary_args! {
            $name {
                [ctx]
                rs1: ctx.params_mut().take_fpu_register_src().0,
                rs2: ctx.params_mut().take_fpu_register_src().0,
                rs3: ctx.params_mut().take_fpu_register_src().0,
                rm: ctx.params_mut().take_static_rounding_mode() as u8,
                rd: ctx.params_mut().take_fpu_register_dest().0,
            }
        } )+
    };
}

macro_rules! impl_fpu32_r_rm_args {
    ($($name:ident),+ $(,)?) => {
        $( impl_arbitrary_args! {
            $name {
                [ctx]
                rs1: ctx.params_mut().take_fpu_register_src().0,
                rs2: ctx.params_mut().take_fpu_register_src().0,
                rm: ctx.params_mut().take_static_rounding_mode() as u8,
                rd: ctx.params_mut().take_fpu_register_dest().0,
            }
        } )+
    };
}

macro_rules! impl_fpu32_r_no_rm_args {
    ($($name:ident),+ $(,)?) => {
        $( impl_arbitrary_args! {
            $name {
                [ctx]
                rs1: ctx.params_mut().take_fpu_register_src().0,
                rs2: ctx.params_mut().take_fpu_register_src().0,
                rd: ctx.params_mut().take_fpu_register_dest().0,
            }
        } )+
    };
}

macro_rules! impl_fpu32_cmp_args {
    ($($name:ident),+ $(,)?) => {
        $( impl_arbitrary_args! {
            $name {
                [ctx]
                rs1: ctx.params_mut().take_fpu_register_src().0,
                rs2: ctx.params_mut().take_fpu_register_src().0,
                rd: ctx.params_mut().take_register_dest().0,
            }
        } )+
    };
}

impl_fpu32_r4_args! {
    FmaddSArgs,
    FmsubSArgs,
    FnmaddSArgs,
    FnmsubSArgs,
}

impl_fpu32_r_rm_args! {
    FaddSArgs,
    FsubSArgs,
    FmulSArgs,
    FdivSArgs,
}

impl_fpu32_r_no_rm_args! {
    FminSArgs,
    FmaxSArgs,
    FsgnjSArgs,
    FsgnjnSArgs,
    FsgnjxSArgs,
}

impl_fpu32_cmp_args! {
    FeqSArgs,
    FltSArgs,
    FleSArgs,
}

impl_arbitrary_args! {
    FsqrtSArgs {
        [ctx]
        rs1: ctx.params_mut().take_fpu_register_src().0,
        rm: ctx.params_mut().take_static_rounding_mode() as u8,
        rd: ctx.params_mut().take_fpu_register_dest().0,
    }
}

impl_arbitrary_args! {
    FclassSArgs {
        [ctx]
        rs1: ctx.params_mut().take_fpu_register_src().0,
        rd: ctx.params_mut().take_register_dest().0,
    }
}

impl_arbitrary_args! {
    FmvWXArgs {
        [ctx]
        rs1: ctx.params_mut().take_register_src().0,
        rd: ctx.params_mut().take_fpu_register_dest().0,
    }
}

impl_arbitrary_args! {
    FmvXWArgs {
        [ctx]
        rs1: ctx.params_mut().take_register_src().0,
        rd: ctx.params_mut().take_fpu_register_dest().0,
    }
}

impl_arbitrary_args! {
    FcvtWSArgs {
        [ctx]
        rs1: ctx.params_mut().take_register_src().0,
        rm: ctx.params_mut().take_static_rounding_mode() as u8,
        rd: ctx.params_mut().take_fpu_register_dest().0,
    }
}

impl_arbitrary_args! {
    FcvtWuSArgs {
        [ctx]
        rs1: ctx.params_mut().take_register_src().0,
        rm: ctx.params_mut().take_static_rounding_mode() as u8,
        rd: ctx.params_mut().take_fpu_register_dest().0,
    }
}

impl_arbitrary_args! {
    FcvtSWArgs {
        [ctx]
        rs1: ctx.params_mut().take_register_src().0,
        rm: ctx.params_mut().take_static_rounding_mode() as u8,
        rd: ctx.params_mut().take_fpu_register_dest().0,
    }
}

impl_arbitrary_args! {
    FcvtSWuArgs {
        [ctx]
        rs1: ctx.params_mut().take_register_src().0,
        rm: ctx.params_mut().take_static_rounding_mode() as u8,
        rd: ctx.params_mut().take_fpu_register_dest().0,
    }
}
