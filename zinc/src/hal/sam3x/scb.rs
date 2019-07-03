extern "C" {
    fn ISRVectors();
}

#[inline(always)]
pub fn set_vector_offset() {
    let scb = unsafe { &reg::SCB };
    scb.vtor.update()
        .set_tbloff((ISRVectors as unsafe extern "C" fn()) as u32);
}

mod reg {
    use ioreg_proc::ioregs;
    ioregs!(SCB = {
        0x0 => reg32 cpuid {
            0..31 => value: ro,
        },
        0x4 => reg32 icsr {
            0..31 => value,
        },
        0x8 => reg32 vtor {
            0..31 => tbloff,
        },
    });
    extern {
        #[link_name="sam3x_iomem_SCB"]
        pub static SCB: scb::Scb;
    }
}
