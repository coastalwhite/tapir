define_instruction_class! {
    MemoryInstruction {
        Lb(Lb),
        Lh(Lh),
        Lw(Lw),
        Lbu(Lbu),
        Lhu(Lhu),
        Sb(Sb),
        Sh(Sh),
        Sw(Sw),
    }
    |ctx| ctx.take_memory_register().is_some()
}
