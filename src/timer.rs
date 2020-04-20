use x86_64::instructions::port::Port;
use crate::println;
///Set the hardware timer to interrupt every `freq` Hz
pub fn set_interrupt_freq(freq: u32) {
	let mut command_port = Port::new(0x43);
	let mut port = Port::new(0x40);
	let divisor = 1_193_182 / freq;
	println!("Setting timer divisor to {} ({}Hz)", divisor as u16, (1_193_182 / divisor as u32) as u16);
	unsafe {
		command_port.write(0x36u8);
		port.write((divisor & 0xFF) as u8);
		port.write((divisor >> 8) as u8);
	}
}