#![allow(dead_code)]

use core::option::Option::{self, Some};

#[no_mangle]
pub extern "C" fn isr_dummy_handler() {
}

extern {
    fn isr_supc();
    fn isr_rstc();
    fn isr_rtc();
    fn isr_rtt();
    fn isr_wdt();
    fn isr_pmc();
    fn isr_efc0();
    fn isr_efc1();
    fn isr_uart();
    fn isr_smc();
    fn isr_dramc();
    fn isr_pioa();
    fn isr_piob();
    fn isr_pioc();
    fn isr_piod();
    fn isr_pioe();
    fn isr_piof();
    fn isr_usart0();
    fn isr_usart1();
    fn isr_usart2();
    fn isr_usart3();
    fn isr_hsmci();
    fn isr_twi0();
    fn isr_twi1();
    fn isr_spi0();
    fn isr_spi1();
    fn isr_ssc();
    fn isr_tc0();
    fn isr_tc1();
    fn isr_tc2();
    fn isr_tc3();
    fn isr_tc4();
    fn isr_tc5();
    fn isr_tc6();
    fn isr_tc7();
    fn isr_tc8();
    fn isr_pwm();
    fn isr_adc();
    fn isr_dacc();
    fn isr_dmac();
    fn isr_utoghs();
    fn isr_trng();
    fn isr_emac();
    fn isr_can0();
    fn isr_can1();
}

#[allow(non_upper_case_globals)]
const ISRCount: usize = 45;

#[allow(non_upper_case_globals)]
#[link_section=".isr_vector_nvic"]
#[no_mangle]
pub static NVICVectors: [Option<unsafe extern fn()>; ISRCount] = [
    Some(isr_supc),
    Some(isr_rstc),
    Some(isr_rtc),
    Some(isr_rtt),
    Some(isr_wdt),
    Some(isr_pmc),
    Some(isr_efc0),
    Some(isr_efc1),
    Some(isr_uart),
    Some(isr_smc),
    None, //Some(isr_dramc),
    Some(isr_pioa),
    Some(isr_piob),
    Some(isr_pioc),
    Some(isr_piod),
    None, //Some(isr_pioe),
    None, //Some(isr_piof),
    Some(isr_usart0),
    Some(isr_usart1),
    Some(isr_usart2),
    Some(isr_usart3),
    Some(isr_hsmci),
    Some(isr_twi0),
    Some(isr_twi1),
    Some(isr_spi0),
    None, //Some(isr_spi1),
    Some(isr_ssc),
    Some(isr_tc0),
    Some(isr_tc1),
    Some(isr_tc2),
    Some(isr_tc3),
    Some(isr_tc4),
    Some(isr_tc5),
    Some(isr_tc6),
    Some(isr_tc7),
    Some(isr_tc8),
    Some(isr_pwm),
    Some(isr_adc),
    Some(isr_dacc),
    Some(isr_dmac),
    Some(isr_utoghs),
    Some(isr_trng),
    Some(isr_emac),
    Some(isr_can0),
    Some(isr_can1),
];
