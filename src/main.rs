// no rust runtime, no standard library, no entry point, bug has C runtime
#![no_std] // don't link the Rust standard library
// don’t want to use the normal entry point chain
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    loop {}
}

/*
 * freestanding executable does not have access to the Rust runtime and crt0,
 * so we need to define our own entry point.
 * Implementing the start language item wouldn’t help,
 * since it would still require crt0.
 * Instead, we need to overwrite the crt0 entry point directly.
 */

static HELLO: &[u8] = b"Hello World!";

// overwriting the operating system entry point with our own _start function:
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}
