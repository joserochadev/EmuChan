// src/gui/debugger_window.rs

use crate::debug::messages::*;
use eframe::egui::*;
use std::sync::mpsc::Sender;

pub struct DebuggerWindow {
	// Comunicação
	command_tx: Sender<EmulatorCommand>,

	// Estado de visibilidade dos painéis
	pub show_debugger: bool,
	show_sm83_test: bool,

	// Registradores
	cpu_state: Option<CpuDebugState>,

	// Disassembly
	disassembly: Option<DisassemblyView>,
	disassembly_pc: u16,
	disassembly_before: u16,
	disassembly_after: u16,
	follow_pc: bool,
	last_pc: u16,

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
			show_debugger: false,
			show_sm83_test: false,
			cpu_state: None,
			disassembly: None,
			disassembly_pc: 0x100,
			disassembly_before: 5,
			disassembly_after: 10,
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
			follow_pc: true,
			last_pc: 0,
		}
	}

	pub fn show_menu(&mut self, ui: &mut Ui) {
		ui.menu_button("Debug", |ui| {
			ui.checkbox(&mut self.show_debugger, "🐛 Show Debugger");

			ui.separator();

			if ui.button("🧪 SM83 Test Runner").clicked() {
				self.show_sm83_test = true;
				ui.close_menu();
			}
		});
	}

	pub fn show_panels(&mut self, ctx: &Context) {
		if !self.show_debugger {
			return;
		}

		// Requisita estado do CPU periodicamente
		let _ = self
			.command_tx
			.send(EmulatorCommand::Debug(DebugCommand::RequestCpuState));

		// Auto-update disassembly se follow_pc estiver ativo
		if self.follow_pc {
			if let Some(cpu) = &self.cpu_state {
				if cpu.pc != self.last_pc {
					self.last_pc = cpu.pc;
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
		}

		// Painel Esquerdo: Registradores + Breakpoints
		SidePanel::left("debug_left_panel")
			.resizable(true)
			.default_width(200.0)
			.min_width(180.0)
			.show(ctx, |ui| {
				self.draw_left_panel(ui);
			});

		// Painel Direito: PPU
		SidePanel::right("debug_right_panel")
			.resizable(true)
			.default_width(280.0)
			.min_width(250.0)
			.show(ctx, |ui| {
				self.draw_right_panel(ui);
			});

		// Painel Inferior: Memory + Trace
		TopBottomPanel::bottom("debug_bottom_panel")
			.resizable(true)
			.default_height(200.0)
			.min_height(300.0)
			.show(ctx, |ui| {
				self.draw_bottom_panel(ui);
			});
	}

	pub fn show_test_window(&mut self, ctx: &Context) {
		let mut open = self.show_sm83_test;
		Window::new("SM83 Test Runner")
			.open(&mut open)
			.resizable(true)
			.default_width(600.0)
			.default_height(400.0)
			.show(ctx, |ui| {
				self.draw_sm83_test(ui);
			});

		self.show_sm83_test = open;
	}

	fn draw_left_panel(&mut self, ui: &mut Ui) {
		// Controles de execução
		ui.heading("⚙️ Controls");
		ui.separator();

		ui.vertical(|ui| {
			if ui.button("▶️ Step Instruction").clicked() {
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::StepInstruction));
			}

			if ui.button("⏭️ Step Frame").clicked() {
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::StepFrame));
			}

			if ui.button("▶️ Continue").clicked() {
				let _ = self.command_tx.send(EmulatorCommand::TogglePause);
			}
		});

		ui.add_space(10.0);

		// Registradores
		ui.heading("📊 Registers");
		ui.separator();

		if let Some(cpu) = &self.cpu_state {
			Grid::new("registers_grid")
				.num_columns(4)
				.striped(true)
				.spacing([5.0, 3.0])
				.show(ui, |ui| {
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

					ui.label("PC:");
					ui.monospace(format!("{:04X}", cpu.pc));
					ui.label("SP:");
					ui.monospace(format!("{:04X}", cpu.sp));
					ui.end_row();
				});

			ui.add_space(5.0);

			ui.horizontal(|ui| {
				ui.label("Flags:");
				let flag_color = |set| if set { Color32::GREEN } else { Color32::DARK_GRAY };
				ui.colored_label(flag_color(cpu.flag_z), "Z");
				ui.colored_label(flag_color(cpu.flag_n), "N");
				ui.colored_label(flag_color(cpu.flag_h), "H");
				ui.colored_label(flag_color(cpu.flag_c), "C");
			});

			ui.add_space(5.0);
			ui.label(format!("Cycles: {}", cpu.total_cycles));
		} else {
			ui.colored_label(Color32::GRAY, "Waiting for CPU state...");
		}

		ui.add_space(20.0);

		ui.heading("Disassembly");
		ui.separator();

		ui.vertical(|ui| {
			ui.label("PC:");
			let mut pc_text = format!("{:04X}", self.disassembly_pc);

			ui.add_enabled_ui(!self.follow_pc, |ui| {
				if ui.text_edit_singleline(&mut pc_text).changed() {
					if let Ok(val) = u16::from_str_radix(&pc_text.trim_start_matches("0x"), 16) {
						self.disassembly_pc = val;
					}
				}
			});

			ui.label("Before:");
			ui.add(Slider::new(&mut self.disassembly_before, 1..=20));

			ui.label("After:");
			ui.add(Slider::new(&mut self.disassembly_after, 1..=20));

			ui.horizontal(|ui| {
				if ui.checkbox(&mut self.follow_pc, "Follow PC").changed() {
					if self.follow_pc {
						// When enabling follow mode, jump to current PC
						if let Some(cpu) = &self.cpu_state {
							self.disassembly_pc = cpu.pc;
							self.last_pc = cpu.pc;
							let _ =
								self
									.command_tx
									.send(EmulatorCommand::Debug(DebugCommand::RequestDisassembly {
										pc: cpu.pc,
										before: self.disassembly_before as usize,
										after: self.disassembly_after as usize,
									}));
						}
					}
				}

				ui.separator();

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
						let _ =
							self
								.command_tx
								.send(EmulatorCommand::Debug(DebugCommand::RequestDisassembly {
									pc: cpu.pc,
									before: self.disassembly_before as usize,
									after: self.disassembly_after as usize,
								}));
					}
				}
			});

			ui.label(format!("Breakpoints: {}", self.breakpoints.len()));
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

	fn draw_right_panel(&mut self, ui: &mut Ui) {
		ui.heading("PPU");
		ui.separator();
	}

	fn draw_bottom_panel(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.selectable_value(&mut self.show_bottom_tab(), BottomTab::Memory, "💾 Memory");
			ui.selectable_value(&mut self.show_bottom_tab(), BottomTab::Trace, "📜 Trace");
		});

		ui.separator();

		match self.show_bottom_tab() {
			BottomTab::Memory => self.draw_memory_tab(ui),
			BottomTab::Trace => self.draw_trace_tab(ui),
		}
	}

	fn show_bottom_tab(&self) -> BottomTab {
		if self.trace_enabled {
			BottomTab::Trace
		} else {
			BottomTab::Memory
		}
	}

	fn draw_memory_tab(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.label("Address:");
			ui.text_edit_singleline(&mut self.memory_address);
			ui.label("Size:");
			ui.add(Slider::new(&mut self.memory_size.parse::<usize>().unwrap_or(256), 16..=1024));

			if ui.button("📖 Read").clicked() {
				if let Ok(addr) = u16::from_str_radix(
					self
						.memory_address
						.trim_start_matches("0x")
						.trim_start_matches("0X"),
					16,
				) {
					let size = self.memory_size.parse::<usize>().unwrap_or(256);
					self.memory_base_address = addr;
					let _ = self
						.command_tx
						.send(EmulatorCommand::Debug(DebugCommand::ReadMemory(addr, size)));
				}
			}
		});

		ui.separator();

		ScrollArea::vertical().show(ui, |ui| {
			if !self.memory_data.is_empty() {
				for (line_idx, chunk) in self.memory_data.chunks(16).enumerate() {
					let addr = self
						.memory_base_address
						.wrapping_add((line_idx * 16) as u16);

					ui.horizontal(|ui| {
						ui.monospace(format!("{:04X}:", addr));

						let mut hex_str = String::new();
						for byte in chunk {
							hex_str.push_str(&format!("{:02X} ", byte));
						}
						if chunk.len() < 16 {
							for _ in chunk.len()..16 {
								hex_str.push_str("   ");
							}
						}
						ui.monospace(hex_str);

						ui.label("|");

						let mut ascii_str = String::new();
						for byte in chunk {
							let ch = if *byte >= 32 && *byte < 127 { *byte as char } else { '.' };
							ascii_str.push(ch);
						}
						ui.monospace(ascii_str);
					});
				}
			} else {
				ui.colored_label(Color32::GRAY, "No memory loaded");
			}
		});
	}

	fn draw_trace_tab(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			if ui
				.checkbox(&mut self.trace_enabled, "🔴 Enable Trace")
				.changed()
			{
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::SetTrace(self.trace_enabled)));
			}

			if ui.button("🗑️ Clear").clicked() {
				self.trace_logs.clear();
			}

			ui.label(format!("Total: {}", self.trace_logs.len()));
		});

		ui.separator();

		ScrollArea::vertical()
			.stick_to_bottom(self.auto_scroll_trace)
			.show(ui, |ui| {
				if self.trace_logs.is_empty() {
					ui.colored_label(Color32::GRAY, "No trace logs");
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
			if ui.button("📂 Select Test File").clicked() {
				if let Some(path) = rfd::FileDialog::new()
					.add_filter("JSON Test Files", &["json"])
					.pick_file()
				{
					let _ = self
						.command_tx
						.send(EmulatorCommand::Debug(DebugCommand::RunSM83Test(path)));
				}
			}

			if ui.button("🗑️ Clear Logs").clicked() {
				self.test_logs.clear();
			}
		});

		ui.separator();

		ScrollArea::vertical()
			.stick_to_bottom(self.auto_scroll_test)
			.show(ui, |ui| {
				if self.test_logs.is_empty() {
					ui.colored_label(Color32::GRAY, "No tests run yet.");
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

					if self.trace_logs.len() > 1000 {
						self.trace_logs.remove(0);
					}
				}
			}

			DebugEvent::BreakpoitHit { address } => {
				self.disassembly_pc = address;
				self.last_pc = address;
				self
					.trace_logs
					.push(format!("🛑 Breakpoint hit at {:04X}", address));

				self.show_debugger = true;

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
				self
					.test_logs
					.push(format!("{} {}: {}", icon, test_name, message));
			}

			DebugEvent::TestStarted { test_name } => {
				self.test_logs.push(format!("🧪 Running: {}", test_name));
			}

			_ => {}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomTab {
	Memory,
	Trace,
}
