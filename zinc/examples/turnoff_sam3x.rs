#![feature(start, asm)]
#![no_std]

extern crate zinc;

use zinc::hal::pin::{ Gpio, GpioConf, GpioDirection };

const LED_CONF: GpioConf = GpioConf {
    index: 27 + 32,
    direction: GpioDirection::Out,
};

#[start]
fn start(_: isize, _: *const *const u8) -> isize {
    use zinc::hal::mem_init;
    unsafe { mem_init::init_stack() };
    mem_init::init_data();

    use zinc::hal::sam3x::watchdog;
    watchdog::disable();

    let led = zinc::hal::sam3x::pin::Pin::from(LED_CONF);
    led.set_low();

    0
}
