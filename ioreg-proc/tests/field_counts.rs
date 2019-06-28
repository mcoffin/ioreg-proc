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
    },
    0x4 => reg32 reg2 {
        0..31 => field1
    }
});

unsafe fn get_value<'a, T, P>(v: &'a T, offset: isize) -> P where
    T: Sized,
    P: Sized + Copy,
{
    let ptr: *const P = mem::transmute(v);
    *(ptr.offset(offset))
}

use field_counts_test::reg1::Field2;

#[no_mangle]
#[inline(never)]
fn set_field_test_reg1_state(test: &field_counts_test::FieldCountsTest, index: usize, value: Field2) {
    println!("[set] before: {:#b}", unsafe { get_value::<_, u32>(&test.reg1, 0) });
    test.reg1.update().set_field2(index, value);
    println!("[set] after: {:#b}", unsafe { get_value::<_, u32>(&test.reg1, 0) });
}

#[no_mangle]
#[inline(never)]
fn get_field_test_reg1_state(test: &field_counts_test::FieldCountsTest, index: usize) -> Field2 {
    println!("[get] value: {:#b}", unsafe { get_value::<_, u32>(&test.reg1, 0) });
    test.reg1.get().field2(index)
}

#[test]
fn round_trip_counted_fields() {
    let test: field_counts_test::FieldCountsTest = unsafe { mem::zeroed() };
    assert_eq!(test.reg1.get().field2(0), Field2::State1);
    assert_eq!(test.reg1.get().field2(1), Field2::State1);
    //assert_eq!(unsafe { get_value::<_, Field2>(&test.reg1, 4) }, Field2::State1);
    set_field_test_reg1_state(&test, 1, Field2::State2);
    assert_eq!(get_field_test_reg1_state(&test, 1), Field2::State2);
    assert_eq!(unsafe { get_value::<_, u32>(&test.reg1, 0) }, 0b10000);
    assert_eq!(test.reg1.get().field2(0), Field2::State1);
    assert_eq!(test.reg1.get().field2(1), Field2::State2);
}
