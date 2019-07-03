use zinc::hal::cortex_m3::systick;
use zinc::wait_for;

#[inline(never)]
pub fn systick_wait(mut ticks: u32) {
    systick::tick();
    while ticks > 0 {
        wait_for!(systick::tick());
        ticks -= 1;
    }
}
