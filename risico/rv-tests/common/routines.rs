#[inline(always)]
fn _mstatus_initialize_fs() {
    unsafe {
        core::arch::asm!(
            "csrw mstatus,{}",
            in(reg) 0b01 << 13,
            options(nomem),
        )
    }
}
