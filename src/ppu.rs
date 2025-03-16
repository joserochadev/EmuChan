#![warn(dead_code)]

#[derive(Debug)]
pub struct PPU {
	pub vram: [u8; 0x2000],
}

impl PPU {
	pub fn new() -> Self {
		Self { vram: [0; 0x2000] }
	}

	pub fn read(&self, addr: u16) -> u8 {
		if addr >= 0x8000 && addr <= 0x9FFF {
			let addr = addr - 0x8000;
			return self.vram[addr as usize];
		}

		return 0x00;
	}

	pub fn write(&mut self, addr: u16, data: u8) {
		if addr >= 0x8000 && addr <= 0x9FFF {
			let addr = addr - 0x8000;
			self.vram[addr as usize] = data;
			return;
		}
	}

	pub fn _dump_vram(&self, start: u16, end: u16) {
		if start < 0x8000 || end > 0x9FFF || start > end {
			println!("Invalid Interval! The VRAM goes from 0x8000 to 0x9FFF.");
			return;
		}

		let start_idx = (start - 0x8000) as usize;
		let end_idx = (end - 0x8000) as usize;

		println!("VRAM Dump (0x{:04X} - 0x{:04X}):", start, end);
		for (i, byte) in self.vram[start_idx..=end_idx].iter().enumerate() {
			if i % 16 == 0 {
				print!("\n0x{:04X}: ", start + i as u16);
			}
			print!("{:02X} ", byte);
		}
		println!();
	}
}
