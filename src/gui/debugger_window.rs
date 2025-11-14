// src/gui/debugger_window.rs

use crate::debug::messages::*;
use eframe::egui::*;
use std::{fmt::format, sync::mpsc::Sender};

pub struct DebuggerWindow {
	// Comunicação
	command_tx: Sender<EmulatorCommand>,

	// Estado das janelas
	show_registers: bool,
	show_disassembly: bool,
	show_memory: bool,
	show_breakpoints: bool,
	show_trace: bool,
	show_sm83_test: bool,

	// Registradores
	cpu_state: Option<CpuDebugState>,

	// Disassembly
	disassembly: Option<DisassemblyView>,
	disassembly_pc: u16,
	disassembly_before: u16,
	disassembly_after: u16,

	// Memory
	memory_address: String,
	memory_size: String,
	memory_data: Vec<u8>,
	memory_base_address: u16,

	// Breakpoints
	breakpoint_input: String,
	breakpoints: Vec<u16>,

	// Trace
	trace_enabled: bool,
	trace_logs: Vec<String>,
	auto_scroll_trace: bool,

	// SM83 Test
	test_logs: Vec<String>,
	auto_scroll_test: bool,
}

impl DebuggerWindow {
	pub fn new(command_tx: Sender<EmulatorCommand>) -> Self {
		Self {
			command_tx,
			show_registers: false,
			show_disassembly: false,
			show_memory: false,
			show_breakpoints: false,
			show_trace: false,
			show_sm83_test: false,
			cpu_state: None,
			disassembly: None,
			disassembly_pc: 0x100,
			disassembly_before: 5,
			disassembly_after: 5,
			memory_address: "0x0000".to_string(),
			memory_size: "256".to_string(),
			memory_data: vec![],
			memory_base_address: 0,
			breakpoint_input: String::new(),
			breakpoints: vec![],
			trace_enabled: false,
			trace_logs: vec![],
			auto_scroll_trace: true,
			test_logs: vec![],
			auto_scroll_test: true,
		}
	}

	/// Mostra o menu principal de debug
	pub fn show_menu(&mut self, ui: &mut Ui) {
		ui.menu_button("Debug", |ui| {
			ui.checkbox(&mut self.show_registers, "📊 Registers");
			ui.checkbox(&mut self.show_disassembly, "📝 Disassembly");
			ui.checkbox(&mut self.show_memory, "💾 Memory");
			ui.checkbox(&mut self.show_breakpoints, "🛑 Breakpoints");
			ui.checkbox(&mut self.show_trace, "📜 Trace Logs");
			ui.checkbox(&mut self.show_sm83_test, "🧪 SM83 Test Runner");

			ui.separator();

			if ui.button("Open All").clicked() {
				self.show_all();
				ui.close_menu();
			}

			if ui.button("Close All").clicked() {
				self.hide_all();
				ui.close_menu();
			}
		});
	}

	pub fn show_all(&mut self) {
		self.show_registers = true;
		self.show_disassembly = true;
		self.show_memory = true;
		self.show_breakpoints = true;
		self.show_trace = true;
		self.show_sm83_test = true;
	}

	pub fn hide_all(&mut self) {
		self.show_registers = false;
		self.show_disassembly = false;
		self.show_memory = false;
		self.show_breakpoints = false;
		self.show_trace = false;
		self.show_sm83_test = false;
	}

	pub fn show(&mut self, ctx: &Context) {
		// Requisita estado do CPU periodicamente
		let _ = self
			.command_tx
			.send(EmulatorCommand::Debug(DebugCommand::RequestCpuState));

		// Cada janela separada
		if self.show_registers {
			self.show_registers_window(ctx);
		}

		if self.show_disassembly {
			self.show_disassembly_window(ctx);
		}

		if self.show_memory {
			self.show_memory_window(ctx);
		}

		if self.show_breakpoints {
			self.show_breakpoints_window(ctx);
		}

		if self.show_trace {
			self.show_trace_window(ctx);
		}

		if self.show_sm83_test {
			self.show_sm83_test_window(ctx);
		}
	}

	fn show_registers_window(&mut self, ctx: &Context) {
		let mut open = self.show_registers;
		Window::new("📊 Registers")
			.open(&mut open)
			.resizable(true)
			.default_pos(Pos2 { x: 5.0, y: 25.0 })
			.default_width(180.0)
			.show(ctx, |ui| {
				self.draw_registers(ui);
			});
		self.show_registers = open;
	}

	fn show_disassembly_window(&mut self, ctx: &Context) {
		let mut open = self.show_disassembly;
		Window::new("📝 Disassembly")
			.open(&mut open)
			.resizable(true)
			.default_pos(Pos2 { x: 210.0, y: 25.0 })
			.default_width(230.0)
			.default_height(600.0)
			.show(ctx, |ui| {
				self.draw_disassembly(ui);
			});
		self.show_disassembly = open;
	}

	fn show_memory_window(&mut self, ctx: &Context) {
		let mut open = self.show_memory;
		Window::new("💾 Memory")
			.open(&mut open)
			.resizable(true)
			.default_width(400.0)
			.default_height(400.0)
			.show(ctx, |ui| {
				self.draw_memory(ui);
			});
		self.show_memory = open;
	}

	fn show_breakpoints_window(&mut self, ctx: &Context) {
		let mut open = self.show_breakpoints;
		Window::new("🛑 Breakpoints")
			.open(&mut open)
			.resizable(true)
			.default_pos(Pos2 { x: 5.0, y: 250.0 })
			.default_width(180.0)
			.default_height(200.0)
			.show(ctx, |ui| {
				self.draw_breakpoints(ui);
			});
		self.show_breakpoints = open;
	}

	fn show_trace_window(&mut self, ctx: &Context) {
		let mut open = self.show_trace;
		Window::new("📜 Trace Logs")
			.open(&mut open)
			.resizable(true)
			.default_width(400.0)
			.default_height(250.0)
			.show(ctx, |ui| {
				self.draw_trace(ui);
			});
		self.show_trace = open;
	}

	fn show_sm83_test_window(&mut self, ctx: &Context) {
		let mut open = self.show_sm83_test;
		Window::new("🧪 SM83 Test Runner")
			.open(&mut open)
			.resizable(true)
			.default_width(600.0)
			.default_height(400.0)
			.show(ctx, |ui| {
				self.draw_sm83_test(ui);
			});
		self.show_sm83_test = open;
	}

	fn draw_registers(&mut self, ui: &mut Ui) {
		ui.heading("Controls");

		ui.vertical(|ui| {
			if ui.button("Step Instruction").clicked() {
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::StepInstruction));
			}

			if ui.button("Step Frame").clicked() {
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::StepFrame));
			}

			if ui.button("Continue").clicked() {
				let _ = self.command_tx.send(EmulatorCommand::TogglePause);
			}

			ui.separator();
		});

		if let Some(cpu) = &self.cpu_state {
			Grid::new("registers_grid")
				.num_columns(4)
				.striped(true)
				.spacing([10.0, 5.0])
				.show(ui, |ui| {
					// 8-bit registers
					ui.label("A:");
					ui.monospace(format!("{:02X}", cpu.a));
					ui.label("F:");
					ui.monospace(format!("{:02X}", cpu.f));
					ui.end_row();

					ui.label("B:");
					ui.monospace(format!("{:02X}", cpu.b));
					ui.label("C:");
					ui.monospace(format!("{:02X}", cpu.c));
					ui.end_row();

					ui.label("D:");
					ui.monospace(format!("{:02X}", cpu.d));
					ui.label("E:");
					ui.monospace(format!("{:02X}", cpu.e));
					ui.end_row();

					ui.label("H:");
					ui.monospace(format!("{:02X}", cpu.h));
					ui.label("L:");
					ui.monospace(format!("{:02X}", cpu.l));
					ui.end_row();

					// 16-bit registers
					ui.label("PC:");
					ui.monospace(format!("{:04X}", cpu.pc));
					ui.label("SP:");
					ui.monospace(format!("{:04X}", cpu.sp));
					ui.end_row();
				});

			ui.separator();

			// Flags
			ui.horizontal(|ui| {
				ui.label("Flags:");

				let flag_color = |set| {
					if set {
						Color32::GREEN
					} else {
						Color32::DARK_GRAY
					}
				};

				ui.colored_label(flag_color(cpu.flag_z), "Z");
				ui.colored_label(flag_color(cpu.flag_n), "N");
				ui.colored_label(flag_color(cpu.flag_h), "H");
				ui.colored_label(flag_color(cpu.flag_c), "C");

				ui.separator();

				ui.colored_label(flag_color(cpu.ime == 1), "IME");
			});

			ui.separator();

			// Counters
			ui.label(format!("Cycles: {}", cpu.total_cycles));
		} else {
			ui.colored_label(Color32::GRAY, "Waiting CPU State...");
		}
	}

	fn draw_disassembly(&mut self, ui: &mut Ui) {
		ui.vertical(|ui| {
			ui.label("PC:");
			let mut pc_text = format!("{:04X}", self.disassembly_pc);
			if ui.text_edit_singleline(&mut pc_text).changed() {
				if let Ok(val) = u16::from_str_radix(&pc_text.trim_start_matches("0x"), 16) {
					self.disassembly_pc = val;
				}
			}

			ui.label("Before:");
			ui.add(Slider::new(&mut self.disassembly_before, 1..=20));

			ui.label("After:");
			ui.add(Slider::new(&mut self.disassembly_after, 1..=20));

			if ui.button("🔄 Update").clicked() {
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::RequestDisassembly {
						pc: self.disassembly_pc,
						before: self.disassembly_before as usize,
						after: self.disassembly_after as usize,
					}));
			}

			if ui.button("➡️ Go to PC").clicked() {
				if let Some(cpu) = &self.cpu_state {
					self.disassembly_pc = cpu.pc;
					let _ = self
						.command_tx
						.send(EmulatorCommand::Debug(DebugCommand::RequestDisassembly {
							pc: cpu.pc,
							before: self.disassembly_before as usize,
							after: self.disassembly_after as usize,
						}));
				}
			}
		});

		ui.separator();

		// Mostrar disassembly em grid
		if let Some(view) = &self.disassembly {
			ScrollArea::vertical()
				.auto_shrink([false; 2])
				.show(ui, |ui| {
					Grid::new("disassembly_grid")
						.num_columns(5)
						.striped(true)
						.spacing([10.0, 2.0])
						.show(ui, |ui| {
							// Header
							ui.strong("");
							ui.strong("BP");
							ui.strong("Address");
							ui.strong("Mnemonic");
							ui.strong("Operands");
							ui.end_row();

							for instr in &view.instructions {
								let is_current = instr.address == view.current_pc;
								let has_breakpoint = self.breakpoints.contains(&instr.address);
								let color = if is_current {
									Color32::YELLOW
								} else {
									Color32::LIGHT_GRAY
								};

								// Marcador
								let marker = if is_current { "▶" } else { "" };
								ui.colored_label(color, marker);

								let bp_text = if has_breakpoint { "🔴" } else { "⚪" };
								if ui.small_button(bp_text).clicked() {
									let _ = self
										.command_tx
										.send(EmulatorCommand::Debug(DebugCommand::ToggleBreakpoint(instr.address)));
								}

								// Endereço
								ui.colored_label(color, format!("{:04X}", instr.address));

								// Mnemônico
								ui.colored_label(color, &instr.mnemonic);

								// Operandos
								ui.colored_label(color, &instr.operands);

								ui.end_row();
							}
						});
				});
		} else {
			ui.colored_label(Color32::GRAY, "No disassembly available. Click 'Update' to fetch.");
		}
	}

	fn draw_memory(&mut self, ui: &mut Ui) {
		ui.vertical(|ui| {
			ui.label("Address:");
			ui.text_edit_singleline(&mut self.memory_address);

			ui.label("Size:");
			ui.text_edit_singleline(&mut self.memory_size);

			if ui.button("📖 Read").clicked() {
				if let Ok(addr) = u16::from_str_radix(
					self
						.memory_address
						.trim_start_matches("0x")
						.trim_start_matches("0X"),
					16,
				) {
					if let Ok(size) = self.memory_size.parse::<usize>() {
						self.memory_base_address = addr;
						let _ = self
							.command_tx
							.send(EmulatorCommand::Debug(DebugCommand::ReadMemory(addr, size)));
					}
				}
			}
		});

		ui.separator();

		// Mostrar memória em hexdump
		ScrollArea::vertical()
			.auto_shrink([false; 2])
			.show(ui, |ui| {
				if !self.memory_data.is_empty() {
					for (line_idx, chunk) in self.memory_data.chunks(16).enumerate() {
						let addr = self
							.memory_base_address
							.wrapping_add((line_idx * 16) as u16);

						ui.horizontal(|ui| {
							// Endereço
							ui.monospace(format!("{:04X}:", addr));

							// Bytes em hexadecimal
							let mut hex_str = String::new();
							for byte in chunk {
								hex_str.push_str(&format!("{:02X} ", byte));
							}

							// Padding se a última linha for incompleta
							if chunk.len() < 16 {
								for _ in chunk.len()..16 {
									hex_str.push_str("   ");
								}
							}

							ui.monospace(hex_str);

							ui.label("|");

							// Caracteres ASCII
							let mut ascii_str = String::new();
							for byte in chunk {
								let ch = if *byte >= 32 && *byte < 127 { *byte as char } else { '.' };
								ascii_str.push(ch);
							}

							ui.monospace(ascii_str);
						});
					}
				} else {
					ui.colored_label(Color32::GRAY, "No memory data. Click 'Read' to fetch.");
				}
			});
	}

	fn draw_breakpoints(&mut self, ui: &mut Ui) {
		ui.vertical(|ui| {
			ui.label("Address:");
			ui.text_edit_singleline(&mut self.breakpoint_input);

			ui.horizontal(|ui| {
				if ui.button("➕ Add").clicked() {
					if let Ok(addr) = u16::from_str_radix(
						self
							.breakpoint_input
							.trim_start_matches("0x")
							.trim_start_matches("0X"),
						16,
					) {
						let _ = self
							.command_tx
							.send(EmulatorCommand::Debug(DebugCommand::AddBreakpoint(addr)));
						self.breakpoint_input.clear();
					}
				}

				if ui.button("🗑️ Clear All").clicked() {
					let _ = self
						.command_tx
						.send(EmulatorCommand::Debug(DebugCommand::ClearBreakpoints));
				}
			});
		});

		ui.separator();

		ui.label(format!("Total: {} breakpoint(s)", self.breakpoints.len()));

		ui.separator();

		ScrollArea::vertical()
			.auto_shrink([false; 2])
			.show(ui, |ui| {
				let mut to_remove = None;

				Grid::new("breakpoints_grid")
					.num_columns(2)
					.striped(true)
					.spacing([10.0, 5.0])
					.show(ui, |ui| {
						for bp in &self.breakpoints {
							ui.monospace(format!("{:04X}", bp));

							if ui.button("❌").clicked() {
								to_remove = Some(*bp);
							}
							ui.end_row();
						}
					});

				if let Some(bp) = to_remove {
					let _ = self
						.command_tx
						.send(EmulatorCommand::Debug(DebugCommand::RemoveBreakpoint(bp)));
				}
			});
	}

	fn draw_trace(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.checkbox(&mut self.trace_enabled, "🔴 Enable Trace");

			if ui.button("🗑️ Clear Logs").clicked() {
				self.trace_logs.clear();
			}

			ui.checkbox(&mut self.auto_scroll_trace, "Auto-scroll");

			ui.label(format!("Total: {} instructions", self.trace_logs.len()));
		});

		ui.separator();

		ScrollArea::vertical()
			.auto_shrink([false; 2])
			.stick_to_bottom(self.auto_scroll_trace)
			.show(ui, |ui| {
				if self.trace_logs.is_empty() {
					ui.colored_label(Color32::GRAY, "No trace logs available.");
					ui.colored_label(Color32::GRAY, "Enable tracing to start logging executed instructions.");
				} else {
					for log in &self.trace_logs {
						if log.contains("🛑") {
							ui.colored_label(Color32::RED, log);
						} else {
							ui.monospace(log);
						}
					}
				}
			});
	}

	fn draw_sm83_test(&mut self, ui: &mut Ui) {
		ui.heading("SM83 CPU Test Runner");
		ui.separator();

		ui.horizontal(|ui| {
			if ui.button("Select File Test").clicked() {
				if let Some(path) = rfd::FileDialog::new()
					.add_filter("JSON Test File", &["json"])
					.pick_file()
				{
					let _ = self
						.command_tx
						.send(EmulatorCommand::Debug(DebugCommand::RunSM83Test(path)));
				}
			}

			if ui.button("Clear Logs").clicked() {
				self.test_logs.clear();
			}

			ui.checkbox(&mut self.auto_scroll_test, "Auto-scroll");
		});

		ui.separator();

		ui.label(format!("Test logs: {} entries", self.test_logs.len()));

		ScrollArea::vertical()
			.auto_shrink([false; 2])
			.stick_to_bottom(self.auto_scroll_test)
			.show(ui, |ui| {
				if self.test_logs.is_empty() {
					ui.colored_label(Color32::GRAY, "No tests run yet.");
					ui.colored_label(Color32::GRAY, "Select a JSON test file to begin.");
				} else {
					for log in &self.test_logs {
						let color = if log.contains("✅") {
							Color32::GREEN
						} else if log.contains("❌") {
							Color32::RED
						} else if log.contains("🧪") {
							Color32::YELLOW
						} else {
							Color32::WHITE
						};

						ui.colored_label(color, log);
					}
				}
			});
	}

	/// Atualiza a janela com dados do emulador
	pub fn handle_debug_event(&mut self, event: DebugEvent) {
		match event {
			DebugEvent::CpuState(state) => {
				self.cpu_state = Some(state);
			}

			DebugEvent::DisassemblyUpdate(view) => {
				self.disassembly = Some(view);
			}

			DebugEvent::MemoryRead { address: _, data } => {
				self.memory_data = data;
			}

			DebugEvent::BreakpointUpdated(bps) => {
				self.breakpoints = bps;
			}

			DebugEvent::InstructionExecuted {
				address,
				mnemonic,
				operands,
				cycles,
			} => {
				if self.trace_enabled {
					self
						.trace_logs
						.push(format!("{:04X}: {} {} ({} cycles)", address, mnemonic, operands, cycles));

					// Manter apenas últimas 1000 instruções
					if self.trace_logs.len() > 1000 {
						self.trace_logs.remove(0);
					}
				}
			}

			DebugEvent::BreakpoitHit { address } => {
				self.disassembly_pc = address;
				self
					.trace_logs
					.push(format!("🛑 Breakpoint Hit: {:04X}", address));

				// Auto-abrir janelas relevantes quando um breakpoint é atingido
				self.show_registers = true;
				self.show_disassembly = true;

				// Atualizar disassembly automaticamente
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::RequestDisassembly {
						pc: address,
						before: self.disassembly_before as usize,
						after: self.disassembly_after as usize,
					}));
			}

			DebugEvent::TestResult {
				test_name,
				passed,
				message,
			} => {
				let icon = if passed { "✅" } else { "❌" };
				let log = format!("{} Test: {} - {}", icon, test_name, message);
				self.test_logs.push(log);
			}

			DebugEvent::TestStarted { test_name } => {
				self
					.test_logs
					.push(format!("🧪 Running test: {}", test_name));
			}

			_ => {}
		}
	}
}
