use std::mem;

ioreg_proc::ioregs!(FIELD_COUNTS_TEST @ 0x0 = {
    0x0 => reg32 reg1 {
        0..1 => field1[2],
        2..5 => field2[2] {
            0b00 => State1,
            0b01 => State2,
            0b11 => State3,
        },
        6..9 => field3[2]: wo,
    }
});

unsafe fn get_value<'a, T, P>(v: &'a T, offset: isize) -> P where
    T: Sized,
    P: Sized + Copy,
{
    let ptr: *const P = mem::transmute(v);
    *(ptr.offset(offset))
}
