#![no_std]
#![feature(asm)]
#![feature(core_intrinsics, lang_items)]

#[cfg(target_os = "none")]
extern crate rlibc;
extern crate ioreg_proc;
extern crate volatile_cell;

pub mod abort;
pub mod hal;
pub mod panic;
#[macro_use]
pub mod util;

#[lang = "eh_personality"]
extern fn zinc_rust_eh_personality() {
}
