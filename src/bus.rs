#![allow(dead_code)]
use log::debug;

use std::sync::{Arc, Mutex};

use crate::cartridge::Cartridge;

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
	pub cartridge: Option<Arc<Mutex<Cartridge>>>,
	pub disable_boot: bool,
}

impl BUS {
	pub fn new() -> BUS {
		BUS {
			memory: [0; 0x10000],
			cartridge: None,
			disable_boot: false,
		}
	}

	pub fn read(&self, addr: u16) -> u8 {
		if addr < 0x100 && self.disable_boot == false {
			// Boot
			return self.memory[addr as usize];
		}

		if addr < 0x8000 {
			// Cartridge ROM

			if let Some(cart) = &self.cartridge {
				let cart = cart.lock().unwrap();
				return cart.read(addr);
			}
		}

		if addr < 0xA000 {
			// VRAM
			debug!("Accessing VRAM at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		if addr < 0xC000 {
			// External RAM
			debug!("Accessing External RAM at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		if addr < 0xE000 {
			// WRAM
			debug!("Accessing WRAM at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		if addr < 0xFE00 {
			// Echo RAM
			debug!("Accessing Echo RAM at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		if addr < 0xFEA0 {
			// OAM
			debug!("Accessing OAM at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		if addr < 0xFF00 {
			// Not Usable
			debug!("Accessing Not Usable memory at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		if addr < 0xFF80 {
			// I/O Registers
			debug!("Accessing I/O Registers at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		if addr < 0xFFFF {
			// HRAM
			debug!("Accessing HRAM at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		if addr == 0xFFFF {
			// Interrupt Enable register (IE)
			debug!("Accessing Interrupt Enable register at 0x{:04X}", addr);
			return self.memory[addr as usize];
		}

		self.memory[addr as usize]
	}

	pub fn write(&mut self, addr: u16, data: u8) {
		if addr < 0x100 {
			// BOOT
			self.memory[addr as usize] = data;
			return;
		}

		if addr < 0x8000 {
			// Cartridge ROM

			if let Some(cart) = &self.cartridge {
				let mut cart = cart.lock().unwrap();
				cart.write(addr, data);
				return;
			}
		}

		if addr < 0xA000 {
			// VRAM
			debug!("Writing to VRAM at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}

		if addr < 0xC000 {
			// External RAM
			debug!("Writing to External RAM at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}

		if addr < 0xE000 {
			// WRAM
			debug!("Writing to WRAM at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}

		if addr < 0xFE00 {
			// Echo RAM
			debug!("Writing to Echo RAM at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}

		if addr < 0xFEA0 {
			// OAM
			debug!("Writing to OAM at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}

		if addr < 0xFF00 {
			// Not Usable
			debug!("Writing to Not Usable memory at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}

		if addr < 0xFF80 {
			// Disable Boot
			if addr == 0xFF50 && self.disable_boot == false {
				self.memory[addr as usize] = data;
				self.disable_boot = true;
				debug!("Boot Rom Disabled.");
				return;
			}
			// I/O Registers
			debug!("Writing to I/O Registers at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}

		if addr < 0xFFFF {
			// HRAM
			debug!("Writing to HRAM at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}

		if addr == 0xFFFF {
			// Interrupt Enable register (IE)
			debug!("Writing to Interrupt Enable register at 0x{:04X}", addr);
			self.memory[addr as usize] = data;
			return;
		}
	}

	pub fn cartridge_connect(&mut self, cart: Arc<Mutex<Cartridge>>) {
		self.cartridge = Some(cart);
	}
}
