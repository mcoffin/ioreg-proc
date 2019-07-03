//! Helper functions for memory initialization

use super::stack;
use core::mem;

extern "C" {
    static _data_load: u32;
    static mut _data: u32;
    static mut _edata: u32;
    static mut _bss: u32;
    static mut _ebss: u32;

    static _eglobals: u32;
}

/// Helper function to initialize the stack limit.
#[inline(always)]
pub fn init_stack() {
    unsafe {
        stack::set_stack_limit((&_eglobals as *const u32) as u32);
    };
}

/// Helper function to initialize memory.
///
/// Copies `.data` sections to RAM and initializes `.bss` sections to zero.
#[inline(always)]
pub fn init_data() {
    unsafe {
        let mut load_addr: *const u32 = &_data_load;
        let mut mem_addr: *mut u32 = &mut _data;
        while mem_addr < &mut _edata as *mut u32 {
            *mem_addr = *load_addr;
            mem_addr = ((mem_addr as usize) + mem::size_of::<u32>()) as *mut u32;
            load_addr = ((load_addr as usize) + mem::size_of::<u32>()) as *const u32;
        }

        mem_addr = &mut _bss as *mut u32;
        while mem_addr < &mut _ebss as *mut u32 {
            *mem_addr = 0u32;
            mem_addr = ((mem_addr as usize) + mem::size_of::<u32>()) as *mut u32;
        }
    }
}
