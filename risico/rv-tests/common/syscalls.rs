#[inline(always)]
fn _syscall_exit(returncode: i32) -> ! {
    unsafe {
        ::core::arch::asm!(
            "ecall",
            in("x17") 0,
            in("x10") returncode,
            options(noreturn),
        )
    }
}

pub trait AssertType {
    const ASSERT_TYPE_VALUE: u32;
    fn as_u32(self) -> u32;
}

impl AssertType for u32 {
    const ASSERT_TYPE_VALUE: u32 = 1;

    #[inline(always)]
    fn as_u32(self) -> u32 {
        self
    }
}
impl AssertType for i32 {
    const ASSERT_TYPE_VALUE: u32 = 2;

    #[inline(always)]
    fn as_u32(self) -> u32 {
        self as u32
    }
}
impl AssertType for f32 {
    const ASSERT_TYPE_VALUE: u32 = 3;

    #[inline(always)]
    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[inline(always)]
fn _syscall_assert_eq_unknown(a: u32, b: u32) {
    unsafe {
        ::core::arch::asm!(
            "ecall",
            in("x17") 4,
            in("x10") 0,
            in("x11") a,
            in("x12") b,
            options(nomem),
        )
    }
}

#[inline(always)]
fn _syscall_assert_eq<T: AssertType>(a: T, b: T) {
    unsafe {
        ::core::arch::asm!(
            "ecall",
            in("x17") 4,
            in("x10") T::ASSERT_TYPE_VALUE,
            in("x11") a.as_u32(),
            in("x12") b.as_u32(),
            options(nomem),
        )
    }
}
