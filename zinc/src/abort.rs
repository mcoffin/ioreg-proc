pub use core::intrinsics::breakpoint;

/// Call the debugger and halt execution
#[no_mangle]
pub extern fn abort() -> ! {
    unsafe {
        breakpoint();
    }
    loop {}
}
