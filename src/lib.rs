#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(alloc_layout_extra)]
#![feature(const_in_array_repeat_expressions)]
#![feature(wake_trait)]

use bootloader::BootInfo;
use x86_64::VirtAddr;

#[cfg(test)]
use bootloader::entry_point;
use core::panic::PanicInfo;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;
pub mod time;
pub mod acpi;
pub mod timer;
use acpi::ACPI;

pub mod task;

extern crate alloc;

pub fn test_runner(tests: &[&dyn Fn()]) {
	let start = unsafe {core::arch::x86_64::_rdtsc()};
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
	}
	let end = unsafe {core::arch::x86_64::_rdtsc()};
	serial_println!("Cycles for tests: {}", end-start);
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
#[no_mangle]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
	init(boot_info);
	test_main();
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn init(boot_info: &'static BootInfo) {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
	x86_64::instructions::interrupts::enable();

	let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator =
        unsafe { memory::init_allocator(&boot_info.memory_map) };
	allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

	acpi::get_rsdp(physical_memory_offset);
	let mut century_register = 0;
	if let Some(fadt) = *(ACPI.fadt.read()) {
		century_register = fadt.century;
	}
	let current_time = time::get_current_time(century_register);
	println!("System startup at {}", current_time);

	println!("{}", ACPI);

	timer::set_interrupt_freq(100);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
