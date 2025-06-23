#![allow(dead_code)]
use crate::common::boot::BOOT_DMG;
use crate::core::bus::BUS;
use crate::core::cartridge::Cartridge;
use crate::core::cpu::CPU;
use crate::core::ppu::PPU;
use crate::ui::UI;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

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
	// pub registers: Register,
	pub flags: EmuStateFlags,
	pub emulation_state: EmulationState,
}

pub struct EmuChan {
	pub bus: Arc<Mutex<BUS>>,
	pub cpu: Arc<Mutex<CPU>>,
	pub ppu: Arc<Mutex<PPU>>,
	pub cartridge: Arc<Mutex<Cartridge>>,
	// pub ui: UI,
	pub emulation_state: Arc<Mutex<EmulationState>>,

	frame_count: u32,
	last_fps_check: Instant,
	pub emu_fps: f64, //

	pub emu_speed_percent: f64,
}

impl EmuChan {
	pub fn new(rom_path: Option<String>) -> Self {
		let bus = Arc::new(Mutex::new(BUS::new()));
		let cpu = Arc::new(Mutex::new(CPU::new(Arc::clone(&bus))));
		let ppu = Arc::new(Mutex::new(PPU::new()));
		let cartridge = Arc::new(Mutex::new(Cartridge::new()));
		let emulation_state = Arc::new(Mutex::new(EmulationState::PAUSED));

		// let mut ui = UI::new(Arc::clone(&ppu));

		{
			let mut bus = bus.lock().unwrap();

			// Conecting cartridge to bus
			bus.cartridge_connect(Arc::clone(&cartridge));
			// Conecting ppu to bus
			bus.ppu_connect(Arc::clone(&ppu));
			// Load boot in memory
			bus.memory[0..=255].copy_from_slice(&BOOT_DMG);
			// Simulating v-blank period
			// bus.write(0xFF44, 0x90);
		}

		// {
		// 	let mut cart = cartridge.lock().unwrap();

		// 	// Load ROM
		// 	if let Some(rom) = rom_path {
		// 		cart.load_rom(rom);
		// 	}

		// 	// Remove null bytes
		// 	let game_title = cart.game_title.trim_end_matches('\0');
		// 	let new_window_title = format!("EmuChan - {}", game_title);

		// 	// Set window title with game name
		// 	// ui.main_window.set_title(&new_window_title).unwrap();
		// }

		Self {
			bus,
			cpu,
			ppu,
			cartridge,
			// ui,
			emulation_state,
			frame_count: 0,
			last_fps_check: Instant::now(),
			emu_fps: 0.0,
			emu_speed_percent: 0.0,
		}
	}

	pub fn load_rom(&mut self, path: String) {
		let mut cartridge = self.cartridge.lock().unwrap();

		cartridge.load_rom(path);
	}

	pub fn get_game_title(&self) -> String {
		self.cartridge.lock().unwrap().game_title.clone()
	}

	pub fn get_video_buffer(&self) -> Vec<u8> {
		let ppu = self.ppu.lock().unwrap();
		return ppu.video_buffer.to_vec();
	}

	// pub fn run(&mut self) {
	// 	let cpu_clone = Arc::clone(&self.cpu);
	// 	let ppu_clone = Arc::clone(&self.ppu);
	// 	let emulation_state_clone = Arc::clone(&self.emulation_state);

	// 	thread::spawn(move || {
	// 		let mut emulation_state = emulation_state_clone.lock().unwrap();
	// 		while *emulation_state == EmulationState::RUNNING {
	// 			// CPU step
	// 			let mut cpu = cpu_clone.lock().unwrap();
	// 			if let Err(e) = cpu.step() {
	// 				*emulation_state = EmulationState::PAUSED;
	// 				println!("EmuChan is PAUSED.");
	// 				println!("{}\n{}", e, cpu);
	// 			}

	// 			// PPU step
	// 			let mut ppu = ppu_clone.lock().unwrap();
	// 			for _ in 0..cpu.cycles {
	// 				for _ in 0..4 {
	// 					ppu.step();
	// 				}
	// 			}
	// 		}
	// 	});

	// 	print!("{}", self.cpu.lock().unwrap());
	// }

	pub fn run_one_frame(&mut self) {
		if *self.emulation_state.lock().unwrap() != EmulationState::RUNNING {
			return;
		}
		const CYCLES_PER_FRAME: u32 = 70224; // ~70k T-ciclos por quadro
		let mut cycles_this_frame = 0;

		while cycles_this_frame < CYCLES_PER_FRAME {
			// Supondo que cpu.step() retorna os T-ciclos da instrução
			let cycles_executed = {
				let mut cpu = self.cpu.lock().unwrap();
				match cpu.step() {
					
					Err(e) => {
						*self.emulation_state.lock().unwrap() = EmulationState::PAUSED;
						println!("EmuChan is PAUSED.");
						println!("{}\n{}", e, cpu);
						break;

					},
					Ok(cycles) => cycles
				}
			};
			cycles_this_frame += cycles_executed; // os ciclos da cpu estao eu m-cycles entao  eu converto em t-cycles

			// A PPU avança o mesmo número de "pontos" (dots) que os T-ciclos da CPU
			// Supondo que ppu.step() avança um "ponto"
			let mut ppu = self.ppu.lock().unwrap();
			for _ in 0..cycles_executed {
				ppu.step();
			}
		}

		const TARGET_FPS: f64 = 59.7275;
		// 1. Incrementa o contador de quadros gerados.
		self.frame_count += 1;

		// 2. Verifica quanto tempo passou desde a última checagem de FPS.
		let elapsed = self.last_fps_check.elapsed();

		// 3. Se passou um segundo ou mais...
		if elapsed.as_secs_f64() >= 1.0 {
			// ... calcula o FPS com base em quantos quadros foram contados.
			self.emu_fps = self.frame_count as f64 / elapsed.as_secs_f64();
			self.emu_speed_percent = (self.emu_fps / TARGET_FPS) * 100.0;

			// ... reinicia o contador e o temporizador para o próximo segundo.
			self.frame_count = 0;
			self.last_fps_check = Instant::now();
		}
	}
}
