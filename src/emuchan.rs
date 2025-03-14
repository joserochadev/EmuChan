#![allow(dead_code)]
use crate::bus::BUS;
use crate::cartridge::Cartridge;
use crate::cpu::{Register, CPU};
use crate::ui::UI;
use crate::utils::boot::BOOT_DMG;

use std::sync::{Arc, Mutex};
use std::thread;

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
	pub bus: Arc<Mutex<BUS>>,
	pub cpu: Arc<Mutex<CPU>>,
	pub cartridge: Arc<Mutex<Cartridge>>,
	pub ui: UI,
	pub emulation_state: Arc<Mutex<EmulationState>>,
}

impl EmuChan {
	pub fn new() -> Self {
		let bus = Arc::new(Mutex::new(BUS::new()));
		let cpu = Arc::new(Mutex::new(CPU::new(Arc::clone(&bus))));
		let cartridge = Arc::new(Mutex::new(Cartridge::new()));
		let emulation_state = Arc::new(Mutex::new(EmulationState::RUNNING));

		let mut ui = UI::new();

		bus
			.lock()
			.unwrap()
			.cartridge_connect(Arc::clone(&cartridge));

		{
			let mut cart = cartridge.lock().unwrap();

			// Load ROM
			cart.load_rom("./roms/games/pokemon.gb".to_string());

			// Remove null bytes
			let game_title = cart.game_title.trim_end_matches('\0');

			let new_window_title = format!("EmuChan - {}", game_title);
			println!("{}", new_window_title);

			// Set window title with game name
			ui.main_window.set_title(&new_window_title).unwrap();
		}

		bus.lock().unwrap().memory[0..=255].copy_from_slice(&BOOT_DMG);

		// Simulating v-blank period
		bus.lock().unwrap().write(0xff44, 0x90);

		Self {
			bus,
			cpu,
			cartridge,
			ui,
			emulation_state,
		}
	}

	pub fn run(&mut self) {
		let cpu_clone = Arc::clone(&self.cpu);
		let emulation_state_clone = Arc::clone(&self.emulation_state);

		thread::spawn(move || {
			let mut cpu = cpu_clone.lock().unwrap();

			while *emulation_state_clone.lock().unwrap() == EmulationState::RUNNING {
				if let Err(e) = cpu.step() {
					panic!("{}\n{}", e, cpu);
				}
			}
		});

		self.ui.run();
	}
}
