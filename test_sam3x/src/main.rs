#![feature(start)]
#![no_std]

extern crate zinc;
extern crate zinc_macro;

mod util;

use zinc::hal::cortex_m3::systick;
use zinc::hal::pin::{GpioConf, GpioDirection, Gpio};
use zinc_macro::zinc_main;

const WAIT_TIME: u32 = 1000;

static LED_CONF: GpioConf = GpioConf {
    index: 26 + 32,
    direction: GpioDirection::Out,
};

#[zinc_main]
fn main() {
    use zinc::hal::sam3x::watchdog;
    use zinc::hal::sam3x::pin;
    use zinc::hal::sam3x::system_clock;

    // let mclk = system_clock::init_default();

    // systick::setup(mclk / 1000);
    // systick::enable();

    // watchdog::disable();

    let led = pin::Pin::from(LED_CONF);
    led.set_low();
}

#[inline(never)]
fn real_main<P: Gpio>(led: &P) {
    led.set_low();
    loop {
        util::systick_wait(WAIT_TIME);
        led.set_high();
        util::systick_wait(WAIT_TIME);
        led.set_low();
    }
}
