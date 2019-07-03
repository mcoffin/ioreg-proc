// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::option::Option;
use core::option::Option::{Some, None};

extern {
  fn main(_: isize, _: *const *const u8) -> isize;
  fn __STACK_BASE();

  fn isr_nmi();
  fn isr_hardfault();
  fn isr_mmfault();
  fn isr_busfault();
  fn isr_usagefault();

  fn isr_svcall();
  fn isr_pendsv();
  fn isr_systick();

  fn isr_debugmon();
  fn isr_reserved_1();
}

#[no_mangle]
pub extern "C" fn isr_systick_default() {
}

unsafe extern "C" fn reset_handler() {
    use core::ptr;
    use ::hal::mem_init;

    mem_init::init_stack();
    mem_init::init_data();

    #[cfg(feature = "mcu_sam3x")]
    ::hal::sam3x::scb::set_vector_offset();

    let _status = main(0, ptr::null());

    loop {
        asm!("nop" :::: "volatile");
    }
}

#[no_mangle]
pub unsafe extern fn isr_handler_wrapper() {
  asm!(".weak isr_nmi, isr_hardfault, isr_mmfault, isr_busfault
      .weak isr_usagefault, isr_svcall, isr_pendsv
      .weak isr_debugmon
      .weak isr_reserved_1

      .thumb_func
      isr_svcall:
      b isr_pendsv

      .thumb_func
      isr_pendsv:
      b.n isr_pendsv

      .thumb_func
      isr_nmi:

      .thumb_func
      isr_hardfault:

      .thumb_func
      isr_mmfault:

      .thumb_func
      isr_busfault:

      .thumb_func
      isr_usagefault:

      .thumb_func
      isr_debugmon:

      b isr_default_fault

      .thumb_func
      isr_default_fault:
      mrs r0, psp
      mrs r1, msp
      ldr r2, [r0, 0x18]
      ldr r3, [r1, 0x18]
      bkpt" :::: "volatile");
}

#[allow(non_upper_case_globals)]
const ISRCount: usize = 16;

#[link_section=".isr_vector"]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static ISRVectors: [Option<unsafe extern fn()>; ISRCount] = [
  Some(__STACK_BASE),
  Some(reset_handler),             // Reset
  Some(isr_nmi),          // NMI
  Some(isr_hardfault),    // Hard Fault
  Some(isr_mmfault),      // CM3 Memory Management Fault
  Some(isr_busfault),     // CM3 Bus Fault
  Some(isr_usagefault),   // CM3 Usage Fault
  Some(isr_reserved_1),   // Reserved - Used as NXP Checksum
  None,                   // Reserved
  None,                   // Reserved
  None,                   // Reserved
  Some(isr_svcall),       // SVCall
  Some(isr_debugmon),     // Reserved for debug
  None,                   // Reserved
  Some(isr_pendsv),       // PendSV
  Some(isr_systick),      // SysTick
];
