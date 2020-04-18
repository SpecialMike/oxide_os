#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(oxide_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use oxide_os::println;
use oxide_os::task::{Task, executor::Executor};
use oxide_os::task::keyboard;

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
    oxide_os::init(boot_info);

    #[cfg(test)]
	test_main();
	
	println!("Didn't crash!");

	let mut executor = Executor::new();
	executor.spawn(Task::new(keyboard::print_keypresses()));
	executor.run();
}
