#![allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BUS {
	pub memory: [u8; 0x10000], // address 0 to 0xffff
}

impl BUS {
	pub fn new() -> BUS {
		BUS {
			memory: [0; 0x10000],
		}
	}

	pub fn read(&self, addr: u16) -> u8 {
		self.memory[addr as usize]
	}

	pub fn write(&mut self, addr: u16, data: u8) {
		self.memory[addr as usize] = data;
	}
}
