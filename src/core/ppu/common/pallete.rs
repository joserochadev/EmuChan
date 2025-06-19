#[derive(Debug)]
pub struct Pallete(u8);

impl Pallete {
	pub fn new(value: u8) -> Self {
		Self(value)
	}

	pub fn get_pallete(&self) -> u8 {
		self.0
	}

	pub fn set_pallete(&mut self, value: u8) {
		self.0 = value;
	}

	pub fn extract_pallete(&self) -> [u8; 4] {
		let mut colors = [0; 4];
		for i in 0..4 {
			colors[i] = (self.0 >> (i * 2)) & 0b11;
		}

		return colors;
	}
}
