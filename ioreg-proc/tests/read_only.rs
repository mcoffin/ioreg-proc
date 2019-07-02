use std::mem;

ioreg_proc::ioregs!(READONLY_TEST = {
    0x0 => reg8 reg1 {
        0 => field1: ro,
    },
});

#[test]
fn can_read_readonly() {
    let test: readonly_test::ReadonlyTest = unsafe { mem::zeroed() };
    assert_eq!(test.reg1.get().field1(), false);
}
