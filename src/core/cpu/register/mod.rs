#[derive(Debug, Clone, Copy)]
pub enum Register8 {
	A,
	B,
	C,
	D,
	E,
	H,
	L,
}

#[derive(Clone, Copy)]
pub enum Register16 {
	AF,
	BC,
	DE,
	HL,
	SP,
}

pub enum Flags {
	Z, // Zero flag
	N, // Subtraction flag (BCD)
	H, // Half Carry flag (BCD) HC = (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
	C, // Carry flag
}

#[derive(Default, Debug, Clone)]
pub struct Register {
	pub a: u8,
	pub b: u8,
	pub c: u8,
	pub d: u8,
	pub e: u8,
	pub f: u8,
	pub h: u8,
	pub l: u8,
	pub pc: u16,
	pub sp: u16,
	pub ime: u8,
	pub ie: u8,
}

impl Register {
	pub fn new() -> Register {
		Register::default()
	}

	pub fn get_r8(&self, reg: Register8) -> u8 {
		match reg {
			Register8::A => self.a,
			Register8::B => self.b,
			Register8::C => self.c,
			Register8::D => self.d,
			Register8::E => self.e,
			Register8::H => self.h,
			Register8::L => self.l,
		}
	}

	pub fn set_r8(&mut self, reg: Register8, value: u8) {
		match reg {
			Register8::A => self.a = value,
			Register8::B => self.b = value,
			Register8::C => self.c = value,
			Register8::D => self.d = value,
			Register8::E => self.e = value,
			Register8::H => self.h = value,
			Register8::L => self.l = value,
		}
	}

	pub fn get_r16(&self, reg: Register16) -> u16 {
		match reg {
			Register16::AF => ((self.a as u16) << 8) | (self.f as u16),
			Register16::BC => ((self.b as u16) << 8) | (self.c as u16),
			Register16::DE => ((self.d as u16) << 8) | (self.e as u16),
			Register16::HL => ((self.h as u16) << 8) | (self.l as u16),
			Register16::SP => self.sp,
		}
	}

	pub fn set_r16(&mut self, reg: Register16, value: u16) {
		let hi = (value >> 8) as u8;
		let lo = (value & 0x00FF) as u8;

		match reg {
			Register16::AF => {
				self.a = hi;
				self.f = lo & 0xF0;
			}
			Register16::BC => {
				self.b = hi;
				self.c = lo;
			}
			Register16::DE => {
				self.d = hi;
				self.e = lo;
			}
			Register16::HL => {
				self.h = hi;
				self.l = lo;
			}
			Register16::SP => {
				self.sp = value;
			}
		}
	}

	pub fn get_flag(&self, flag: Flags) -> u8 {
		match flag {
			Flags::Z => (self.f >> 7) & 0b1,
			Flags::N => (self.f >> 6) & 0b1,
			Flags::H => (self.f >> 5) & 0b1,
			Flags::C => (self.f >> 4) & 0b1,
		}
	}

	pub fn set_flag(&mut self, flag: Flags, conditional: bool) {
		let bit = match flag {
			Flags::Z => 7,
			Flags::N => 6,
			Flags::H => 5,
			Flags::C => 4,
		};

		self.f = if conditional {
			self.f | (1 << bit) // Seta o bit
		} else {
			self.f & !(1 << bit) // Limpa o bit
		};
	}
}
