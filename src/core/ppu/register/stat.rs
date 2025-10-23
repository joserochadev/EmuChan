#[derive(Debug)]
pub struct STAT(u8);

impl STAT {
	pub const PPU_MODE: u8 = 0b0000_0011;
	pub const LYC_EQ_LY: u8 = 0b0000_0100;
	pub const HBLANK_INT: u8 = 0b0000_1000; // Mode 0 interupt
	pub const VBLANK_INT: u8 = 0b0001_0000; // Mode 1 interupt
	pub const OAM_INT: u8 = 0b0010_0000; // Mode 2 interupt
	pub const LYC_INT: u8 = 0b0100_0000;

	pub fn new(value: u8) -> Self {
		Self(value)
	}

	pub fn set_stat(&mut self, value: u8) {
		self.0 = value;
	}

	pub fn get_stat(&self) -> u8 {
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
