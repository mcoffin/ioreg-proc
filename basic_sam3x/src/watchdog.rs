//! Watchdog timer

/// Set enable state of watchdog timer
pub fn set_enabled(enabled: bool) {
    let wdc = unsafe { &reg::WDC };
    wdc.mr.update().set_wddis(!enabled);
}

/// Enable the watchdog timer
#[inline(always)]
pub fn enable() {
    set_enabled(true)
}

/// Disable the watchdog timer
#[inline(always)]
pub fn disable() {
    set_enabled(false)
}

mod reg {
    use ioreg_proc::ioregs;
    ioregs!(WDC = {
        0x0 => reg32 cr {
            0 => wdrstt: wo,
            24..31 => key: wo,
        },
        0x4 => reg32 mr {
            15 => wddis
        },
    });
    extern {
        #[link_name="sam3x_iomem_WDC"]
        pub static WDC: wdc::Wdc;
    }
}
