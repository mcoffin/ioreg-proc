use volatile_cell::VolatileCell;

use std::mem;

unsafe fn get_value_u32<'a, T>(v: &'a T, offset: usize) -> u32 {
    let ptr: *const u32 = mem::transmute(v);
    *(ptr.offset(offset as isize))
}

#[test]
fn round_trip_simple_field_values_1() {
    let test: BASIC_TEST::BASIC_TEST = unsafe { mem::zeroed() };
    assert_eq!(test.reg1.get().field1(), false);
    test.reg1.update().set_field1(true);
    assert_eq!(test.reg1.get().field1(), true);
    assert_eq!(unsafe { get_value_u32(&test, 0) }, 0x1);
    assert_eq!(unsafe { get_value_u32(&test, 1) }, 0x0);
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

ioreg_proc::ioregs!(MULTI_PROP_TEST @ 0x0 = {
    0x0 => reg32 reg1 {
        0      => field1,
        1..3   => field2,
        16..24 => field3,
        25     => field4: (set_to_clear, wo),
    },
});

ioreg_proc::ioregs!(VARIANT_TEST @ 0x0 = {
    0x0    => reg32 cr {
        0      => rxe,
        1      => txe,
        2      => rxie,
        3      => txie,
        4..12  => br,
        14..16 => parity {
          0x0  => NoParity,
          0x2  => EvenParity,
          0x3  => OddParity,
        }
    },
    0x4    => reg32 sr {
        0      => rxne: ro,
        1      => txe: ro,
        2      => fe: set_to_clear,
    },
    0x8    => reg32 dr {
        0..7   => d
    }
});

// ioreg_proc::ioregs!(GROUP_TEST @ 0 = {
//     0x0 => group regs[5] {
//         0x0 => reg32 reg1 {
//             0..31 => field1
//         },
//         0x4 => reg32 reg2 {
//             0..31 => field2
//         }
//     }
// });
