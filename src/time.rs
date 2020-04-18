pub struct Calendar {
	pub year: u32,
	pub month: u32,
	pub day: u32,
	pub hour: u32,
	pub minute: u32,
	pub second: u32,
}
impl core::fmt::Display for Calendar {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "{}-{}-{} {:02}:{:02}:{:02}", self.year, self.month, self.day, self.hour, self.minute, self.second)
	}
}

fn get_update_in_progress_flag() -> bool {
	use x86_64::instructions::port::{Port, PortReadOnly};
	let mut rtc_address_port = Port::<u8>::new(0x70);
	let mut rtc_data_port = PortReadOnly::<u8>::new(0x71);
	unsafe {
		let nmi_bit: u8 = rtc_address_port.read() & 0x80 as u8;
		rtc_address_port.write(nmi_bit | (0x0A as u8));
		rtc_data_port.read() & 0x80 == 0x80
	}
}

fn get_rtc_register(reg: u8) -> u8 {
	use x86_64::instructions::port::{Port, PortReadOnly};
	let mut rtc_address_port = Port::<u8>::new(0x70);
	let mut rtc_data_port = PortReadOnly::new(0x71);
	unsafe {
		let nmi_bit: u8 = rtc_address_port.read() & 0x80;
		rtc_address_port.write(nmi_bit | reg);
		rtc_data_port.read()
	}
}

pub fn get_current_time(century_register: u8) -> Calendar {
	while get_update_in_progress_flag(){}
	let mut second = get_rtc_register(0x00) as u32;
	let mut minute = get_rtc_register(0x02) as u32;
	let mut hour = get_rtc_register(0x04) as u32;
	let mut day = get_rtc_register(0x07) as u32;
	let mut month = get_rtc_register(0x08) as u32;
	let mut year = get_rtc_register(0x09) as u32;
	let mut century = if century_register != 0 {get_rtc_register(century_register)} else {0} as u32;

	loop {
		let last_second = second;
		let last_minute = minute;
		let last_hour = hour;
		let last_day = day;
		let last_month = month;
		let last_year = year;
		let last_century = century;
		while get_update_in_progress_flag() {}
		second = get_rtc_register(0x00) as u32;
		minute = get_rtc_register(0x02) as u32;
		hour = get_rtc_register(0x04) as u32;
		day = get_rtc_register(0x07) as u32;
		month = get_rtc_register(0x08) as u32;
		year = get_rtc_register(0x09) as u32;
		century = if century_register != 0 {get_rtc_register(century_register)} else {0} as u32;
		if last_second == second && last_minute == minute && last_hour == hour && last_day == day && last_month == month && last_year == year && last_century == century {break};
	}

	//todo: get the century register

	let status_register = get_rtc_register(0x0B);
	if status_register & 0b100 == 0b000 {
		//convert BCD to binary values
		second = (second & 0x0F) + ((second / 16) * 10);
		minute = (minute & 0x0F) + ((minute / 16) * 10);
		hour = ((hour & 0x0F) + ((hour / 16) * 10)) | (hour & 0x80);
		day = (day & 0x0F) + ((day / 16) * 10);
		month = (month & 0x0F) + ((month / 16) * 10);
		year = (year & 0x0F) + ((year / 16) * 10);
		if century_register != 0 {
			century = (century & 0x0F) + ((century / 16) * 10);
		}
	}
	if (status_register & 0b10) == 0 && (hour & 0x80) == 0x80 {
		//convert 12 hour to 24 hour clock
		hour = 99;
	}

	if century_register != 0 {
		year += century * 100;
	}
	//get the full year, the RTC stores only the last two digits (thanks, Y2K bug)
	Calendar {
		second,
		minute,
		hour,
		day,
		month,
		year
	}
}