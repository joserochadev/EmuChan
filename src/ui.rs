#![warn(dead_code)]
use crate::utils::config::SCREEN_SIZE;
use sdl2::{event::Event, pixels::Color, rect::Rect};

pub struct UI {
	_context: sdl2::Sdl,
	_main_window: sdl2::video::Window,
	_debug_window: sdl2::video::Window,
	main_canvas: sdl2::render::Canvas<sdl2::video::Window>,
	debug_canvas: sdl2::render::Canvas<sdl2::video::Window>,
	event_pump: sdl2::EventPump,
}

impl UI {
	pub fn new() -> Self {
		let context = sdl2::init().expect("Failed to init SDL2 Context");
		let video_subsystem = context
			.video()
			.expect("Failed to init SDL2 video subsystem");
		let event_pump = context.event_pump().expect("Failed to init Event Pump");

		let display_bounds = video_subsystem.display_bounds(0).unwrap();
		let screen_width = display_bounds.width();
		let screen_height = display_bounds.height();

		let main_width = SCREEN_SIZE.0 as u32;
		let main_height = SCREEN_SIZE.1 as u32;
		let debug_width = 256;
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
			_main_window: main_window,
			_debug_window: debug_window,
			main_canvas,
			debug_canvas,
			event_pump,
		}
	}

	pub fn run(&mut self) {
		'runnign: loop {
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
						break 'runnign;
					}
					_ => {}
				}
			}

			self.main_canvas.set_draw_color(Color::RGB(0, 0, 0));
			self.main_canvas.clear();
			self.main_canvas.present();

			self.debug_canvas.set_draw_color(Color::RGB(50, 50, 50));
			self.debug_canvas.clear();

			self.debug_canvas.set_draw_color(Color::RGB(200, 200, 200));
			self
				.debug_canvas
				.fill_rect(Rect::new(10, 10, 32, 32))
				.unwrap();
			self.debug_canvas.present();
		}
	}
}
