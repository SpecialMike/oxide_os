use x86_64::{VirtAddr, PhysAddr};
use spin::{RwLock};

pub mod sdt;
use sdt::{FADT,MADT,HPET};

pub struct Acpi {
	pub rsdt: RwLock<Option<&'static RSDT>>,
	pub fadt: RwLock<Option<&'static FADT>>,
	pub madt: RwLock<Option<&'static MADT>>,
	pub hpet: RwLock<Option<&'static HPET>>,
}
impl core::fmt::Display for Acpi {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		writeln!(f, "ACPI table:")?;
		if let Some(ptr) = *(self.rsdt.read()) {
			writeln!(f, "rsdt: {:p}", ptr)?;
		}
		if let Some(ptr) = *(self.fadt.read()) {
			writeln!(f, "fadt: {:p}", ptr)?;
		}
		if let Some(ptr) = *(self.madt.read()) {
			writeln!(f, "madt: {:p}", ptr)?;
		}
		if let Some(ptr) = *(self.hpet.read()) {
			writeln!(f, "hpet: {:p}", ptr)?;
		}
		Ok(())
	}
}
pub static ACPI: Acpi = Acpi{
	rsdt: RwLock::new(None),
	fadt: RwLock::new(None),
	madt: RwLock::new(None),
	hpet: RwLock::new(None),
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RSDPDescriptor {
	// 8-byte string, MUST equal "RSD PTR "
	pub signature: [u8;8],
	// A value to be added to all other bytes to calculate the checksum of the table
	pub checksum: u8,
	// A string that identifies the OEM
	pub oem_id: [u8;6],
	// If 0, ACPI version 1.0 is used, the value 2 is used for ACPI version 2 to 6.1
	pub revision: u8,
	// The physical address to the RSDT table
	pub rsdt_address: PhysAddr,
}

#[repr(C)]
#[derive(Debug)]
pub struct RSDT {
	header: ACPISDTHeader,
	/// An array of u32 in memory, which are pointers to SDT
	/// QEMU has three SDT: FACP, APIC and HPET
	pointer_to_other_sdt: u32,
}
impl RSDT {
	pub fn get_num_sdt(&self) -> usize {
		(self.header.length as usize - core::mem::size_of::<ACPISDTHeader>()) / 4
	}
	pub fn get_sdt_addresses(&self) -> &'static [u32] {
		unsafe {core::slice::from_raw_parts(&self.pointer_to_other_sdt as *const u32, self.get_num_sdt())}
	}
}

const SIGNATURE_RSDP: &[u8;8] = b"RSD PTR ";
const SIGNATURE_FADT: &[u8;4] = b"FACP";
const SIGNATURE_MADT: &[u8;4] = b"APIC";
const SIGNATURE_HPET: &[u8;4] = b"HPET";

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ACPISDTHeader {
	pub signature: [u8;4],
	length: u32,
	revision: u8,
	checksum: u8,
	oem_id: [u8;6],
	oem_table_id: [u8;8],
	oem_revision: u32,
	creator_id: u32,
	creator_revision: u32,
}
impl ACPISDTHeader {
	pub fn as_string(&self) -> &str{
		core::str::from_utf8(&self.signature).expect("Failed to parse")
	}
}

pub fn get_rsdp(physical_memory_offset: VirtAddr) {
	//the RSDP is either located in the first 1KB of the EBDA, or in the memory region below 1MiB
	//we have to search for the "RSD PTR " string (including the trailing space), which is the signature of the RSDPDescriptor struct
	//this signature is guaranteed to be on a 16-byte boundary
	for i in (0..0x0010_0000).step_by(16) {
		let descriptor_ptr: *const RSDPDescriptor = (physical_memory_offset + (i as u64)).as_ptr();
		let descriptor = unsafe{*descriptor_ptr};
		let mut success = true;
		for (i, byte) in SIGNATURE_RSDP.iter().enumerate() {
			if *byte != descriptor.signature[i] {
				success = false;
				break;
			}
		}
		if success && rsdp_descriptor_checksum(unsafe{*(descriptor_ptr as *const [u8; 20])}) {
			let rsdt_header_addr = VirtAddr::new(physical_memory_offset.as_u64() + descriptor.rsdt_address.as_u64());
			let rsdt_header_ptr: *const RSDT = rsdt_header_addr.as_ptr();
			let rsdt = unsafe {&*rsdt_header_ptr};
			{
				let mut rsdt_lock = ACPI.rsdt.write();
				*rsdt_lock = Some(rsdt);
			}
			if !unsafe{sdt_checksum(rsdt_header_addr, rsdt.header.length as usize)} {
				crate::println!("Checksum failed!");
			}
			for sdt_address in rsdt.get_sdt_addresses() {
				let sdt_addr = VirtAddr::new(*sdt_address as u64 + physical_memory_offset.as_u64());
				let sdt_header_ptr: *const ACPISDTHeader = sdt_addr.as_ptr();
				let header = unsafe{*sdt_header_ptr};
				match &header.signature {
					SIGNATURE_HPET => {
						let sdt_ptr: *const sdt::HPET = sdt_addr.as_ptr();
						let mut hpet_lock = ACPI.hpet.write();
						*hpet_lock = Some(unsafe {&*sdt_ptr});
					},
					SIGNATURE_FADT => {
						let sdt_ptr: *const sdt::FADT = sdt_addr.as_ptr();
						let mut fadt_lock = ACPI.fadt.write();
						*fadt_lock = Some(unsafe {&*sdt_ptr});
					},
					SIGNATURE_MADT => {
						let sdt_ptr: *const sdt::MADT = sdt_addr.as_ptr();
						let mut madt_lock = ACPI.madt.write();
						*madt_lock = Some(unsafe {&*sdt_ptr});
					},
					_ => crate::println!("Found unknown SDT: {}", header.as_string())
				}
			}
			return;
		}
	}
	panic!("No RSDP signature found!")
}

fn rsdp_descriptor_checksum(table_bytes: [u8; 20]) -> bool {
	let mut sum = 0u32;
	for byte in table_bytes.iter() {
		sum += *byte as u32;
	}
	sum.trailing_zeros() >= 8
}

/// Validates the checksum of a System Description Table
/// # Safety
/// - It must be valid to read all bytes from `table_bytes_base_addr` to `table_bytes_base_addr`+`size`
unsafe fn sdt_checksum(table_bytes_base_addr: VirtAddr, size: usize) -> bool {
	let bytes: &[u8] = core::slice::from_raw_parts(table_bytes_base_addr.as_ptr(), size);
	let mut sum = 0;
	for byte in bytes {
		sum += (*byte) as u32;
	}
	sum.trailing_zeros() >= 8
}