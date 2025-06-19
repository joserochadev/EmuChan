#[derive(Debug)]
pub struct LCDC(u8);

impl LCDC {
	pub const BG_ON: u8 = 0b0000_0001;
	pub const OBJ_ON: u8 = 0b0000_0010;
	pub const OBJ_SIZE: u8 = 0b0000_0100;
	pub const BG_MAP: u8 = 0b0000_1000;
	pub const BG_ADDR: u8 = 0b0001_0000;
	pub const WINDOW_ON: u8 = 0b0010_0000;
	pub const WINDOW_MAP: u8 = 0b0100_0000;
	pub const LCD_ON: u8 = 0b1000_0000;

	pub fn new(value: u8) -> Self {
		Self(value)
	}

	pub fn set_lcdc(&mut self, value: u8) {
		self.0 = value;
	}

	pub fn get_lcdc(&self) -> u8 {
		self.0
	}

	pub fn clear(&mut self, flag: u8) {
		self.0 &= !flag;
	}

	pub fn set(&mut self, flag: u8) {
		self.0 |= flag;
	}

	pub fn is_set(&self, flag: u8) -> bool {
		self.0 & flag != 0
	}
}
