extern "C" {
    #[link_name="SystemInit"]
    pub fn system_init();

    #[link_name="SystemCoreClock"]
    static mut SYSTEM_CORE_CLOCK: u32;
}

#[inline(always)]
pub fn system_core_clock() -> u32 {
    unsafe { SYSTEM_CORE_CLOCK }
}
