#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(oxide_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use oxide_os::println;
extern crate alloc;

/// This is called on panic. ! indicated that it is a diverging function
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    oxide_os::hlt_loop()
}

/// This is the panic handler in test mode, so we print the panic message to serial
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    oxide_os::test_panic_handler(info);
}

entry_point!(kernel_main);
/// No mangle keeps the rust compiler from outputting a mangled function name
/// extern "C" tells the rust compiler to use the C calling convention for the function, instead of Rust's
/// The method is called _start becuase that's what the linker is looking for by convention
#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use oxide_os::{allocator, memory};
    use x86_64::VirtAddr;
	println!("Hello World!");

    oxide_os::init(boot_info);

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator =
        unsafe { memory::init_allocator(&boot_info.memory_map) };
	allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

	println!("Didn't crash!");

    oxide_os::hlt_loop()
}
