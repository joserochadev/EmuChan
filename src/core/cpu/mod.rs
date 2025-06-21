#![allow(dead_code)]
use std::{
	fmt,
	sync::{Arc, Mutex},
};

mod opcodes;
mod register;

use opcodes::Opcode;
use register::{Flags, Register};

use crate::core::bus::BUS;

#[derive(Debug)]
pub struct CPU {
	pub reg: Register,
	bus: Arc<Mutex<BUS>>,
	pub cycles: usize,
}

impl CPU {
	pub fn new(bus: Arc<Mutex<BUS>>) -> CPU {
		CPU {
			reg: Register::new(),
			bus,
			cycles: 0,
		}
	}

	pub fn read(&mut self, addr: u16) -> u8 {
		let bus = self.bus.lock().unwrap();
		bus.read(addr)
	}

	pub fn write(&mut self, addr: u16, data: u8) {
		let mut bus = self.bus.lock().unwrap();
		bus.write(addr, data);
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

	pub fn set_cycles(&mut self, t_cycles: usize) {
		self.cycles = t_cycles / 4; // convert to M-Cycles
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


	pub fn step(&mut self) -> Result<(), String> {
		let instruction = self.fetch();
		let mut opcode = Opcode::new(self);

		match opcode.decode(instruction) {
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
			self.reg.get_flag(Flags::Z),
			self.reg.get_flag(Flags::N),
			self.reg.get_flag(Flags::H),
			self.reg.get_flag(Flags::C)
		)?;
		writeln!(f, "L: 0x{:02X}", self.reg.l)?;
		writeln!(f, "PC: 0x{:04X}", self.reg.pc)?;
		writeln!(f, "SP: 0x{:04X}", self.reg.sp)?;

		writeln!(f, "{}", "=".repeat(40))
	}
}
