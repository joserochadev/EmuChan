pub fn get_destination(code: u8) -> &'static str {
	match code {
		0x00 => "Japan (and possibly overseas)",
		0x01 => "Overseas only",
		_ => "Unknown",
	}
}
