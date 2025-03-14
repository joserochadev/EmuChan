use log::{debug, info};
use std::fs::File;
use std::io::{BufReader, Read};

use crate::utils::cartridge_destination::get_destination;
use crate::utils::cartridge_type::get_cartridge_type;
use crate::utils::licensee_codes::{get_old_publisher, get_publisher};
use crate::utils::ram_size::get_ram_size;
use crate::utils::rom_size::get_rom_size;

#[derive(Debug)]
pub struct Cartridge {
	pub rom: Vec<u8>,
}

impl Cartridge {
	pub fn new() -> Self {
		Self { rom: Vec::new() }
	}

	pub fn load_rom(&mut self, rom_path: String) {
		debug!("Starting Cartridge...");

		let file = File::open(&rom_path).expect("ROM not found!");
		let file_size = file.metadata().expect("Failed to get file metadata.").len() as usize;
		let mut buffer = BufReader::new(file);

		self.rom.resize(file_size, 0x00);

		debug!("Reading ROM: {}", &rom_path);
		buffer
			.read_exact(&mut self.rom)
			.expect("Failed to read ROM.");
		debug!("ROM Read Successfully!");

		// Game Title
		let title = String::from_utf8_lossy(&self.rom[0x134..=0x143]);
		info!("Game Title: {}", title);

		// Game Version
		let game_version = &self.rom[0x14C];
		info!("Game Version: {}.0", *game_version);

		// Licensee Codes
		let licensee_code = &self.rom[0x14b];

		if *licensee_code == 0x33 {
			let new_lic_code = String::from_utf8_lossy(&self.rom[0x144..=0x145]).to_string();
			info!("New Licensee Code: {}", get_publisher(new_lic_code));
		} else {
			info!("Old Licensee Code: {}", get_old_publisher(*licensee_code as u16))
		}

		// Cartridge Type
		let cartridge_type_code = &self.rom[0x147];
		info!("Cartridge Type: {}", get_cartridge_type(*cartridge_type_code));

		// ROM Size
		let rom_size_code = &self.rom[0x148];
		info!("ROM Size: {}", get_rom_size(*rom_size_code));

		// RAM Size
		let ram_size_code = &self.rom[0x149];
		info!("RAM Size: {}", get_ram_size(*ram_size_code));

		// Cartridge Destination
		let destination_code = &self.rom[0x14A];
		info!("Cartridge Destination: {}", get_destination(*destination_code));

		// Checksum
		let header_checksum = &self.rom[0x14D];
		let mut checksum: u8 = 0;

		for addr in 0x134..=0x14C {
			checksum = checksum.wrapping_sub(self.rom[addr]).wrapping_sub(1);
		}

		info!(
			"Checksum: {:02X} ({})",
			checksum,
			if *header_checksum == checksum {
				"PASSED"
			} else {
				"FAILED"
			}
		);

		// panic!("stop");
	}

	pub fn read(&self, addr: u16) -> u8 {
		return self.rom[addr as usize];
	}

	pub fn write(&mut self, addr: u16, data: u8) {
		self.rom[addr as usize] = data;
	}
}
