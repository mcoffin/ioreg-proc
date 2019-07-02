mod reg {
    use ioreg_proc::ioregs;
    ioregs!(INVERTED_RANGE_TEST = {
        0x0 => reg32 reg1 {
            31..0 => field1,
        }
    });
    use inverted_range_test::InvertedRangeTest;
}

#[test]
fn inverted_range() {
}
