#![feature(start, asm)]
#![no_std]
#![no_main]

extern crate zinc;

use zinc::hal::cortex_m3::systick;
use zinc::hal::sam3x::system_clock;
use zinc::hal::pin::{ GpioConf, GpioDirection, Gpio };
use zinc::wait_for;

const WAIT_TIME: u32 = 1000;

const LED_CONF: GpioConf = GpioConf {
    index: 27 + 32,
    direction: GpioDirection::Out,
};

#[no_mangle]
fn main(_: isize, _: *const *const u8) -> isize {
    use zinc::hal::mem_init;
    unsafe { mem_init::init_stack() };
    mem_init::init_data();

    let mck_freq = system_clock::init_clock(system_clock::ClockSource::InternalSlow, None);

    let pll = system_clock::Pll {
        mul: 0x1,
        div: 0x1,
        count: 0x3f,
    };
    let mck_freq = system_clock::init_clock(system_clock::ClockSource::Main(Some(12_000_000)), Some(pll));
    systick::setup(mck_freq / 1000);
    systick::enable();
    // system_clock::temp_boot();

    use zinc::hal::sam3x::watchdog;
    watchdog::disable();

    let led = ::zinc::hal::sam3x::pin::Pin::from(LED_CONF);
    real_main(&led)
}

#[inline(never)]
fn wait(mut ticks: u32) {
    systick::tick();
    while ticks > 0 {
        wait_for!(systick::tick());
        ticks -= 1;
    }
}

pub fn real_main<P: Gpio>(led: &P) -> isize {
    led.set_low();
    loop {
        wait(WAIT_TIME);
        led.set_high();
        wait(WAIT_TIME);
        led.set_low();
    }
    return 0;
}
