define_instruction_class! {
    FPU32Instruction {
        FmaddS(FmaddS),
        FmsubS(FmsubS),
        FnmaddS(FnmaddS),
        FnmsubS(FnmsubS),
        FaddS(FaddS),
        FsubS(FsubS),
        FmulS(FmulS),
        FdivS(FdivS),
        FsqrtS(FsqrtS),
        FsgnjS(FsgnjS),
        FsgnjnS(FsgnjnS),
        FsgnjxS(FsgnjxS),
        FminS(FminS),
        FmaxS(FmaxS),
        FcvtWS(FcvtWS),
        FcvtWuS(FcvtWuS),
        FmvXW(FmvXW),
        FeqS(FeqS),
        FltS(FltS),
        FleS(FleS),
        FclassS(FclassS),
        FcvtSW(FcvtSW),
        FcvtSWu(FcvtSWu),
        FmvWX(FmvWX),
    }
    |ctx| ctx.state().are_f_ext_instrs_available()
}
