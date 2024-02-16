// option: march = imac
#![no_std]
#![no_main]


include!("../common/panic.rs");
include!("../common/syscalls.rs");

fn matrix_multiply(a: [[i32; 3]; 4], b: [[i32; 4]; 3]) -> [[i32; 4]; 4] {
    let mut result = [[0; 4]; 4];

    for y in 0..4 {
        for x in 0..4 {
            for i in 0..3 {
                result[y][x] += a[y][i] * b[i][x];
                // _syscall_write_value(result[y][x] as u32);
            }
        }
    }

    result
}

#[no_mangle]
fn _start() -> ! {
    let a = [
        [ 5,  3,  2],
        [ 8, 23,  2],
        [ 7, 12, 43],
        [21, 44,  9],
    ];
    let b = [
        [123, 42, 42,  23],
        [  5,  6,  1,  12],
        [  8, 23, 12, 213],
    ];

    const EXPECTED_RESULT: [[i32; 4]; 4] = [
        [646,   274,  237,  577],
        [1115,  520,  383,  886],
        [1265, 1355,  822, 9464],
        [2875, 1353, 1034, 2928],
    ];

    let result = matrix_multiply(a, b);

    for y in 0..4 {
        for x in 0..4 {
            _syscall_assert_eq(result[y][x], EXPECTED_RESULT[y][x]);
        }
    }

    _syscall_exit(0);
}