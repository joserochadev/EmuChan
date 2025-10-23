use crate::emuchan::{EmuChan, EmulationState};
use crate::tests::sm83::SM83;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::gui::common::dialog::{self, open_json_test_dialog};
use crate::gui::common::palettes::{self, ColorPalette};
use crate::gui::common::window_scale::WindowScale;

pub struct EmuChanGui {
	emulator: Arc<Mutex<EmuChan>>,
	emulator_texture: Option<egui::TextureHandle>,
	displayed_title: String,
	rom_path_sender: Sender<PathBuf>,
	rom_path_receiver: Receiver<PathBuf>,
	selected_palette: ColorPalette,
	window_scale: WindowScale,
	test_result_sender: Sender<String>,
	test_result_receiver: Receiver<String>,
	test_log: Vec<String>,
	show_test_runner_window: bool,
}

impl eframe::App for EmuChanGui {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.handle_logic(ctx);

		self.ui_top_painel(ctx);
		self.ui_central_painel(ctx);
		self.ui_bottom_painel(ctx);

		ctx.request_repaint();
	}
}

impl EmuChanGui {
	pub fn new(emulator: Arc<Mutex<EmuChan>>) -> Self {
		let (sender, receiver) = channel();
		let (test_sender, test_receiver) = channel();

		Self {
			emulator,
			emulator_texture: None,
			displayed_title: "EmuChan".to_string(),
			rom_path_sender: sender,
			rom_path_receiver: receiver,
			selected_palette: ColorPalette::Classic,
			window_scale: WindowScale::X4,
			test_result_sender: test_sender,
			test_result_receiver: test_receiver,
			test_log: Vec::new(),
			show_test_runner_window: false,
		}
	}

	fn update_emulator_texture(&mut self, ctx: &egui::Context) {
		let pallete = palettes::get_colors(self.selected_palette);

		let emulator = self.emulator.lock().unwrap();

		let video_buffer = emulator.get_video_buffer();

		let color_buffer: Vec<egui::Color32> = video_buffer
			.iter()
			.map(|&pixel_index| pallete[pixel_index as usize])
			.collect();

		let rgba_buffer: Vec<u8> = color_buffer.iter().flat_map(|c| c.to_array()).collect();
		let image = egui::ColorImage::from_rgba_unmultiplied([160, 144], &rgba_buffer);

		if let Some(texture) = &mut self.emulator_texture {
			texture.set(image, egui::TextureOptions::NEAREST);
		} else {
			self.emulator_texture =
				Some(ctx.load_texture("emulator_screen", image, egui::TextureOptions::NEAREST));
		}
	}

	fn handle_logic(&mut self, ctx: &egui::Context) {
		if let Ok(log_message) = self.test_result_receiver.try_recv() {
			self.test_log.push(log_message);
		}

		if let Ok(path) = self.rom_path_receiver.try_recv() {
			println!("Selected ROM: {}", path.display());

			let mut emulator = self.emulator.lock().unwrap();
			emulator.load_rom(path.to_string_lossy().to_string());
			*emulator.emulation_state.lock().unwrap() = EmulationState::RUNNING;
		}

		self.emulator.lock().unwrap().run_one_frame();
		self.update_emulator_texture(ctx);

		self.update_window_title(ctx);

		if self.show_test_runner_window {
			self.ui_test_runner_window(ctx);
		}
	}

	fn ui_top_painel(&mut self, ctx: &egui::Context) {
		egui::TopBottomPanel::top("top_painel").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("load rom...").clicked() {
						dialog::open_rom_dialog(self.rom_path_sender.clone());
						ui.close_menu();
					}

					ui.separator();

					if ui.button("Quit").clicked() {
						ctx.send_viewport_cmd(egui::ViewportCommand::Close);
					}
				});

				ui.menu_button("Emulation", |ui| {
					ui.menu_button("Video", |ui| {
						ui.menu_button("Color Pallete", |ui| {
							ui.radio_value(&mut self.selected_palette, ColorPalette::Default, "Default");
							ui.radio_value(&mut self.selected_palette, ColorPalette::Classic, "Classic Green");
							ui.radio_value(&mut self.selected_palette, ColorPalette::Greyscale, "Greyscale");
							ui.radio_value(&mut self.selected_palette, ColorPalette::Chocolate, "Chocolate");
						});
						ui.separator();
						ui.menu_button("Resolution", |ui| {
							ui.radio_value(&mut self.window_scale, WindowScale::X1, "1x (160 x 144)");
							ui.radio_value(&mut self.window_scale, WindowScale::X2, "2x (320 x 288)");
							ui.radio_value(&mut self.window_scale, WindowScale::X3, "3x (480 x 432)");
							ui.radio_value(&mut self.window_scale, WindowScale::X4, "4x (640 x 576)");
						});
					});
				});

				ui.menu_button("Developer", |ui| {
					if ui.button("SM83 Test").clicked() {
						self.show_test_runner_window = true;
						ui.close_menu();
					}
				});
			});
		});
	}

	fn ui_central_painel(&mut self, ctx: &egui::Context) {
		let frame = egui::Frame {
			inner_margin: egui::Margin::same(0),
			fill: egui::Color32::TRANSPARENT,
			..Default::default()
		};

		egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
			if self.emulator_texture.is_none() {
				self.update_emulator_texture(ctx);
			}

			ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
				if let Some(texture) = &self.emulator_texture {
					let scale_factor = self.window_scale.as_factor();
					let available_size = ui.available_size();
					let scaled_width = 160.0 * scale_factor;
					let scaled_height = 144.0 * scale_factor;

					let final_size =
						egui::vec2(scaled_width.min(available_size.x), scaled_height.min(available_size.y));

					let image = egui::Image::new(texture).fit_to_exact_size(final_size);
					ui.add(image);
				}
			});
		});
	}

	fn ui_bottom_painel(&mut self, ctx: &egui::Context) {
		egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				let emulator = self.emulator.lock().unwrap();
				let emu_speed = emulator.emu_speed_percent;
				let emu_fps = emulator.emu_fps;

				ui.label(format!("Speed: {:.0}%", emu_speed));
				ui.separator();
				ui.label(format!("FPS: {:.1}", emu_fps));
			});
		});
	}

	fn update_window_title(&mut self, ctx: &egui::Context) {
		let new_title = {
			let emulator = self.emulator.lock().unwrap();
			let game_title = emulator
				.get_game_title()
				.trim_matches(char::from(0))
				.to_string();

			if game_title.is_empty() {
				"EmuChan".to_string()
			} else {
				format!("EmuChan - {}", game_title)
			}
		};

		if self.displayed_title != new_title {
			self.displayed_title = new_title.clone();
			ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.displayed_title.clone()));
		}
	}

	fn ui_test_runner_window(&mut self, ctx: &egui::Context) {
		egui::Window::new("SM83 CPU Tests")
			.open(&mut self.show_test_runner_window)
			.show(ctx, |ui| {
				if ui.button("Select Json Test").clicked() {
					// Dispara a lógica para abrir o diálogo de arquivo
					open_json_test_dialog(self.test_result_sender.clone());
				}

				ui.separator(); // Linha divisória

				// Área de scroll para exibir os resultados do teste
				ui.heading("Test Results:");
				egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
					ui.set_min_size(egui::vec2(300.0, 100.0));
					egui::ScrollArea::vertical()
					.stick_to_bottom(true) // Faz o scroll ir para o final automaticamente
					.show(ui, |ui| {
						ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
							for message in &self.test_log {
									// Usamos um `monospace` para um visual de log mais tradicional
								ui.label(egui::RichText::new(message).monospace());
							}
						});
					});
				});
			});
	}
}
