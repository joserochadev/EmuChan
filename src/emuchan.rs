#![allow(dead_code)]
use crate::common::boot::BOOT_DMG;
use crate::core::bus::BUS;
use crate::core::cartridge::Cartridge;
use crate::core::cpu::register::Flags;
use crate::core::cpu::{register::Register, CPU};
// use crate::core::cpu::register
use crate::core::ppu::PPU;
use crate::debug::debugger::Debugger;
use crate::debug::disassembler::Disassembler;
use crate::debug::messages::{
	CpuDebugState, DebugCommand, DebugEvent, DisassemblyView, EmulatorCommand, EmulatorEvent,
	EmulatorState, TileDebugData, TileMapDebugData,
};
use crate::tests::sm83::SM83;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepMode {
	None,
	Instruction,
	Frame,
}

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

	debugger: Debugger,

	step_mode: StepMode,

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
			debugger: Debugger::new(),
			step_mode: StepMode::None,
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

				EmulatorCommand::Debug(debug_cmd) => {
					self.process_debug_command(debug_cmd);
				}
			}
		}
	}

	fn process_debug_command(&mut self, cmd: DebugCommand) {
		match cmd {
			DebugCommand::AddBreakpoint(addr) => {
				self.debugger.add_breakpoint(addr);
				let bps = self.debugger.get_breakpoints();
				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::BreakpointUpdated(bps)));
			}

			DebugCommand::RemoveBreakpoint(addr) => {
				self.debugger.remove_breakpoint(addr);
				let bps = self.debugger.get_breakpoints();
				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::BreakpointUpdated(bps)));
			}

			DebugCommand::ClearBreakpoints => {
				self.debugger.clear_breakpoints();
				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::BreakpointUpdated(vec![])));
			}

			DebugCommand::ToggleBreakpoint(addr) => {
				self.debugger.toggle_breakpoint(addr);
				let bps = self.debugger.get_breakpoints();
				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::BreakpointUpdated(bps)));
			}

			DebugCommand::RequestCpuState => {
				let cpu = self.cpu.lock().unwrap();
				let state = CpuDebugState {
					a: cpu.reg.a,
					b: cpu.reg.b,
					c: cpu.reg.c,
					d: cpu.reg.d,
					e: cpu.reg.e,
					f: cpu.reg.f,
					h: cpu.reg.h,
					l: cpu.reg.l,
					pc: cpu.reg.pc,
					sp: cpu.reg.sp,
					flag_c: cpu.reg.get_flag(Flags::C) == 1,
					flag_h: cpu.reg.get_flag(Flags::H) == 1,
					flag_n: cpu.reg.get_flag(Flags::N) == 1,
					flag_z: cpu.reg.get_flag(Flags::Z) == 1,
					ime: cpu.reg.ime,
					total_cycles: cpu.cycles,
				};

				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::CpuState(state)));
			}

			DebugCommand::RequestDisassembly { pc, before, after } => {
				let bus = self.bus.lock().unwrap();
				let start_addr = pc.saturating_sub((before * 3) as u16);
				let instructions =
					Disassembler::disassemble_range(&bus.memory, start_addr, before + after + 1);

				let view = DisassemblyView {
					current_pc: pc,
					instructions,
				};

				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::DisassemblyUpdate(view)));
			}

			DebugCommand::ReadMemory(addr, count) => {
				let bus = self.bus.lock().unwrap();
				// let end = (addr as usize + count).min(bus.memory.len());
				// let data = bus.memory[addr as usize..end].to_vec();
				let mut data = Vec::new();

				for i in 0..count {
					if i >= bus.memory.len() {
						return;
					}

					let value = bus.read(addr + i as u16);
					data.push(value);
				}

				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::MemoryRead {
						address: addr,
						data,
					}));
			}

			DebugCommand::SetTrace(enabled) => {
				self.debugger.set_trace(enabled);
			}

			DebugCommand::StepInstruction => {
				self.step_mode = StepMode::Instruction;
				if self.emulator_state == EmulatorState::Paused {
					self.emulator_state = EmulatorState::Running;
					let _ = self
						.event_tx
						.send(EmulatorEvent::StateChanged(EmulatorState::Running));
				}
			}

			DebugCommand::StepFrame => {
				self.step_mode = StepMode::Frame;
				if self.emulator_state == EmulatorState::Paused {
					self.emulator_state = EmulatorState::Running;
					let _ = self
						.event_tx
						.send(EmulatorEvent::StateChanged(EmulatorState::Running));
				}
			}

			DebugCommand::RunSM83Test(path) => {
				self.run_sm83_test(path);
			}

			DebugCommand::RequestPpuState => {
				let ppu = self.ppu.lock().unwrap();
				let (state, _) = ppu.get_debug_state();

				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::PpuState(state)));
			}

			DebugCommand::RequestTileData { tile_index } => {
				let ppu = self.ppu.lock().unwrap();
				let pixels = ppu.get_tile_data(tile_index);

				let tile_data = TileDebugData { tile_index, pixels };

				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::TileData(tile_data)));
			}

			DebugCommand::RequestTileMap { map_select } => {
				let ppu = self.ppu.lock().unwrap();
				let tiles = ppu.get_tilemap_data(map_select);

				let tilemap_data = TileMapDebugData { map_select, tiles };

				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::TileMapData(tilemap_data)));
			}

			_ => {}
		}
	}

	fn run_sm83_test(&mut self, path: PathBuf) {
		let test_name = path
			.file_name()
			.and_then(|n| n.to_str())
			.unwrap_or("Unknown")
			.to_string();

		let _ = self
			.event_tx
			.send(EmulatorEvent::Debug(DebugEvent::TestStarted {
				test_name: test_name.clone(),
			}));

		let mut sm83 = SM83::new();

		let result = sm83.run_test(path.to_string_lossy().to_string());

		match result {
			Ok(report) => {
				let summary_passed = report.passed == report.total;
				let summary_msg =
					format!("Passed: {}/{} | Failed: {}", report.passed, report.total, report.failed);

				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::TestResult {
						test_name: test_name.clone(),
						passed: summary_passed,
						message: summary_msg,
					}));

				for (idx, failure) in report.failures.iter().enumerate() {
					let failure_msg = format!(
						"Test #{}: {}\nExpected: {}\nGot: {}",
						idx + 1,
						failure.test_name,
						failure.expected,
						failure.actual
					);

					let _ = self
						.event_tx
						.send(EmulatorEvent::Debug(DebugEvent::TestResult {
							test_name: failure.test_name.clone(),
							passed: false,
							message: failure_msg,
						}));
				}

				if summary_passed {
					let _ = self
						.event_tx
						.send(EmulatorEvent::Debug(DebugEvent::TestResult {
							test_name: "Summary".to_string(),
							passed: true,
							message: "✅ All tests passed!".to_string(),
						}));
				} else {
					let _ = self
						.event_tx
						.send(EmulatorEvent::Debug(DebugEvent::TestResult {
							test_name: "Summary".to_string(),
							passed: false,
							message: format!("❌ {} test(s) failed", report.failed),
						}));
				}
			}
			Err(e) => {
				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::TestResult {
						test_name,
						passed: false,
						message: format!("Error: {}", e),
					}));
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
			let should_break = {
				let cpu = self.cpu.lock().unwrap();
				self.debugger.should_break(cpu.reg.pc)
			};

			if should_break {
				let cpu = self.cpu.lock().unwrap();
				let _ = self
					.event_tx
					.send(EmulatorEvent::Debug(DebugEvent::BreakpoitHit {
						address: cpu.reg.pc,
					}));

				self.emulator_state = EmulatorState::Paused;
				let _ = self
					.event_tx
					.send(EmulatorEvent::StateChanged(EmulatorState::Paused));

				break;
			}

			let cycles_executed = {
				let mut cpu = self.cpu.lock().unwrap();

				if self.debugger.is_trace_enable() {
					let pc = cpu.reg.pc;
					let bus = self.bus.lock().unwrap();
					let instr = Disassembler::disassemble_at(&bus.memory, pc);
					let _ = self
						.event_tx
						.send(EmulatorEvent::Debug(DebugEvent::InstructionExecuted {
							address: pc,
							mnemonic: instr.mnemonic,
							operands: instr.operands,
							cycles: instr.cycles,
						}));
				}

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

			match self.step_mode {
				StepMode::Instruction => {
					self.step_mode = StepMode::None;
					self.emulator_state = EmulatorState::Paused;
					let _ = self
						.event_tx
						.send(EmulatorEvent::StateChanged(EmulatorState::Paused));
					break;
				}

				StepMode::Frame => {}
				StepMode::None => {}
			}
		}

		if self.step_mode == StepMode::Frame && cycles_this_frame >= CYCLES_PER_FRAME {
			self.step_mode = StepMode::None;
			self.emulator_state = EmulatorState::Paused;
			let _ = self
				.event_tx
				.send(EmulatorEvent::StateChanged(EmulatorState::Paused));
		}

		self.update_fps();

		let video_buffer = self.get_video_buffer();
		let _ = self
			.event_tx
			.send(EmulatorEvent::FrameReady(Box::new(video_buffer)));
	}
}
