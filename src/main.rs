// no rust runtime, no standard library, no entry point, bug has C runtime
#![no_std] // don't link the Rust standard library
// don’t want to use the normal entry point chain
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_buffer;

use core::panic::PanicInfo;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    println!("{}", info);
    loop {
        rust_os::hlt_loop();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

/*
 * freestanding executable does not have access to the Rust runtime and crt0,
 * so we need to define our own entry point.
 * Implementing the start language item wouldn’t help,
 * since it would still require crt0.
 * Instead, we need to overwrite the crt0 entry point directly.
 */

// overwriting the operating system entry point with our own _start function:
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    rust_os::init();
    println!("test{}", "!");

    #[cfg(test)]
    test_main();

    loop {
        // 在下一个中断触发之前休息一下，进入休眠状态来节省一点点能源
        rust_os::hlt_loop();
    }
}
