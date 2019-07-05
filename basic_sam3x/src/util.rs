use cortex_m::peripheral::SYST;

macro_rules! wait_for {
    ($cond:expr) => {
        loop {
            if $cond {
                break;
            }
        }
    }
}

#[inline(never)]
pub fn systick_wait(syst: &mut SYST, mut ticks: u32) {
    let _dont_care = syst.has_wrapped();
    while ticks > 0 {
        wait_for!(syst.has_wrapped());
        ticks -= 1;
    }
}
