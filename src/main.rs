// no rust runtime, no standard library, no entry point, bug has C runtime
#![no_std] // don't link the Rust standard library
// don’t want to use the normal entry point chain
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_buffer;

use bootloader::{entry_point, BootInfo};
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

// 宏为定义了真正的低级_start入口点
entry_point!(kernel_main);

// overwriting the operating system entry point with our own _start function:
#[no_mangle] // don't mangle the name of this function
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory::active_level_4_table;
    use x86_64::{structures::paging::PageTable, VirtAddr};

    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };
    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);

            let phys = entry.frame().unwrap().start_address();
            let virt = phys.as_u64() + boot_info.physical_memory_offset;
            let ptr = VirtAddr::new(virt).as_ptr();
            let l3_table: &PageTable = unsafe { &*ptr };
            for (i, entry) in l3_table.iter().enumerate() {
                if !entry.is_unused() {
                    println!("  L3 Entry {}: {:?}", i, entry);
                }
            }
        }
    }

    #[cfg(test)]
    test_main();

    loop {
        // 在下一个中断触发之前休息一下，进入休眠状态来节省一点点能源
        rust_os::hlt_loop();
    }
}
