#![allow(dead_code)]
use crate::bus::BUS;
use crate::cartridge::Cartridge;
use crate::cpu::{Register, CPU};
use crate::utils::boot::BOOT_DMG;

#[derive(Default, Clone, PartialEq)]
pub enum EmulationState {
	#[default]
	PAUSED,
	RUNNING,
	STEP,
}

#[derive(Default)]
pub struct EmuStateFlags {
	pub z: bool,
	pub h: bool,
	pub n: bool,
	pub c: bool,
}

pub struct EmuState {
	pub registers: Register,
	pub flags: EmuStateFlags,
	pub emulation_state: EmulationState,
}

pub struct EmuChan {
	pub bus: BUS,
	pub cpu: CPU,
	pub cartridge: Cartridge,
	pub emulation_state: EmulationState,
}

impl EmuChan {
	pub fn new() -> Self {
		let mut bus = BUS::new();
		let cpu = CPU::new(&mut bus);
		let mut cartridge = Cartridge::new();
		let emulation_state = EmulationState::RUNNING;

		bus.cartridge_connect(&mut cartridge);

		cartridge.load_rom("./roms/games/tetris.gb".to_string());

		bus.memory[0..=255].copy_from_slice(&BOOT_DMG);

		// Simulating v-blank period
		bus.write(0xff44, 0x90);

		Self {
			bus,
			cpu,
			cartridge,
			emulation_state,
		}
	}

	pub fn run(&mut self) {
		while self.emulation_state == EmulationState::RUNNING {
			if let Err(e) = self.cpu.step() {
				panic!("{}\n{}", e, self.cpu);
			}
		}
	}
}
