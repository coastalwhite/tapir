define_instruction_class! {
    AluInstruction "still" {
        Lui(Lui),
        Auipc(Auipc),
        Addi(Addi),
        Slti(Slti),
        Sltiu(Sltiu),
        Xori(Xori),
        Ori(Ori),
        Andi(Andi),
        Slli(Slli),
        Srli(Srli),
        Srai(Srai),
        Add(Add),
        Sub(Sub),
        Sll(Sll),
        Slt(Slt),
        Sltu(Sltu),
        Xor(Xor),
        Srl(Srl),
        Sra(Sra),
        Or(Or),
        And(And),
    }
}

define_instruction_class! {
    CAluInstruction "still" {
        CAddi16Sp(CAddi16Sp),
        CAddi4SpN(CAddi4SpN),
        CAddi(CAddi),
        CLi(CLi),
        CLui(CLui),
        CSrli(CSrli),
        CSrai(CSrai),
        CAndi(CAndi),
        CSub(CSub),
        CXor(CXor),
        COr(COr),
        CAnd(CAnd),
        CSlli(CSlli),
        CAdd(CAdd),
        CMv(CMv),
        CNop(CNop),
    }
    |ctx| ctx.state().are_c_ext_instrs_available()
}
