#[test]
fn basic_test() {
    println!("working?");
}

ioreg_proc::ioregs!(BASIC_TEST @ 0x0 = {
    0x0 => reg32 reg1 {
        0      => field1,
        1..3   => field2,
        16..24 => field3,
        25     => field4: (set_to_clear),
    },
    0x4 => reg32 reg2 {
        0      => field1,
    },
    0x8 => reg32 wo_reg {
        0..15  => field1: (wo),
        16..31 => field2: (wo),
    }
});
