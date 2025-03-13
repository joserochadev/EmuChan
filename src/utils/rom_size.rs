pub fn get_rom_size(code: u8) -> &'static str {
	match code {
		0x00 => "32 KiB (2 ROM banks)",
		0x01 => "64 KiB (4 ROM banks)",
		0x02 => "128 KiB (8 ROM banks)",
		0x03 => "256 KiB (16 ROM banks)",
		0x04 => "512 KiB (32 ROM banks)",
		0x05 => "1 MiB (64 ROM banks)",
		0x06 => "2 MiB (128 ROM banks)",
		0x07 => "4 MiB (256 ROM banks)",
		0x08 => "8 MiB (512 ROM banks)",
		0x52 => "1.1 MiB (72 ROM banks)",
		0x53 => "1.2 MiB (80 ROM banks)",
		0x54 => "1.5 MiB (96 ROM banks)",
		_ => "Unknown",
	}
}
