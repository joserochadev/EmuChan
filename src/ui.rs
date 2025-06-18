#![warn(dead_code)]
use std::sync::{Arc, Mutex};

use crate::config::SCREEN_SIZE;
use crate::ppu::PPU;
use sdl2::{event::Event, pixels::Color, rect::Rect};

const COLOR_PALLET: [Color; 4] = [
	Color::RGBA(223, 248, 208, 1), // light green
	Color::RGBA(136, 192, 112, 1), // moderate green
	Color::RGBA(54, 103, 82, 1),   // dark green
	Color::RGBA(7, 24, 33, 1),     // very dark green
];

pub struct UI {
	_context: sdl2::Sdl,
	pub main_window: sdl2::video::Window,
	_debug_window: sdl2::video::Window,
	main_canvas: sdl2::render::Canvas<sdl2::video::Window>,
	debug_canvas: sdl2::render::Canvas<sdl2::video::Window>,
	event_pump: sdl2::EventPump,
	ppu: Arc<Mutex<PPU>>,
	scale: i32,
}

impl UI {
	pub fn new(ppu: Arc<Mutex<PPU>>) -> Self {
		let context = sdl2::init().expect("Failed to init SDL2 Context");
		let video_subsystem = context
			.video()
			.expect("Failed to init SDL2 video subsystem");
		let event_pump = context.event_pump().expect("Failed to init Event Pump");

		let display_bounds = video_subsystem.display_bounds(0).unwrap();
		let screen_width = display_bounds.width();
		let screen_height = display_bounds.height();

		let main_width = SCREEN_SIZE.width;
		let main_height = SCREEN_SIZE.height;
		let debug_width = 16 * 8 * 3;
		let debug_height = main_height;

		let main_x = ((screen_width - main_width - debug_width) / 2) as u32;
		let main_y = ((screen_height - main_height) / 2) as u32;
		let debug_x = main_x + main_width + 10;
		let debug_y = main_y;

		let main_window = video_subsystem
			.window("EmuChan", main_width, main_height)
			.position(main_x as i32, main_y as i32)
			.opengl()
			.build()
			.expect("Failed to init Emuchan Window.");

		let debug_window = video_subsystem
			.window("Debug - VRAM Tiles", debug_width, debug_height)
			.position(debug_x as i32, debug_y as i32)
			.opengl()
			.build()
			.expect("Failed to init Debug Window.");

		let main_canvas = main_window
			.clone()
			.into_canvas()
			.present_vsync()
			.build()
			.expect("Failed to create main canvas");
		let debug_canvas = debug_window
			.clone()
			.into_canvas()
			.present_vsync()
			.build()
			.expect("Failed to create debug canvas");

		Self {
			_context: context,
			main_window,
			_debug_window: debug_window,
			main_canvas,
			debug_canvas,
			event_pump,
			ppu,
			scale: 3,
		}
	}

	pub fn run(&mut self) {
		'running: loop {
			for event in self.event_pump.poll_iter() {
				match event {
					Event::Quit { .. }
					| Event::Window {
						win_event: sdl2::event::WindowEvent::Close,
						..
					}
					| Event::KeyDown {
						keycode: Some(sdl2::keyboard::Keycode::Escape),
						..
					} => {
						break 'running;
					}
					_ => {}
				}
			}

			// self.main_canvas.set_draw_color(Color::RGB(0, 0, 0));
			// self.main_canvas.clear();
			// self.main_canvas.present();

			self.update_main_window();
			self.main_canvas.present();

			// self.debug_canvas.set_draw_color(Color::RGB(50, 50, 50));
			// self.debug_canvas.clear();

			// self.update_debug_window();
			// self.debug_canvas.present();
		}
	}

	fn display_tile(&mut self, start_location: u16, tile_num: u16, x: i32, y: i32) {
		let ppu = self.ppu.lock().unwrap();

		let mut pixels = [[0; 8]; 8];

		// read 1 tile of 16 bytes (2 bytes by row, 8 rows)
		for tile in 0..8 {
			let byte_1 = ppu.read(start_location + (tile_num * 16) + tile * 2);
			let byte_2 = ppu.read(start_location + (tile_num * 16) + tile * 2 + 1);

			// fill row of pixels, from left to right
			for bit in 0..8 {
				let lo = (byte_1 >> (7 - bit)) & 1;
				let hi = (byte_2 >> (7 - bit)) & 1;

				let color = (hi << 1) | lo;

				pixels[tile as usize][bit] = color;
			}
		}

		// draw tile on screen
		for row in 0..8 {
			for col in 0..8 {
				let pixel = pixels[row][col];

				let pixel_x = x + (col as i32 * self.scale);
				let pixel_y = y + (row as i32 * self.scale);

				let width = self.scale;
				let height = self.scale;

				self
					.debug_canvas
					.set_draw_color(COLOR_PALLET[pixel as usize]);
				self
					.debug_canvas
					.fill_rect(Rect::new(pixel_x, pixel_y, width as u32, height as u32))
					.unwrap();
			}
		}
	}

	fn update_debug_window(&mut self) {
		let mut tile_num = 0;
		let addr = 0x8000;
		let spacing = 2;

		// 384 tiles, 24 X 16

		for row in 0..24 {
			for col in 0..16 {
				let x_draw = col * (8 * self.scale + spacing);
				let y_draw = row * (8 * self.scale + spacing);
				self.display_tile(addr, tile_num, x_draw, y_draw);
				tile_num += 1;
			}
		}
	}

	fn update_main_window(&mut self) {
		let ppu = self.ppu.lock().unwrap();

		let screen_width = 160 as i32;
		let screen_height = 144 as i32;

		for y in 0..screen_height {
			for x in 0..screen_width {
				let pixel_index = (y as usize * screen_width as usize) + x as usize;
				let color = COLOR_PALLET[ppu.video_buffer[pixel_index] as usize];

				let pixel_x = x * 4;
				let pixel_y = y * 4;

				let width = 4;
				let height = 4;

				self.main_canvas.set_draw_color(color);
				self
					.main_canvas
					.fill_rect(Rect::new(pixel_x, pixel_y, width as u32, height as u32))
					.unwrap();
			}
		}
	}
}
