#![allow(dead_code)]

/*
+-------+-------+---------------------------------+-----------------------------------------------------+
| Start | End   | Description                 		| Notes                                     					|
+-------+-------+---------------------------------+-----------------------------------------------------+
| 0000  | 3FFF  | 16 KiB ROM bank 00          		| From cartridge, usually a fixed bank     						|
| 4000  | 7FFF  | 16 KiB ROM Bank 01–NN       		| From cartridge, switchable bank via mapper (if any) |
| 8000  | 9FFF  | 8 KiB Video RAM (VRAM)      		| In CGB mode, switchable bank 0/1         						|
| A000  | BFFF  | 8 KiB External RAM          		| From cartridge, switchable bank if any   						|
| C000  | CFFF  | 4 KiB Work RAM (WRAM)       		|                                         						|
| D000  | DFFF  | 4 KiB Work RAM (WRAM)       		| In CGB mode, switchable bank 1–7        						|
| E000  | FDFF  | Echo RAM (mirror of C000–DDFF) 	| Nintendo says use of this area is prohibited. 			|
| FE00  | FE9F  | Object attribute memory (OAM) 	|                                         						|
| FEA0  | FEFF  | Not Usable                  		| Nintendo says use of this area is prohibited. 			|
| FF00  | FF7F  | I/O Registers               		|                                         						|
| FF80  | FFFE  | High RAM (HRAM)             		|                                         						|
| FFFF  | FFFF  | Interrupt Enable register (IE) 	|                                         						|
+-------+-------+---------------------------------+-----------------------------------------------------+

*/

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
		if addr < 0x8000 {
			// Cartridge ROM
			return self.memory[addr as usize];
		}
		
		if addr < 0xA000 {
			// VRAM
			return self.memory[addr as usize];
		}

		if addr < 0xC000 {
			// External RAM
			return self.memory[addr as usize];
		}
		
		if addr < 0xE000 {
			// WRAM
			return self.memory[addr as usize];
		}
		
		if addr < 0xFE00 {
			// Echo RAM
			return self.memory[addr as usize];
		}
		
		if addr < 0xFEA0 {
			// OAM
			return self.memory[addr as usize];
		}
		
		if addr < 0xFF00 {
			// Not Usable
			return self.memory[addr as usize];
		}
		
		if addr < 0xFF80 {
			// I/O Registers
			return self.memory[addr as usize];
		}

		if addr < 0xFFFF {
			// HRAM
			return self.memory[addr as usize];
		}
		
		if addr == 0xFFFF {
			// Interrupt Enable register (IE)
			return self.memory[addr as usize];
		}
		


		self.memory[addr as usize]
	}

	pub fn write(&mut self, addr: u16, data: u8) {

		if addr < 0x8000 {
			// Cartridge ROM
			self.memory[addr as usize] = data;
		}
		
		if addr < 0xA000 {
			// VRAM
			self.memory[addr as usize] = data;
		}

		if addr < 0xC000 {
			// External RAM
			self.memory[addr as usize] = data;
		}
		
		if addr < 0xE000 {
			// WRAM
			self.memory[addr as usize] = data;
		}
		
		if addr < 0xFE00 {
			// Echo RAM
			self.memory[addr as usize] = data;
		}
		
		if addr < 0xFEA0 {
			// OAM
			self.memory[addr as usize] = data;
		}
		
		if addr < 0xFF00 {
			// Not Usable
			self.memory[addr as usize] = data;
		}
		
		if addr < 0xFF80 {
			// I/O Registers
			self.memory[addr as usize] = data;
		}

		if addr < 0xFFFF {
			// HRAM
			self.memory[addr as usize] = data;
		}
		
		if addr == 0xFFFF {
			// Interrupt Enable register (IE)
			self.memory[addr as usize] = data;
		}
		

	}
}
