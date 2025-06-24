#![allow(dead_code)]
use crate::common::boot::BOOT_DMG;
use crate::core::bus::BUS;
use crate::core::cartridge::Cartridge;
use crate::core::cpu::CPU;
use crate::core::ppu::PPU;

use std::sync::{Arc, Mutex};
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
	pub flags: EmuStateFlags,
	pub emulation_state: EmulationState,
}

pub struct EmuChan {
	pub bus: Arc<Mutex<BUS>>,
	pub cpu: Arc<Mutex<CPU>>,
	pub ppu: Arc<Mutex<PPU>>,
	pub cartridge: Arc<Mutex<Cartridge>>,
	pub emulation_state: Arc<Mutex<EmulationState>>,

	frame_count: u32,
	last_fps_check: Instant,
	pub emu_fps: f64, //

	pub emu_speed_percent: f64,
}

impl EmuChan {
	pub fn new() -> Self {
		let bus = Arc::new(Mutex::new(BUS::new()));
		let cpu = Arc::new(Mutex::new(CPU::new(Arc::clone(&bus))));
		let ppu = Arc::new(Mutex::new(PPU::new()));
		let cartridge = Arc::new(Mutex::new(Cartridge::new()));
		let emulation_state = Arc::new(Mutex::new(EmulationState::PAUSED));


		{
			let mut bus = bus.lock().unwrap();

			// Conecting cartridge to bus
			bus.cartridge_connect(Arc::clone(&cartridge));
			// Conecting ppu to bus
			bus.ppu_connect(Arc::clone(&ppu));
			// Load boot in memory
			bus.memory[0..=255].copy_from_slice(&BOOT_DMG);
		}

		Self {
			bus,
			cpu,
			ppu,
			cartridge,
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

	pub fn run_one_frame(&mut self) {
		if *self.emulation_state.lock().unwrap() != EmulationState::RUNNING {
			return;
		}
		const CYCLES_PER_FRAME: u32 = 70224; // ~70k T-cycles per frame
		let mut cycles_this_frame = 0;

		while cycles_this_frame < CYCLES_PER_FRAME {
			let cycles_executed = {
				let mut cpu = self.cpu.lock().unwrap();
				match cpu.step() {
					Err(e) => {
						*self.emulation_state.lock().unwrap() = EmulationState::PAUSED;
						println!("EmuChan is PAUSED.");
						println!("{}\n{}", e, cpu);
						break;
					}
					Ok(cycles) => cycles,
				}
			};

			cycles_this_frame += cycles_executed; 

			let mut ppu = self.ppu.lock().unwrap();
			for _ in 0..cycles_executed {
				ppu.step();
			}
		}

		// Calc FPS and Speed
		const TARGET_FPS: f64 = 59.7275;
		self.frame_count += 1;

		let elapsed = self.last_fps_check.elapsed();

		if elapsed.as_secs_f64() >= 1.0 {
			self.emu_fps = self.frame_count as f64 / elapsed.as_secs_f64();
			self.emu_speed_percent = (self.emu_fps / TARGET_FPS) * 100.0;

			self.frame_count = 0;
			self.last_fps_check = Instant::now();
		}
	}
}
