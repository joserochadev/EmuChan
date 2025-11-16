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
	// breakpoint_input: String,
	breakpoints: Vec<u16>,

	// Trace
	trace_enabled: bool,
	trace_logs: Vec<String>,
	auto_scroll_trace: bool,

	// SM83 Test
	test_logs: Vec<String>,
	auto_scroll_test: bool,

	// PPU Debug
	ppu_state: Option<PpuDebugState>,
	selected_tile: u8,
	tile_data: Option<TileDebugData>,
	tilemap_data: Option<TileMapDebugData>,
	show_tilemap_1: bool, // false = 0x9800, true = 0x9C00
	tile_palette: [Color32; 4],
	current_ppu_tab: PpuTab,
	vram_tiles: Vec<Vec<u8>>,
	vram_tiles_loaded: bool,
}

impl DebuggerWindow {
	pub fn new(command_tx: Sender<EmulatorCommand>) -> Self {
		Self {
			command_tx,
			show_debugger: true,
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
			breakpoints: vec![],
			trace_enabled: false,
			trace_logs: vec![],
			auto_scroll_trace: true,
			test_logs: vec![],
			auto_scroll_test: true,
			follow_pc: true,
			last_pc: 0,
			ppu_state: None,
			selected_tile: 0,
			tile_data: None,
			tilemap_data: None,
			show_tilemap_1: false,
			tile_palette: [
				Color32::from_gray(255),
				Color32::from_gray(170),
				Color32::from_gray(85),
				Color32::from_gray(0),
			],
			current_ppu_tab: PpuTab::Registers,
			vram_tiles: Vec::new(),
			vram_tiles_loaded: false,
		}
	}

	pub fn show_menu(&mut self, ui: &mut Ui) {
		ui.menu_button("Debug", |ui| {
			ui.checkbox(&mut self.show_debugger, "Show Debugger");

			ui.separator();

			if ui.button("SM83 Test Runner").clicked() {
				self.show_sm83_test = true;
				ui.close_menu();
			}
		});
	}

	pub fn show_panels(&mut self, ctx: &Context) {
		if !self.show_debugger {
			return;
		}

		let _ = self
			.command_tx
			.send(EmulatorCommand::Debug(DebugCommand::RequestCpuState));

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
			.default_width(700.0)
			// .min_width(250.0)
			.show(ctx, |ui| {
				self.draw_right_panel(ui);
			});

		// Painel Inferior: Memory + Trace
		TopBottomPanel::bottom("debug_bottom_panel")
			.resizable(true)
			.default_height(350.0)
			.min_height(350.0)
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
		ui.heading("⚙ Controls");
		ui.separator();

		ui.vertical(|ui| {
			if ui.button("▶ Step Instruction").clicked() {
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::StepInstruction));
			}

			if ui.button("⏭ Step Frame").clicked() {
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::StepFrame));
			}

			if ui.button("▶ Continue").clicked() {
				let _ = self.command_tx.send(EmulatorCommand::TogglePause);
			}
		});

		ui.add_space(10.0);

		// Registradores
		ui.heading("Registers");
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

				if ui.button("➡ Go to PC").clicked() {
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
		let _ = self
			.command_tx
			.send(EmulatorCommand::Debug(DebugCommand::RequestPpuState));

		ui.heading("PPU");
		ui.separator();

		ui.horizontal(|ui| {
			ui.selectable_value(&mut self.current_ppu_tab, PpuTab::Registers, "Registers");
			ui.selectable_value(&mut self.current_ppu_tab, PpuTab::Tiles, "Tiles");
			ui.selectable_value(&mut self.current_ppu_tab, PpuTab::Vram, "VRAM");
			ui.selectable_value(&mut self.current_ppu_tab, PpuTab::Tilemap, "Tilemap");
		});

		ui.separator();

		match self.current_ppu_tab {
			PpuTab::Registers => self.draw_ppu_registers(ui),
			PpuTab::Tiles => self.draw_tiles(ui),
			PpuTab::Vram => self.draw_vram_viewer(ui),
			PpuTab::Tilemap => self.draw_tilemap(ui),
		}
	}

	fn draw_ppu_registers(&mut self, ui: &mut Ui) {
		if let Some(ppu) = &self.ppu_state {
			ScrollArea::vertical().show(ui, |ui| {
				// LCDC
				ui.strong("LCDC (FF40)");
				ui.separator();

				Grid::new("lcdc_grid")
					.num_columns(2)
					.spacing([5.0, 3.0])
					.show(ui, |ui| {
						let flag_color = |set| if set { Color32::GREEN } else { Color32::DARK_GRAY };

						ui.label("LCD Enable:");
						ui.colored_label(flag_color(ppu.lcd_enable), if ppu.lcd_enable { "ON" } else { "OFF" });
						ui.end_row();

						ui.label("BG/Window:");
						ui.colored_label(
							flag_color(ppu.bg_window_enable),
							if ppu.bg_window_enable { "ON" } else { "OFF" },
						);
						ui.end_row();

						ui.label("OBJ Enable:");
						ui.colored_label(flag_color(ppu.obj_enable), if ppu.obj_enable { "ON" } else { "OFF" });
						ui.end_row();

						ui.label("OBJ Size:");
						ui.label(if ppu.obj_size { "8x16" } else { "8x8" });
						ui.end_row();

						ui.label("BG Tile Map:");
						ui.label(if ppu.bg_tile_map { "0x9C00" } else { "0x9800" });
						ui.end_row();

						ui.label("Tile Data:");
						ui.label(if ppu.bg_window_tile_data { "0x8000" } else { "0x8800" });
						ui.end_row();

						ui.label("Window Enable:");
						ui.colored_label(
							flag_color(ppu.window_enable),
							if ppu.window_enable { "ON" } else { "OFF" },
						);
						ui.end_row();

						ui.label("Window Map:");
						ui.label(if ppu.window_tile_map { "0x9C00" } else { "0x9800" });
						ui.end_row();
					});

				ui.add_space(10.0);

				// STAT
				ui.strong("STAT (FF41)");
				ui.separator();

				Grid::new("stat_grid")
					.num_columns(2)
					.spacing([5.0, 3.0])
					.show(ui, |ui| {
						ui.label("Mode:");
						let mode_color = match ppu.mode {
							0 => Color32::GREEN,  // HBlank
							1 => Color32::BLUE,   // VBlank
							2 => Color32::YELLOW, // OAM
							3 => Color32::RED,    // VRAM
							_ => Color32::WHITE,
						};
						ui.colored_label(mode_color, format!("{} ({})", ppu.mode, ppu.current_mode));
						ui.end_row();

						ui.label("LYC=LY:");
						ui.colored_label(
							if ppu.lyc_ly_flag {
								Color32::GREEN
							} else {
								Color32::DARK_GRAY
							},
							if ppu.lyc_ly_flag { "Match" } else { "No Match" },
						);
						ui.end_row();
					});

				ui.add_space(10.0);

				// Scroll & Window
				ui.strong("Scroll & Window");
				ui.separator();

				Grid::new("scroll_grid")
					.num_columns(2)
					.spacing([5.0, 3.0])
					.show(ui, |ui| {
						ui.label("SCY (FF42):");
						ui.monospace(format!("{:02X} ({})", ppu.scy, ppu.scy));
						ui.end_row();

						ui.label("SCX (FF43):");
						ui.monospace(format!("{:02X} ({})", ppu.scx, ppu.scx));
						ui.end_row();

						ui.label("LY (FF44):");
						ui.monospace(format!("{:02X} ({})", ppu.ly, ppu.ly));
						ui.end_row();

						ui.label("LYC (FF45):");
						ui.monospace(format!("{:02X} ({})", ppu.lyc, ppu.lyc));
						ui.end_row();

						ui.label("WY (FF4A):");
						ui.monospace(format!("{:02X} ({})", ppu.wy, ppu.wy));
						ui.end_row();

						ui.label("WX (FF4B):");
						ui.monospace(format!("{:02X} ({})", ppu.wx, ppu.wx));
						ui.end_row();
					});

				ui.add_space(10.0);

				// Palettes
				ui.strong("Palettes");
				ui.separator();

				Grid::new("palette_grid")
					.num_columns(2)
					.spacing([5.0, 3.0])
					.show(ui, |ui| {
						ui.label("BGP (FF47):");
						ui.horizontal(|ui| {
							ui.monospace(format!("{:02X}", ppu.bgp));
							self.draw_palette_preview(ui, ppu.bgp);
						});
						ui.end_row();

						ui.label("OBP0 (FF48):");
						ui.horizontal(|ui| {
							ui.monospace(format!("{:02X}", ppu.obp0));
							self.draw_palette_preview(ui, ppu.obp0);
						});
						ui.end_row();

						ui.label("OBP1 (FF49):");
						ui.horizontal(|ui| {
							ui.monospace(format!("{:02X}", ppu.obp1));
							self.draw_palette_preview(ui, ppu.obp1);
						});
						ui.end_row();
					});

				ui.add_space(10.0);

				// Internal State
				ui.strong("Internal State");
				ui.separator();
				ui.label(format!("Cycles: {}", ppu.cycles));
			});
		} else {
			ui.colored_label(Color32::GRAY, "Waiting for PPU state...");
		}
	}

	fn draw_palette_preview(&self, ui: &mut Ui, palette: u8) {
		for i in 0..4 {
			let color_id = (palette >> (i * 2)) & 0b11;
			let color = self.tile_palette[color_id as usize];
			let (rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
			ui.painter().rect_filled(rect, 0.0, color);
		}
	}

	fn draw_tiles(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.label("Tile Index:");
			ui.add(Slider::new(&mut self.selected_tile, 0..=255));
			ui.monospace(format!("{:02X}", self.selected_tile));

			if ui.button("Load Tile").clicked() {
				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::RequestTileData {
						tile_index: self.selected_tile,
					}));
			}
		});

		ui.separator();

		if let Some(tile) = &self.tile_data {
			ui.label(format!("Tile ${:02X}", tile.tile_index));

			let scale = 16.0;
			let tile_size = 8.0 * scale;
			let (rect, _) = ui.allocate_exact_size(vec2(tile_size, tile_size), Sense::hover());

			let painter = ui.painter();

			for y in 0..8 {
				for x in 0..8 {
					let pixel_idx = y * 8 + x;
					let color_id = tile.pixels[pixel_idx];
					let color = self.tile_palette[color_id as usize];

					let px = rect.min.x + (x as f32 * scale);
					let py = rect.min.y + (y as f32 * scale);
					let pixel_rect = Rect::from_min_size(pos2(px, py), vec2(scale, scale));

					painter.rect_filled(pixel_rect, 0.0, color);
				}
			}

			// Grid lines
			for i in 0..=8 {
				let x = rect.min.x + (i as f32 * scale);
				painter.line_segment(
					[pos2(x, rect.min.y), pos2(x, rect.max.y)],
					Stroke::new(0.5, Color32::from_gray(128)),
				);

				let y = rect.min.y + (i as f32 * scale);
				painter.line_segment(
					[pos2(rect.min.x, y), pos2(rect.max.x, y)],
					Stroke::new(0.5, Color32::from_gray(128)),
				);
			}
		} else {
			ui.colored_label(Color32::GRAY, "Click 'Load Tile' to view");
		}
	}

	fn draw_vram_viewer(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.label("VRAM Tile Viewer");

			if ui.button("Load All Tiles").clicked() {
				self.load_all_vram_tiles();
			}
		});

		ui.separator();

		if !self.vram_tiles_loaded || self.vram_tiles.is_empty() {
			ui.colored_label(Color32::GRAY, "Click 'Load All Tiles' to view VRAM");
			return;
		}

		ScrollArea::both().show(ui, |ui| {
			// Mostrar tiles em grid 16x24 (384 tiles total)
			Grid::new("vram_tiles_grid")
				// .spacing([2.0, 2.0])
				.show(ui, |ui| {
					for row in 0..24 {
						for col in 0..16 {
							let tile_idx = row * 16 + col;

							if tile_idx >= self.vram_tiles.len() {
								break;
							}

							let scale = 4.0;
							let tile_size = 8.0 * scale; // 8x8 pixels dobrados
							let (rect, response) =
								ui.allocate_exact_size(vec2(tile_size, tile_size), Sense::click());

							// Desenhar tile
							let painter = ui.painter();
							let pixels = &self.vram_tiles[tile_idx];

							for y in 0..8 {
								for x in 0..8 {
									let pixel_idx = y * 8 + x;
									let color_id = pixels[pixel_idx];
									let color = self.tile_palette[color_id as usize];

									let px = rect.min.x + (x as f32 * scale);
									let py = rect.min.y + (y as f32 * scale);
									let pixel_rect = Rect::from_min_size(pos2(px, py), vec2(scale, scale));

									painter.rect_filled(pixel_rect, 0.0, color);
								}
							}

							// Border
							// painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(100)));

							if response.clicked() {
								self.selected_tile = tile_idx as u8;
								self.current_ppu_tab = PpuTab::Tiles;
								let _ =
									self
										.command_tx
										.send(EmulatorCommand::Debug(DebugCommand::RequestTileData {
											tile_index: tile_idx as u8,
										}));
							}

							response.on_hover_text(format!("Tile ${:02X}", tile_idx));
						}
						ui.end_row();
					}
				});
		});
	}

	fn load_all_vram_tiles(&mut self) {
		self.vram_tiles.clear();

		// Carregar todos os 384 tiles (256 do set 0 + 128 do set 1)
		for i in 0..384 {
			let _ = self
				.command_tx
				.send(EmulatorCommand::Debug(DebugCommand::RequestTileData {
					tile_index: i as u8,
				}));
		}
	}

	fn draw_tilemap(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.label("Tilemap:");
			ui.radio_value(&mut self.show_tilemap_1, false, "0x9800");
			ui.radio_value(&mut self.show_tilemap_1, true, "0x9C00");

			if ui.button("Load Tilemap").clicked() {
				self.load_all_vram_tiles();

				let _ = self
					.command_tx
					.send(EmulatorCommand::Debug(DebugCommand::RequestTileMap {
						map_select: self.show_tilemap_1,
					}));
			}
		});

		ui.separator();

		if let Some(tilemap) = &self.tilemap_data {
			if self.vram_tiles.is_empty() {
				ui.colored_label(Color32::YELLOW, "Loading tiles...");
				return;
			}

			ScrollArea::both().show(ui, |ui| {
				ui.label(format!("Tilemap at ${}", if tilemap.map_select { "9C00" } else { "9800" }));

				// Renderizar tilemap graficamente (32x32 tiles = 256x256 pixels)
				let tilemap_size = 32.0 * 16.0; // 32 tiles * 16 pixels por tile (8x8 dobrado)
				let (rect, _) = ui.allocate_exact_size(vec2(tilemap_size, tilemap_size), Sense::hover());

				let painter = ui.painter();

				for row in 0..32 {
					for col in 0..32 {
						let idx = row * 32 + col;
						let tile_id = tilemap.tiles[idx] as usize;

						if tile_id >= self.vram_tiles.len() {
							continue;
						}

						let pixels = &self.vram_tiles[tile_id];

						let base_x = rect.min.x + (col as f32 * 16.0);
						let base_y = rect.min.y + (row as f32 * 16.0);

						for y in 0..8 {
							for x in 0..8 {
								let pixel_idx = y * 8 + x;
								let color_id = pixels[pixel_idx];
								let color = self.tile_palette[color_id as usize];

								let px = base_x + (x as f32 * 2.0);
								let py = base_y + (y as f32 * 2.0);
								let pixel_rect = Rect::from_min_size(pos2(px, py), vec2(2.0, 2.0));

								painter.rect_filled(pixel_rect, 0.0, color);
							}
						}
					}
				}

				// painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(128)));
			});
		} else {
			ui.colored_label(Color32::GRAY, "Click 'Load Tilemap' to view");
		}
	}

	fn draw_bottom_panel(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.selectable_value(&mut self.show_bottom_tab(), BottomTab::Memory, "Memory");
			ui.selectable_value(&mut self.show_bottom_tab(), BottomTab::Trace, "Trace");
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

			DebugEvent::PpuState(state) => {
				self.ppu_state = Some(state);
			}

			DebugEvent::TileData(data) => {
				let tile_idx = data.tile_index as usize;

				while self.vram_tiles.len() <= tile_idx {
					self.vram_tiles.push(Vec::new());
				}
				self.vram_tiles[tile_idx] = data.pixels.clone();

				if tile_idx >= 255 {
					self.vram_tiles_loaded = true;
				}

				self.tile_data = Some(data);
			}

			DebugEvent::TileMapData(data) => {
				self.tilemap_data = Some(data);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PpuTab {
	Registers,
	Tiles,
	Tilemap,
	Vram,
}
