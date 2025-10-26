#![allow(dead_code)]
use crate::common::boot::BOOT_DMG;
use crate::core::bus::BUS;
use crate::core::cartridge::Cartridge;
use crate::core::cpu::{register::Register, CPU};
// use crate::core::cpu::register
use crate::core::ppu::PPU;
use crate::debug::messages::{EmulatorCommand, EmulatorEvent, EmulatorState};

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// #[derive(Default, Clone, PartialEq)]
// pub enum EmulationState {
// 	#[default]
// 	PAUSED,
// 	RUNNING,
// 	STEP,
// }

// #[derive(Default)]
// pub struct EmuStateFlags {
// 	pub z: bool,
// 	pub h: bool,
// 	pub n: bool,
// 	pub c: bool,
// }

// pub struct EmuState {
// 	pub flags: EmuStateFlags,
// 	pub emulation_state: EmulationState,
// }

pub struct EmuChan {
	pub bus: Arc<Mutex<BUS>>,
	pub cpu: Arc<Mutex<CPU>>,
	pub ppu: Arc<Mutex<PPU>>,
	pub cartridge: Arc<Mutex<Cartridge>>,

	pub emulator_state: EmulatorState,

	frame_count: u32,
	last_fps_check: Instant,
	pub emu_fps: f64,
	pub emu_speed_percent: f64,

	pub command_rx: Receiver<EmulatorCommand>,
	pub event_tx: Sender<EmulatorEvent>,
}

impl EmuChan {
	pub fn new(command_rx: Receiver<EmulatorCommand>, event_tx: Sender<EmulatorEvent>) -> Self {
		let bus = Arc::new(Mutex::new(BUS::new()));
		let cpu = Arc::new(Mutex::new(CPU::new(Arc::clone(&bus))));
		let ppu = Arc::new(Mutex::new(PPU::new()));
		let cartridge = Arc::new(Mutex::new(Cartridge::new()));

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
			emulator_state: EmulatorState::Idle,
			frame_count: 0,
			last_fps_check: Instant::now(),
			emu_fps: 0.0,
			emu_speed_percent: 0.0,
			command_rx,
			event_tx,
		}
	}

	pub fn process_commands(&mut self) {
		while let Ok(cmd) = self.command_rx.try_recv() {
			match cmd {
				EmulatorCommand::LoadRom(path) => {
					self.load_rom_internal(path);
				}

				EmulatorCommand::TogglePause => {
					self.emulator_state = match self.emulator_state {
						EmulatorState::Running => EmulatorState::Paused,
						EmulatorState::Paused => EmulatorState::Running,
						other => other,
					};

					let _ = self
						.event_tx
						.send(EmulatorEvent::StateChanged(self.emulator_state));
				}

				EmulatorCommand::Stop => {
					self.emulator_state = EmulatorState::Stoped;
					let _ = self
						.event_tx
						.send(EmulatorEvent::StateChanged(self.emulator_state));
				}

				EmulatorCommand::Reset => {
					self.reset_internal();
				}

				EmulatorCommand::SetSpeed(speed) => {
					todo!()
				}

				EmulatorCommand::SaveState(path) => {
					todo!()
				}

				EmulatorCommand::LoadState(path) => {
					todo!()
				}

				EmulatorCommand::JoypadInput(button, is_pressed) => {
					todo!()
				}
			}
		}
	}

	fn load_rom_internal(&mut self, path: PathBuf) {
		let mut cartridge = self.cartridge.lock().unwrap();

		match std::fs::metadata(&path) {
			Ok(_) => {
				cartridge.load_rom(path.to_string_lossy().to_string());
				let title = cartridge.game_title.clone();

				let _ = self.event_tx.send(EmulatorEvent::RomLoaded {
					title,
					size: cartridge.rom.len(),
				});

				self.emulator_state = EmulatorState::Running;
				let _ = self
					.event_tx
					.send(EmulatorEvent::StateChanged(self.emulator_state));
			}

			Err(e) => {
				let _ = self
					.event_tx
					.send(EmulatorEvent::RomLoadError(format!("Failed to load ROM: {}", e)));
			}
		}
	}

	// Reset Emulator
	fn reset_internal(&mut self) {
		// CPU Reset
		{
			let mut cpu = self.cpu.lock().unwrap();
			cpu.reg = Register::new()
		}

		// PPU Reset
		{
			let mut ppu = self.ppu.lock().unwrap();
			*ppu = PPU::new();
		}

		// Reload boot ROM
		{
			let mut bus = self.bus.lock().unwrap();
			bus.memory[0..=255].copy_from_slice(&BOOT_DMG);
			bus.disable_boot = false;
		}

		self.emulator_state = EmulatorState::Running;
		let _ = self
			.event_tx
			.send(EmulatorEvent::StateChanged(self.emulator_state));
	}

	fn update_fps(&mut self) {
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
		self.process_commands();

		if self.emulator_state != EmulatorState::Running {
			return;
		}

		const CYCLES_PER_FRAME: u32 = 70224; // ~70k T-cycles per frame
		let mut cycles_this_frame = 0;

		while cycles_this_frame < CYCLES_PER_FRAME {
			let cycles_executed = {
				let mut cpu = self.cpu.lock().unwrap();
				match cpu.step() {
					Err(e) => {
						self.emulator_state = EmulatorState::Paused;
						let _ = self
							.event_tx
							.send(EmulatorEvent::Error(format!("CPU Error: {}", e)));
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

		self.update_fps();

		let video_buffer = self.get_video_buffer();
		let _ = self
			.event_tx
			.send(EmulatorEvent::FrameReady(Box::new(video_buffer)));
	}
}
