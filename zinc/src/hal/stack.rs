//! Stack layout information

use core::mem;

extern {
    fn __STACK_BASE();
    static mut __STACK_LIMIT: u32;
}

/// Returns the address of the main stack base (end of ram).
pub fn stack_base() -> u32 {
    unsafe {
        mem::transmute(__STACK_BASE as unsafe extern "C" fn())
    }
}

/// Returns the current stack limit
pub unsafe fn stack_limit() -> u32 {
    __STACK_LIMIT
}

/// Sets the current stack limit
pub unsafe fn set_stack_limit(val: u32) {
    __STACK_LIMIT = val;
}
