#![feature(asm, start)]
#![no_std]

extern crate zinc;
extern crate zinc_macro;

#[zinc_macro::zinc_main]
fn main() {
    use zinc::hal::mem_init::{init_data, init_stack};
    init_data();
    unsafe { init_stack() };
    unsafe { asm!("nop") }
}
