#![allow(dead_code)]
use crate::bus::BUS;
use crate::cpu::{Register, CPU};
use crate::utils::boot::{BOOT_DMG, NINTENDO_LOGO};

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
	pub emulation_state: EmulationState,
}

impl EmuChan {
	pub fn new() -> Self {
		let mut bus = BUS::new();
		let cpu = CPU::new(&mut bus);
		let emulation_state = EmulationState::RUNNING;

		bus.memory[0..=255].copy_from_slice(&BOOT_DMG);
		bus.memory[0x104..=0x133].copy_from_slice(&NINTENDO_LOGO);

		// Simulating v-blank period
		bus.write(0xff44, 0x90);

		Self {
			bus,
			cpu,
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
