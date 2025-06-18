#![allow(dead_code)]

pub struct Dimensions {
	pub width: u32,
	pub height: u32,
}

impl Dimensions {
	pub const fn new(width: u32, height: u32) -> Self {
		Self { width, height }
	}

	pub const fn scale(&self, factor: u32) -> Self {
		Self {
			width: self.width * factor,
			height: self.height * factor,
		}
	}
}

pub const SCREEN_SCALE: u32 = 4;
pub const GAMEBOY_RESOLUTION: Dimensions = Dimensions::new(160, 144);
pub const SCREEN_SIZE: Dimensions = GAMEBOY_RESOLUTION.scale(SCREEN_SCALE);
