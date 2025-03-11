#![allow(dead_code)]
use std::fmt;

use crate::bus::BUS;

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

	pub fn get_bc(&self) -> u16 {
		let register = ((self.b as u16) << 8) | self.c as u16;
		register
	}

	pub fn set_bc(&mut self, data: u16) {
		let hi = (data >> 8) & 0xFF;
		let lo = data & 0xFF;
		self.b = hi as u8;
		self.c = lo as u8;
	}
	pub fn get_de(&self) -> u16 {
		let register = ((self.d as u16) << 8) | self.e as u16;
		register
	}

	pub fn set_de(&mut self, data: u16) {
		let hi = (data >> 8) & 0xFF;
		let lo = data & 0xFF;
		self.d = hi as u8;
		self.e = lo as u8;
	}
	pub fn get_hl(&self) -> u16 {
		let register = ((self.h as u16) << 8) | self.l as u16;
		register
	}

	pub fn set_hl(&mut self, data: u16) {
		let hi = (data >> 8) & 0xFF;
		let lo = data & 0xFF;
		self.h = hi as u8;
		self.l = lo as u8;
	}
}

#[derive(Debug)]
pub struct CPU {
	pub reg: Register,
	bus: *mut BUS,
	pub cycles: usize,
}

impl CPU {
	pub fn new(bus: &mut BUS) -> CPU {
		CPU {
			reg: Register::new(),
			bus,
			cycles: 0,
		}
	}

	// pub fn bus_connect(&mut self, bus: *mut BUS) {
	//   self.bus = bus
	// }

	pub fn read(&mut self, addr: u16) -> u8 {
		// let bus = self.bus.lock().unwrap();
		// bus.read(addr)
		unsafe { (*self.bus).read(addr) }
	}

	pub fn write(&mut self, addr: u16, data: u8) {
		// let mut bus = self.bus.lock().unwrap();
		// bus.write(addr, data);
		unsafe {
			(*self.bus).write(addr, data);
		}
	}

	pub fn view_memory_at(&self, memory: &[u8], address: usize, n: usize) {
		// Garantimos que não tentaremos acessar fora do limite de memória
		let end_address = (address + n).min(memory.len());

		let next_n_bytes: Vec<String> = memory[address..end_address]
			.iter() // Itera sobre a slice da memória
			.map(|&v| format!("0x{:02x}", v)) // Converte cada byte para o formato hexadecimal
			.collect(); // Coleta os resultados como uma `Vec<String>`

		println!(
			"0x{:04x}: {}",
			address,
			next_n_bytes.join(" ") // Junta os bytes em uma string separada por espaços
		);
	}

	pub fn get_flag(&self, flag: Flags) -> u8 {
		match flag {
			Flags::Z => (self.reg.f >> 7) & 0b1,
			Flags::N => (self.reg.f >> 6) & 0b1,
			Flags::H => (self.reg.f >> 5) & 0b1,
			Flags::C => (self.reg.f >> 4) & 0b1,
		}
	}

	pub fn set_flag(&mut self, flag: Flags, conditional: bool) {
		let bit = match flag {
			Flags::Z => 7,
			Flags::N => 6,
			Flags::H => 5,
			Flags::C => 4,
		};

		self.reg.f = if conditional {
			self.reg.f | (1 << bit) // Seta o bit
		} else {
			self.reg.f & !(1 << bit) // Limpa o bit
		};
	}

	pub fn set_cycles(&mut self, t_cycles: usize) {
		self.cycles = t_cycles;
	}

	pub fn fetch(&mut self) -> u8 {
		let data = self.read(self.reg.pc);
		self.reg.pc = self.reg.pc.wrapping_add(1);

		data
	}

	pub fn fetch16(&mut self) -> u16 {
		let lo = self.read(self.reg.pc);
		let hi = self.read(self.reg.pc + 1);
		self.reg.pc = self.reg.pc.wrapping_add(2);
		let data = ((hi as u16) << 8) | lo as u16;
		data
	}

	pub fn push(&mut self, data: u16) {
		let hi = (data >> 8) & 0xFF;
		let lo = data & 0xFF;

		self.reg.sp = self.reg.sp.wrapping_sub(1);
		self.write(self.reg.sp, hi as u8);

		self.reg.sp = self.reg.sp.wrapping_sub(1);
		self.write(self.reg.sp, lo as u8);
	}

	pub fn pop(&mut self) -> u16 {
		let lo = self.read(self.reg.sp) as u16;
		self.reg.sp = self.reg.sp.wrapping_add(1);

		let hi = self.read(self.reg.sp) as u16;
		self.reg.sp = self.reg.sp.wrapping_add(1);

		let data = (hi << 8) | lo;
		data
	}

	pub fn decode(&mut self, instruction: u8) -> Result<(), String> {
		match instruction {
			0x00 => {
				self.set_cycles(4);
			}
			0x05 => {
				let before = self.reg.b;
				let result = self.reg.b.wrapping_sub(1);
				self.reg.b = result;

				let hc = (before & 0xF) < 1;

				self.set_flag(Flags::Z, result == 0);
				self.set_flag(Flags::N, true);
				self.set_flag(Flags::H, hc);
				self.set_cycles(4);
			}
			0x06 => {
				let data = self.fetch();
				self.reg.b = data;
				self.set_cycles(8);
			}
			0x0C => {
				let before = self.reg.c;
				let result = self.reg.c.wrapping_add(1);
				self.reg.c = result;
				let hc = (before & 0xF) + (1 & 0xF);

				self.set_flag(Flags::Z, result == 0);
				self.set_flag(Flags::N, false);
				self.set_flag(Flags::H, hc > 0xF);
				self.set_cycles(4);
			}
			0x0E => {
				let data = self.fetch();
				self.reg.c = data;
				self.set_cycles(8);
			}
			0x11 => {
				let data = self.fetch16();
				self.reg.set_de(data);
				self.set_cycles(12);
			}
			0x13 => {
				let de = self.reg.get_de();
				self.reg.set_de(de + 1);
				self.set_cycles(8);
			}
			0x17 => {
				let carry = self.get_flag(Flags::C);
				let bit_7 = (self.reg.a >> 7) & 0b1;

				self.reg.a = (self.reg.a << 1) | carry;

				self.set_flag(Flags::Z, false);
				self.set_flag(Flags::N, false);
				self.set_flag(Flags::H, false);
				self.set_flag(Flags::C, bit_7 == 1);

				self.set_cycles(4);
			}
			0x1A => {
				let addr = self.reg.get_de();
				let data = self.read(addr);
				self.reg.a = data;
				self.set_cycles(8);
			}
			0x20 => {
				let data = self.fetch() as i8;
				if self.get_flag(Flags::Z) == 1 {
					self.reg.pc = self.reg.pc.wrapping_add(data as u16);
					self.set_cycles(12);
				}
				self.set_cycles(8);
			}
			0x21 => {
				let data = self.fetch16();
				self.reg.set_hl(data);
				self.set_cycles(12);
			}
			0x22 => {
				let data = self.reg.a;
				let addr = self.reg.get_hl();
				self.write(addr, data);
				self.reg.set_hl(addr + 1);
				self.set_cycles(8);
			}
			0x23 => {
				let hl = self.reg.get_hl();
				self.reg.set_hl(hl + 1);
				self.set_cycles(8);
			}
			0x31 => {
				let data = self.fetch16();
				self.reg.sp = data;
				self.set_cycles(12);
			}
			0x32 => {
				let addr = self.reg.get_hl();
				let data = self.reg.a;
				self.write(addr, data);

				self.reg.set_hl(addr - 1);
				self.set_cycles(8);
			}
			0x3E => {
				let data = self.fetch();
				self.reg.a = data;
				self.set_cycles(8);
			}
			0x4F => {
				let data = self.reg.a;
				self.reg.c = data;
				self.set_cycles(4);
			}
			0x77 => {
				let data = self.reg.a;
				let addr = self.reg.get_hl();
				self.write(addr, data);
				self.set_cycles(8);
			}
			0x7B => {
				let e = self.reg.e;
				self.reg.a = e;
				self.set_cycles(4);
			}
			0xAF => {
				let result = self.reg.a ^ self.reg.a;
				self.reg.a = result;

				self.set_flag(Flags::Z, result == 0);
				self.set_flag(Flags::N, false);
				self.set_flag(Flags::H, false);
				self.set_flag(Flags::C, false);
				self.set_cycles(4);
			}
			0xC1 => {
				let data = self.pop();
				self.reg.set_bc(data);
				self.set_cycles(12);
			}
			0xC5 => {
				let data = self.reg.get_bc();
				self.push(data);
				self.set_cycles(16);
			}
			0xC9 => {
				let addr = self.pop();
				self.reg.pc = addr;
				self.set_cycles(16);
			}
			0xCB => {
				let cb_intruction = self.fetch();

				match cb_intruction {
					0x11 => {
						let carry = self.get_flag(Flags::C);
						let bit_7 = (self.reg.c >> 7) & 0b1;

						self.reg.c = (self.reg.c << 1) | carry;

						self.set_flag(Flags::Z, self.reg.c == 0);
						self.set_flag(Flags::N, false);
						self.set_flag(Flags::H, false);
						self.set_flag(Flags::C, bit_7 == 1);

						self.set_cycles(8);
					}
					0x7C => {
						let bit_7_h = (self.reg.h >> 7) & 0b1;

						self.set_flag(Flags::Z, bit_7_h == 1);
						self.set_flag(Flags::N, false);
						self.set_flag(Flags::H, true);
						self.set_cycles(8);
					}
					_ => return Err(format!("Unknow CB instruction. OPCODE: {:02X}", cb_intruction)),
				}
			}
			0xCD => {
				let nn = self.fetch16();
				self.push(self.reg.pc);
				self.reg.pc = nn;
				self.set_cycles(24);
			}
			0xE0 => {
				let hi = (0xFF << 8) as u16;
				let lo = self.fetch() as u16;
				let addr = hi | lo;
				let data = self.reg.a;
				self.write(addr, data);
				self.set_cycles(12);
			}
			0xE2 => {
				let hi = (0xFF << 8) as u16;
				let lo = self.reg.c as u16;
				let addr = hi | lo;
				self.write(addr, self.reg.a);
				self.set_cycles(8);
			}
			0xEA => {
				let addr = self.fetch16();
				self.write(addr, self.reg.a);
				self.set_cycles(16);
			}
			0xFE => {
				let before = self.reg.a;
				let n = self.fetch();
				let a = self.reg.a;
				let result = a.wrapping_sub(n);
				let hc = (before & 0xF) < 1;
				let bit_7 = (result >> 7) & 0b1;

				self.set_flag(Flags::Z, result == 0);
				self.set_flag(Flags::N, true);
				self.set_flag(Flags::H, hc);
				self.set_flag(Flags::C, bit_7 == 1);
			}
			_ => return Err(format!("Unknow instruction. OPCODE: {:02X}", instruction)),
		}

		Ok(())
	}

	pub fn step(&mut self) -> Result<(), String> {
		let instruction = self.fetch();
		// println!("instruction: 0x{:02X}", instruction);
		match self.decode(instruction) {
			Err(e) => return Err(format!("ERROR: {}", e)),
			Ok(_) => return Ok(()),
		}
	}
}

impl fmt::Display for CPU {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "A: 0x{:02X}    AF: 0x{:02X}{:02X}", self.reg.a, self.reg.a, self.reg.f)?;
		writeln!(f, "B: 0x{:02X}    BC: 0x{:02X}{:02X}", self.reg.b, self.reg.b, self.reg.c)?;
		writeln!(f, "C: 0x{:02X}    DE: 0x{:02X}{:02X}", self.reg.c, self.reg.d, self.reg.e)?;
		writeln!(f, "D: 0x{:02X}    HL: 0x{:02X}{:02X}", self.reg.d, self.reg.h, self.reg.l)?;
		writeln!(f, "E: 0x{:02X}", self.reg.e)?;
		writeln!(f, "F: 0x{:02X}    FLAGS:", self.reg.f)?;
		writeln!(
			f,
			"H: 0x{:02X}    Z:{} N:{} H:{} C:{}",
			self.reg.h,
			self.get_flag(Flags::Z),
			self.get_flag(Flags::N),
			self.get_flag(Flags::H),
			self.get_flag(Flags::C)
		)?;
		writeln!(f, "L: 0x{:02X}", self.reg.l)?;
		writeln!(f, "PC: 0x{:04X}", self.reg.pc)?;
		writeln!(f, "SP: 0x{:04X}", self.reg.sp)?;

		writeln!(f, "{}", "=".repeat(40))
	}
}
