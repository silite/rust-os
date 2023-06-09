// no rust runtime, no standard library, no entry point, bug has C runtime
#![no_std] // don't link the Rust standard library
// don’t want to use the normal entry point chain
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_buffer;

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::allocator;
use rust_os::memory::BootInfoFrameAllocator;
use rust_os::task::simple_executor::SimpleExecutor;
use rust_os::task::Task;
use x86_64::VirtAddr;

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
             // bootloader初始化时，将启动信息传入 features = ["map_physical_memory"]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    rust_os::init();

    let phy_mom_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { rust_os::memory::init(phy_mom_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.run();

    #[cfg(test)]
    test_main();

    loop {
        // 在下一个中断触发之前休息一下，进入休眠状态来节省一点点能源
        rust_os::hlt_loop();
    }
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
