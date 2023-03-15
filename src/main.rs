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
use x86_64::structures::paging::Translate;

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
    use rust_os::memory;
    use x86_64::{structures::paging::Page, VirtAddr};

    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = memory::EmptyFrameAllocator;

    // 映射未使用的页
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // 通过新的映射将字符串 `New!`  写到屏幕上。
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    // let addresses = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x201008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    #[cfg(test)]
    test_main();

    loop {
        // 在下一个中断触发之前休息一下，进入休眠状态来节省一点点能源
        rust_os::hlt_loop();
    }
}
