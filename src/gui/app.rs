use crate::gui::common::dialog;

use crate::emuchan::{EmuChan, EmulationState};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct EmuChanGui {
	emulator: Arc<Mutex<EmuChan>>,

	emulator_texture: Option<egui::TextureHandle>,
	displayed_title: String,

	rom_path_sender: Sender<PathBuf>,
	rom_path_receiver: Receiver<PathBuf>,

}

impl EmuChanGui {
	pub fn new(emulator: Arc<Mutex<EmuChan>>) -> Self {
		let (sender, receiver) = channel();

		Self {
			emulator,
			emulator_texture: None,
			displayed_title: "EmuChan".to_string(),
			rom_path_sender: sender,
			rom_path_receiver: receiver,
		}
	}

	fn update_emulator_texture(&mut self, ctx: &egui::Context) {
		const GAMEBOY_PALETTE: [egui::Color32; 4] = [
			egui::Color32::from_rgb(155, 188, 15), // Lightest Green
			egui::Color32::from_rgb(139, 172, 15), // Light Green
			egui::Color32::from_rgb(48, 98, 48),   // Dark Green
			egui::Color32::from_rgb(15, 56, 15),   // Darkest Green
		];

		let emulator = self.emulator.lock().unwrap();

		let video_buffer = emulator.get_video_buffer();

		let color_buffer: Vec<egui::Color32> = video_buffer
			.iter()
			.map(|&pixel_index| GAMEBOY_PALETTE[pixel_index as usize])
			.collect();

		let rgba_buffer: Vec<u8> = color_buffer.iter().flat_map(|c| c.to_array()).collect();

		let image = egui::ColorImage::from_rgba_unmultiplied([160, 144], &rgba_buffer);

		// let texture = ctx.load_texture("emulator_screen", image, egui::TextureOptions::NEAREST);
		// self.emulator_texture = Some(texture);

		if let Some(texture) = &mut self.emulator_texture {
			texture.set(image, egui::TextureOptions::NEAREST);
		} else {
			self.emulator_texture =
				Some(ctx.load_texture("emulator_screen", image, egui::TextureOptions::NEAREST));
		}
	}
}

impl eframe::App for EmuChanGui {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		if let Ok(path) = self.rom_path_receiver.try_recv() {
			println!("Selected ROM: {}", path.display());

			let mut emulator = self.emulator.lock().unwrap();
			emulator.load_rom(path.to_string_lossy().to_string());

			*emulator.emulation_state.lock().unwrap() = EmulationState::RUNNING;
		}

		self.emulator.lock().unwrap().run_one_frame();

		self.update_emulator_texture(ctx);

		let new_title = {
			let emulator = self.emulator.lock().unwrap();
			let game_title = emulator.get_game_title();

			let clean_title = game_title.trim_matches(char::from(0));

			if clean_title.is_empty() {
				"EmuChan".to_string()
			} else {
				format!("EmuChan - {}", clean_title)
			}
		};

		if self.displayed_title != new_title {
			self.displayed_title = new_title.clone();
			ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.displayed_title.clone()));
		}

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
			});
		});

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

		egui::CentralPanel::default().show(ctx, |ui| {
			egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
				if self.emulator_texture.is_none() {
					self.update_emulator_texture(ctx);
				}

				if let Some(texture) = &self.emulator_texture {
					let desired_size = egui::vec2(160.0 * 4.0, 144.0 * 4.0);

					let emu_screen_image = egui::Image::new(texture);

					let sized_image = emu_screen_image.fit_to_exact_size(desired_size);

					ui.add(sized_image);
				}
			});
		});

		ctx.request_repaint();
	}
}

