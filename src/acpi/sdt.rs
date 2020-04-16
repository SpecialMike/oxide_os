use crate::acpi::ACPISDTHeader;
#[derive(Debug)]
#[repr(C)]
pub struct FADT {
    header: ACPISDTHeader,
    firmware_ctrl: u32,
    dsdt: u32,
    reserved: u8,

    preferred_power_management_profile: u8,
    sci_interrupt: u16,
    smi_command_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_control: u8,
    pm1a_event_block: u32,
    pm1b_event_block: u32,
    pm1a_control_block: u32,
    pm1b_control_block: u32,
    pm2_control_block: u32,
    pm_timer_block: u32,
    gpe0_block: u32,
    gpe1_block: u32,
    pm1_event_length: u8,
    pm1_control_length: u8,
    pm2_control_length: u8,
    pm_timer_length: u8,
    gpe0_length: u8,
    gpe1_length: u8,
    gpe1_base: u8,
    c_state_control: u8,
    worst_c2_latency: u16,
    worst_c3_latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    pub century: u8,

    // reserved in ACPI 1.0; used since ACPI 2.0+
    boot_architecture_flags: u16,

    reserved2: u8,
    flags: u32,

    // 12 byte structure; see below for details
    reset_reg: GenericAddressStructure,

    reset_value: u8,
    reserved3: [u8; 3],

    // 64bit pointers - Available on ACPI 2.0+
    x_firmware_control: u64,
    x_dsdt: u64,

    x_pm1a_event_block: GenericAddressStructure,
    x_pm1b_event_block: GenericAddressStructure,
    x_pm1a_control_block: GenericAddressStructure,
    x_pm1b_control_block: GenericAddressStructure,
    x_pm2_control_block: GenericAddressStructure,
    x_pm_timer_block: GenericAddressStructure,
    x_gpe0_block: GenericAddressStructure,
    x_gpe1_block: GenericAddressStructure,
}

#[derive(Debug)]
#[repr(C)]
pub struct GenericAddressStructure {
    address_space: u8,	// 0 - system memory, 1 - system I/O
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    address: u64,
}

#[repr(C)]
pub struct MADT {
	header: ACPISDTHeader,
	apic_address: u32,
	flags: u32,
	//todo: figure out a good way to access the interrupt devices through this
}

#[repr(C)]
pub struct HPET {
	header: ACPISDTHeader,
	hardware_rev_id: u8,
	packed_field: u8,	//contains the bits for the next four fields:
    // comparator_count: 5 bits
    // counter_size:1 bit
    // reserved: 1 bit
    // legacy_replacement: 1 bit
    pci_vendor_id: u16,
    address: GenericAddressStructure,
    hpet_number: u8,
    minimum_tick: u16,
    page_protection: u8,
}
impl HPET {
	pub fn get_comparator_count(&self) -> u8 {
		self.packed_field & 0b0001_1111
	}
	pub fn get_counter_size(&self) -> u8 {
		self.packed_field & 0b0010_0000 >> 5
	}
	pub fn get_legacy_replacement(&self) -> u8 {
		self.packed_field & 0b1000_0000 >> 7
	}
}