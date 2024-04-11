// @Note
// We cannot generate DIV or REM instructions if all registers are 0, we should also not
// select a register that has a zero value for the quotient I don't know whether that is messing
// stuff up at the moment.
define_instruction_class! {
    MulDivInstruction {
        Mul(Mul),
        MulH(MulH),
        MulHsu(MulHsu),
        MulHu(MulHu),
        Div(Div),
        DivU(DivU),
        Rem(Rem),
        RemU(RemU),
    }
}
