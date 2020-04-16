#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(oxide_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

use oxide_os::{serial_print, serial_println};
use alloc::boxed::Box;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
	use oxide_os::allocator;
	use oxide_os::memory;
	use x86_64::VirtAddr;

	oxide_os::init(boot_info);
	let physical_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
	let mut mapper = unsafe { memory::init(physical_mem_offset)};
	let mut frame_allocator =
        unsafe { memory::init_allocator(&boot_info.memory_map) };
	allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
	test_main();

	loop{}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    oxide_os::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
	serial_print!("simple allocation... ");
	let heap_value = Box::new(41);
	assert_eq!(*heap_value, 41);
	serial_println!("[ok]");
}

use alloc::vec::Vec;
#[test_case]
fn large_vec() {
	serial_print!("large_vec... ");
	let n = 1000;
	let mut vec = Vec::new();
	for i in 0..n {
		vec.push(i);
	}
	assert_eq!(vec.iter().sum::<u64>(), (n-1) * n / 2);
	serial_println!("[ok]");
}

use oxide_os::allocator::HEAP_SIZE;

#[test_case]
fn many_boxes() {
	serial_print!("many_boxes... ");
	for i in 0..HEAP_SIZE {
		let x = Box::new(i);
		assert_eq!(*x, i);
	}
	serial_println!("[ok]");
}