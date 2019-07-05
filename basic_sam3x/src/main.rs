#![feature(asm)]
#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_halt;
extern crate volatile_cell;

pub mod hal;
mod isr;
mod sam;
pub mod watchdog;
pub mod pin;
pub mod peripheral_clock;
mod util;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

const LED_BUILTIN_CONF: hal::pin::GpioConf = hal::pin::GpioConf {
    index: 27 + 32,
    direction: hal::pin::GpioDirection::Out,
};

#[entry]
fn main() -> ! {
    use hal::pin::Gpio;

    unsafe {
        sam::system_init();
    }

    watchdog::disable();

    let mut peripherals = Peripherals::take().unwrap();
    peripherals.SYST.set_reload(sam::system_core_clock() / 1000);
    peripherals.SYST.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    peripherals.SYST.clear_current();
    peripherals.SYST.enable_counter();

    let led = pin::Pin::from(LED_BUILTIN_CONF);
    real_main(&led, &mut peripherals)
}

fn real_main<P>(led: &P, peripherals: &mut Peripherals) -> ! where
    P: hal::pin::Gpio,
{
    loop {
        led.set_high();
        util::systick_wait(&mut peripherals.SYST, 1000);
        led.set_low();
        util::systick_wait(&mut peripherals.SYST, 1000);
    }
}
