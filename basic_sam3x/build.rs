use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // First, copy linker scripts
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", &out_dir);
    let out_dir: &Path = out_dir.as_ref();
    fs::copy("./src/memory.x", out_dir.join("memory.x"))
        .expect("Failed to copy memory layout linker script");
    fs::copy("./src/device.x", out_dir.join("device.x"))
        .expect("Failed to copy device linker script");

    // Now, add the libsam library
    println!("cargo:rustc-link-search=native=/home/mcoffin/.arduino15/packages/arduino/hardware/sam/1.6.12/variants/arduino_due_x");
    println!("cargo:rustc-link-lib=static=sam_sam3x8e_gcc_rel");
}
