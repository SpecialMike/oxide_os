#[derive(Debug)]
pub enum PCIBar {
    None,
    Memory((u32, bool)),
    Port(u16),
}

impl From<u32> for PCIBar {
    fn from(base_address_register: u32) -> Self {
        if base_address_register & 0xFFFF_FFFC == 0 {
            PCIBar::None
        } else if base_address_register & 1 == 0 {
			let prefetchable = base_address_register & 0b1000 == 0b1000;
            PCIBar::Memory((base_address_register & 0xFFFF_FFF0, prefetchable))
        } else {
            PCIBar::Port((base_address_register & 0xFFFC) as u16)
        }
    }
}
