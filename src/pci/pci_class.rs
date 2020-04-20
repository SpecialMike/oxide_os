#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PCIClass {
	Unclassified,
	MassStorage,
	Network,
	Display,
	Multimedia,
	Memory,
	BridgeDevice,
	Simple,
	BasePeripheral,
	InputDevice,
	Dock,
	Processor,
	SerialBus,
	Wireless,
	IntelligentController,
	Satellite,
	Encryption,
	SignalProcessing,
	ProcessingAccelerator,
	NonEssential,
	CoProcessor = 0x40,
	Unassigned = 0xFF,
}
impl From<u8> for PCIClass {
	fn from(class: u8) -> Self {
		match class {
			0x00 => PCIClass::Unclassified,
			0x01 => PCIClass::MassStorage,
			0x02 => PCIClass::Network,
			0x03 => PCIClass::Display,
			0x04 => PCIClass::Multimedia,
			0x05 => PCIClass::Memory,
			0x06 => PCIClass::BridgeDevice,
			0x07 => PCIClass::Simple,
			0x08 => PCIClass::BasePeripheral,
			0x09 => PCIClass::InputDevice,
			0x0A => PCIClass::Dock,
			0x0B => PCIClass::Processor,
			0x0C => PCIClass::SerialBus,
			0x0D => PCIClass::Wireless,
			0x0E => PCIClass::IntelligentController,
			0x0F => PCIClass::Satellite,
			0x10 => PCIClass::Encryption,
			0x11 => PCIClass::SignalProcessing,
			0x12 => PCIClass::ProcessingAccelerator,
			0x13 => PCIClass::NonEssential,
			0x40 => PCIClass::CoProcessor,
			_ => panic!("Unknown PCI class: {}", class),
		}
	}
}
impl Default for PCIClass {
	fn default() -> Self {PCIClass::Unclassified}
}