use core::mem;
use core::ops::{Deref, DerefMut};
use core::slice;
use x86_64::instructions::port::Port;

mod pci_class;
use pci_class::PCIClass;
mod pci_bar;
use pci_bar::PCIBar;

#[derive(Default, Debug)]
#[repr(C)]
pub struct PCIHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision: u8,
    pub interface: u8,
    pub subclass: u8,
    pub class: PCIClass,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
    pub base_address_registers: [u32; 6],
    pub cardbus_cis_ptr: u32,
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub expansion_rom_bar: u32,
    pub capabilities: u8,
    pub reserved: [u8; 7],
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

impl Deref for PCIHeader {
    type Target = [u32];
    fn deref(&self) -> &[u32] {
        unsafe {
            slice::from_raw_parts(
                self as *const PCIHeader as *const u32,
                mem::size_of::<PCIHeader>() / 4,
            ) as &[u32]
        }
    }
}

impl DerefMut for PCIHeader {
    fn deref_mut(&mut self) -> &mut [u32] {
        unsafe {
            slice::from_raw_parts_mut(
                self as *mut PCIHeader as *mut u32,
                mem::size_of::<PCIHeader>() / 4,
            ) as &mut [u32]
        }
    }
}

impl core::fmt::Display for PCIHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.class)?;
        match self.class {
            PCIClass::MassStorage => match self.subclass {
                0x01 => write!(f, " IDE"),
                0x06 => write!(f, " SATA"),
                _ => Ok(()),
            },
            PCIClass::SerialBus => match self.subclass {
                0x03 => match self.interface {
                    0x00 => write!(f, " UHCI"),
                    0x10 => write!(f, " OHCI"),
                    0x20 => write!(f, " EHCI"),
                    0x30 => write!(f, " XHCI"),
                    _ => Ok(()),
                },
                _ => Ok(()),
            },
            _ => Ok(()),
        }?;
        writeln!(f)?;
        writeln!(
            f,
            "latency: {}, IRQ:{}",
            self.max_latency, self.interrupt_line
        )?;
        for (i, base_address_register) in self.base_address_registers.iter().enumerate() {
            match PCIBar::from(*base_address_register) {
                PCIBar::None => Ok(()),
                PCIBar::Memory(address) => writeln!(f, "BAR[{}]:{:>08X}(memory)", i, address),
                PCIBar::Port(address) => writeln!(f, "BAR[{}]:{:>04X}(port)", i, address),
            }?;
        }
        Ok(())
    }
}

pub struct PCIFunc<'pci> {
    pub device: &'pci PCIDevice<'pci>,
    pub num: u8,
}
impl<'pci> PCIFunc<'pci> {
    pub fn header(&self) -> Option<PCIHeader> {
        // if the device does not exist, the host bridge will return all ones on read
        unsafe {
            if self.read(0) != 0xFFFF_FFFF {
                let mut header = PCIHeader::default();
                let dwords = header.deref_mut();
                dwords.iter_mut().fold(0usize, |offset, dword| {
                    *dword = self.read(offset as u8);
                    offset + 4
                });
                Some(header)
            } else {
                None
            }
        }
    }
    /// Reads from the PCI controller for this Function and its device/bus
    /// # Safety
    /// - This method reads from CPU I/O ports
    pub unsafe fn read(&self, offset: u8) -> u32 {
        self.device.read(self.num, offset)
    }
}

pub struct PCIDevice<'pci> {
    pub bus: &'pci PCIBus<'pci>,
    pub num: u8,
}
impl<'pci> PCIDevice<'pci> {
    pub fn functions(&'pci self) -> PCIDeviceIter<'pci> {
        PCIDeviceIter::new(self)
    }
    /// Reads from the PCI controller for this device and its bus
    /// # Safety
    /// - This method reads from CPU I/O ports
    pub unsafe fn read(&self, func: u8, offset: u8) -> u32 {
        self.bus.read(self.num, func, offset)
    }
}
pub struct PCIDeviceIter<'pci> {
    device: &'pci PCIDevice<'pci>,
    count: u8,
}
impl<'pci> PCIDeviceIter<'pci> {
    pub fn new(device: &'pci PCIDevice<'pci>) -> Self {
        Self {
            device,
            count: 0,
        }
    }
}
impl<'pci> Iterator for PCIDeviceIter<'pci> {
    type Item = PCIFunc<'pci>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 8 {
            let func = PCIFunc {
                device: self.device,
                num: self.count,
            };
            self.count += 1;
            Some(func)
        } else {
            None
        }
    }
}

pub struct PCIBus<'pci> {
    pub pci: &'pci PCI,
    pub num: u8,
}
impl<'pci> PCIBus<'pci> {
    pub fn devices(&'pci self) -> PCIBusIter<'pci> {
        PCIBusIter::new(self)
    }
    /// Reads from the PCI controller for this bus
    /// # Safety
    /// - This method reads from CPU I/O ports
    pub unsafe fn read(&self, device: u8, func: u8, offset: u8) -> u32 {
        self.pci.read(self.num, device, func, offset)
    }
}

/// Iterates over devices on a given PCIBus
pub struct PCIBusIter<'pci> {
    bus: &'pci PCIBus<'pci>,
    count: u8,
}
impl<'pci> PCIBusIter<'pci> {
    pub fn new(bus: &'pci PCIBus<'pci>) -> Self {
        Self { bus, count: 0 }
    }
}
impl<'pci> Iterator for PCIBusIter<'pci> {
    type Item = PCIDevice<'pci>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 32 {
            let device = PCIDevice {
                bus: self.bus,
                num: self.count,
            };
            self.count += 1;
            Some(device)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct PCI;

impl PCI {
    pub fn new() -> Self {
        PCI
    }
    pub fn busses(&self) -> PCIIter {
        PCIIter::new(self)
    }

    /// Read from the PCI controller for the given arguments
    /// # Safety
    /// - This method reads from CPU I/O ports
    pub unsafe fn read(&self, bus: u8, device: u8, func: u8, offset: u8) -> u32 {
        let address = 0x8000_0000
            | ((bus as u32) << 16)
            | ((device as u32) << 11)
            | ((func as u32) << 8)
            | ((offset as u32) & 0xFC);
        let mut port_config_address = Port::new(0xCF8);
        let mut port_config_data = Port::new(0xCFC);
        port_config_address.write(address);
        port_config_data.read()
    }
}

/// Iterates over PCIBusses for a given PCI controller
pub struct PCIIter<'pci> {
    pci: &'pci PCI,
    count: u16,
}
impl<'pci> PCIIter<'pci> {
    pub fn new(pci: &'pci PCI) -> Self {
        PCIIter { pci, count: 0 }
    }
}

impl<'pci> Iterator for PCIIter<'pci> {
    type Item = PCIBus<'pci>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 256 {
            let bus = PCIBus {
                pci: self.pci,
                num: self.count as u8,
            };
            self.count += 1;
            Some(bus)
        } else {
            None
        }
    }
}
