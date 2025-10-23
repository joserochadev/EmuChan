use crate::core::cpu::{
	register::{Flags, Register16, Register8},
	CPU,
};

#[derive(Clone, Copy)]
pub enum Condition {
	NZ,
	Z,
	NC,
	C,
}

pub struct Opcode<'a> {
	cpu: &'a mut CPU,
}

impl<'a> Opcode<'a> {
	pub fn new(cpu: &'a mut CPU) -> Self {
		Self { cpu }
	}

	pub fn decode(&mut self, instruction: u8) -> Result<(), String> {
		match instruction {
			0x00 => self.nop(),

			0x06 => self.ld_r8_imm8(Register8::B),
			0x0E => self.ld_r8_imm8(Register8::C),
			0x16 => self.ld_r8_imm8(Register8::D),
			0x1E => self.ld_r8_imm8(Register8::E),
			0x2E => self.ld_r8_imm8(Register8::L),
			0x3E => self.ld_r8_imm8(Register8::A),

			0x4F => self.ld_r8_r8(Register8::C, Register8::A),
			0x57 => self.ld_r8_r8(Register8::D, Register8::A),
			0x67 => self.ld_r8_r8(Register8::H, Register8::A),
			0x78 => self.ld_r8_r8(Register8::A, Register8::B),
			0x7B => self.ld_r8_r8(Register8::A, Register8::E),
			0x7C => self.ld_r8_r8(Register8::A, Register8::H),
			0x7D => self.ld_r8_r8(Register8::A, Register8::L),

			0x1A => self.ld_a_from_addr_r16(Register16::DE),
			0xF0 => self.ld_a_from_addr_ff00_plus_imm8(),

			0x22 => self.ld_a_into_hli(),
			0x32 => self.ld_a_into_hld(),
			0x77 => self.ld_into_hl_r8(Register8::A),
			0xE0 => self.ld_into_ff00_plus_u8_reg_a(),
			0xE2 => self.ld_to_addr_ff00_plus_c_from_a(),
			0xEA => self.ld_to_addr_u16_from_a(),

			0x11 => self.ld_r16_imm16(Register16::DE),
			0x21 => self.ld_r16_imm16(Register16::HL),
			0x31 => self.ld_r16_imm16(Register16::SP),

			0x04 => self.inc_r8(Register8::B),
			0x0C => self.inc_r8(Register8::C),
			0x24 => self.inc_r8(Register8::H),

			0x05 => self.dec_r8(Register8::B),
			0x0D => self.dec_r8(Register8::C),
			0x15 => self.dec_r8(Register8::D),
			0x1D => self.dec_r8(Register8::E),
			0x3D => self.dec_r8(Register8::A),

			0xAF => self.xor_r8_r8(Register8::A, Register8::A),

			0x86 => {
				let addr = self.cpu.reg.get_r16(Register16::HL);
				let value = self.cpu.read(addr);
				self.add_a(value);
				self.cpu.set_cycles(4);
			}
			0x90 => {
				let value = self.cpu.reg.get_r8(Register8::B);
				self.sub_a(value);
			}
			0xBE => {
				let addr = self.cpu.reg.get_r16(Register16::HL);
				let value = self.cpu.read(addr);
				self.cp_a(value);
				self.cpu.set_cycles(8);
			}
			0xFE => {
				let value = self.cpu.fetch();
				self.cp_a(value);
				self.cpu.set_cycles(8);
			}

			0x13 => self.inc_r16(Register16::DE),
			0x23 => self.inc_r16(Register16::HL),

			0x17 => self.rla(),

			0xC5 => self.push_r16(Register16::BC),
			0xC1 => self.pop_r16(Register16::BC),

			0xCD => self.call_imm16(),
			0xC9 => self.ret(),

			0x18 => self.jr_imm8(),
			0x20 => self.jr_cc_imm8(Condition::NZ),
			0x28 => self.jr_cc_imm8(Condition::Z),
			0x30 => self.jr_cc_imm8(Condition::NC),
			0x38 => self.jr_cc_imm8(Condition::C),

			0xC0 => self.ret_cc(Condition::NZ),
			0xC8 => self.ret_cc(Condition::Z),
			0xD0 => self.ret_cc(Condition::NC),
			0xD8 => self.ret_cc(Condition::C),

			0xCB => self.decode_cb()?,

			_ => return Err(format!("Unknow instruction. OPCODE: {:02X}", instruction)),
		}

		Ok(())
	}

	// Dispatcher for 0xCBxx instructions
	fn decode_cb(&mut self) -> Result<(), String> {
		let cb_instruction = self.cpu.fetch();
		match cb_instruction {
			// BIT family b, r
			0x40..=0x7F => {
				// Decode the bit and register from the opcode
				let bit = (cb_instruction - 0x40) / 8;
				let reg_code = cb_instruction % 8;
				let reg = self.decode_register8(reg_code)?;
				self.op_cb_bit(bit, reg);
			}

			// RL Family r
			0x10..=0x17 => {
				let reg_code = cb_instruction % 8;
				let reg = self.decode_register8(reg_code)?;
				self.op_cb_rl(reg);
			}

			_ => return Err(format!("Unknow CB instruction. OPCODE: {:02X}", cb_instruction)),
		}
		Ok(())
	}

	// Helper function to decode the register from the 3-bit code
	fn decode_register8(&self, code: u8) -> Result<Register8, String> {
		match code {
			0 => Ok(Register8::B),
			1 => Ok(Register8::C),
			2 => Ok(Register8::D),
			3 => Ok(Register8::E),
			4 => Ok(Register8::H),
			5 => Ok(Register8::L),
			// 6 is (HL), treated separately
			7 => Ok(Register8::A),
			_ => Err("Invalid register code".to_string()),
		}
	}

	fn op_cb_bit(&mut self, bit: u8, reg: Register8) {
		let value = self.cpu.reg.get_r8(reg);
		let result = (value >> bit) & 1;

		self.cpu.reg.set_flag(Flags::Z, result == 0);
		self.cpu.reg.set_flag(Flags::N, false);
		self.cpu.reg.set_flag(Flags::H, true);
		self.cpu.set_cycles(8);
	}

	fn op_cb_rl(&mut self, reg: Register8) {
		let value = self.cpu.reg.get_r8(reg);
		let carry = self.cpu.reg.get_flag(Flags::C);
		let new_carry = (value >> 7) & 1;

		let result = (value << 1) | carry;
		self.cpu.reg.set_r8(reg, result);

		self.cpu.reg.set_flag(Flags::Z, result == 0);
		self.cpu.reg.set_flag(Flags::N, false);
		self.cpu.reg.set_flag(Flags::H, false);
		self.cpu.reg.set_flag(Flags::C, new_carry == 1);
		self.cpu.set_cycles(8);
	}

	fn check_condition(&self, cond: Condition) -> bool {
		match cond {
			Condition::NZ => self.cpu.reg.get_flag(Flags::Z) == 0,
			Condition::Z => self.cpu.reg.get_flag(Flags::Z) == 1,
			Condition::NC => self.cpu.reg.get_flag(Flags::C) == 0,
			Condition::C => self.cpu.reg.get_flag(Flags::C) == 1,
		}
	}

	// load a immediate value in a register (LD r, u8)
	fn ld_r8_imm8(&mut self, reg: Register8) {
		let data = self.cpu.fetch();
		self.cpu.reg.set_r8(reg, data);
		self.cpu.set_cycles(8);
	}

	// load register value into another register (LD r, r)
	fn ld_r8_r8(&mut self, reg1: Register8, reg2: Register8) {
		let value = self.cpu.reg.get_r8(reg2);
		self.cpu.reg.set_r8(reg1, value);
		self.cpu.set_cycles(4);
	}

	// load into register A a value obtained from an address formed by a 16-bit register
	fn ld_a_from_addr_r16(&mut self, reg: Register16) {
		let addr = self.cpu.reg.get_r16(reg);
		let data = self.cpu.read(addr);
		self.cpu.reg.set_r8(Register8::A, data);
		self.cpu.set_cycles(8);
	}

	// load into register A a value obtained from 0xFF00+u8
	fn ld_a_from_addr_ff00_plus_imm8(&mut self) {
		let data = self.cpu.fetch();
		let addr = ((0xFF << 8) as u16) | data as u16;
		self.cpu.reg.a = self.cpu.read(addr);
		self.cpu.set_cycles(12);
	}

	// load into address HL the register A, after incremente HL
	fn ld_a_into_hli(&mut self) {
		let data = self.cpu.reg.a;
		let addr = self.cpu.reg.get_r16(Register16::HL);
		self.cpu.write(addr, data);
		self.cpu.reg.set_r16(Register16::HL, addr + 1);
		self.cpu.set_cycles(8);
	}

	// load into address HL the register A, after decremente HL
	fn ld_a_into_hld(&mut self) {
		let data = self.cpu.reg.a;
		let addr = self.cpu.reg.get_r16(Register16::HL);
		self.cpu.write(addr, data);
		self.cpu.reg.set_r16(Register16::HL, addr - 1);
		self.cpu.set_cycles(8);
	}

	// load into address HL a 8-bit register
	fn ld_into_hl_r8(&mut self, reg: Register8) {
		let data = self.cpu.reg.get_r8(reg);
		let addr = self.cpu.reg.get_r16(Register16::HL);
		self.cpu.write(addr, data);
		self.cpu.set_cycles(8);
	}

	// load into FF00+u8 the value of register A
	fn ld_into_ff00_plus_u8_reg_a(&mut self) {
		let hi = (0xFF << 8) as u16;
		let lo = self.cpu.fetch() as u16;
		let addr = hi | lo;
		let data = self.cpu.reg.a;
		self.cpu.write(addr, data);
	}

	// load into FF00+C register A
	fn ld_to_addr_ff00_plus_c_from_a(&mut self) {
		let hi = (0xFF << 8) as u16;
		let lo = self.cpu.reg.c as u16;
		let addr = hi | lo;
		self.cpu.write(addr, self.cpu.reg.a);
		self.cpu.set_cycles(8);
	}

	// load addr to immediate 16-bits from register a
	fn ld_to_addr_u16_from_a(&mut self) {
		let addr = self.cpu.fetch16();
		self.cpu.write(addr, self.cpu.reg.a);
		self.cpu.set_cycles(16);
	}

	// load 16-bits value into 16-bits register
	fn ld_r16_imm16(&mut self, reg: Register16) {
		let data = self.cpu.fetch16();
		self.cpu.reg.set_r16(reg, data);
		self.cpu.set_cycles(12);
	}

	// incremet 8bits register
	fn inc_r8(&mut self, reg: Register8) {
		let before = self.cpu.reg.get_r8(reg);
		let result = before.wrapping_add(1);
		self.cpu.reg.set_r8(reg, result);
		let hc = (before & 0xF) + (1 & 0xF);

		self.cpu.reg.set_flag(Flags::Z, result == 0);
		self.cpu.reg.set_flag(Flags::N, false);
		self.cpu.reg.set_flag(Flags::H, hc > 0xF);
		self.cpu.set_cycles(4);
	}

	// decrement 8bits register
	fn dec_r8(&mut self, reg: Register8) {
		let before = self.cpu.reg.get_r8(reg);
		let result = before.wrapping_sub(1);
		self.cpu.reg.set_r8(reg, result);

		let hc = (before & 0xF) < 1;

		self.cpu.reg.set_flag(Flags::Z, result == 0);
		self.cpu.reg.set_flag(Flags::N, true);
		self.cpu.reg.set_flag(Flags::H, hc);
		self.cpu.set_cycles(4);
	}

	// Increment a 16-bit register
	fn inc_r16(&mut self, reg: Register16) {
		let value = self.cpu.reg.get_r16(reg).wrapping_add(1);
		self.cpu.reg.set_r16(reg, value);
		self.cpu.set_cycles(8);
	}

	// Decrement a 16-bit register
	fn dec_r16(&mut self, reg: Register16) {
		let value = self.cpu.reg.get_r16(reg).wrapping_sub(1);
		self.cpu.reg.set_r16(reg, value);
		self.cpu.set_cycles(8);
	}

	// xor with two registers
	fn xor_r8_r8(&mut self, reg1: Register8, reg2: Register8) {
		let r1 = self.cpu.reg.get_r8(reg1);
		let r2 = self.cpu.reg.get_r8(reg2);
		let result = r1 ^ r2;
		self.cpu.reg.set_r8(reg1, result);

		self.cpu.reg.set_flag(Flags::Z, result == 0);
		self.cpu.reg.set_flag(Flags::N, false);
		self.cpu.reg.set_flag(Flags::H, false);
		self.cpu.reg.set_flag(Flags::C, false);
		self.cpu.set_cycles(4);
	}

	// Helper function for ADDITION logic
	fn add_a(&mut self, value: u8) {
		let a = self.cpu.reg.a;
		let result = a.wrapping_add(value);
		self.cpu.reg.a = result;

		self.cpu.reg.set_flag(Flags::Z, result == 0);
		self.cpu.reg.set_flag(Flags::N, false);
		self
			.cpu
			.reg
			.set_flag(Flags::H, (a & 0x0F) + (value & 0x0F) > 0x0F);
		self
			.cpu
			.reg
			.set_flag(Flags::C, (a as u16) + (value as u16) > 0xFF);
		self.cpu.set_cycles(4);
	}

	// Helper function for SUBTRACTION logic
	fn sub_a(&mut self, value: u8) {
		let a = self.cpu.reg.a;
		let result = a.wrapping_sub(value);
		self.cpu.reg.a = result;

		self.cpu.reg.set_flag(Flags::Z, result == 0);
		self.cpu.reg.set_flag(Flags::N, true);
		self.cpu.reg.set_flag(Flags::H, (a & 0x0F) < (value & 0x0F));
		self.cpu.reg.set_flag(Flags::C, (a as u16) < (value as u16));
		self.cpu.set_cycles(4);
	}

	fn cp_a(&mut self, value: u8) {
		let a = self.cpu.reg.a;
		let result = a.wrapping_sub(value);

		self.cpu.reg.set_flag(Flags::Z, result == 0);
		self.cpu.reg.set_flag(Flags::N, true);
		self.cpu.reg.set_flag(Flags::H, (a & 0x0F) < (value & 0x0F));
		self.cpu.reg.set_flag(Flags::C, a < value);
		self.cpu.set_cycles(4);
	}

	// RLA - Rotates A to the left via Carry
	fn rla(&mut self) {
		let a = self.cpu.reg.a;
		let carry = self.cpu.reg.get_flag(Flags::C);
		let new_carry = (a >> 7) & 1;

		let result = (a << 1) | carry;
		self.cpu.reg.a = result;

		self.cpu.reg.set_flag(Flags::Z, false); // Z é sempre 0
		self.cpu.reg.set_flag(Flags::N, false);
		self.cpu.reg.set_flag(Flags::H, false);
		self.cpu.reg.set_flag(Flags::C, new_carry == 1);
		self.cpu.set_cycles(4);
	}

	// NOP - No Operation
	fn nop(&mut self) {
		self.cpu.set_cycles(4);
	}

	// Relative Unconditional Jump
	fn jr_imm8(&mut self) {
		let offset = self.cpu.fetch() as i8;
		self.cpu.reg.pc = self.cpu.reg.pc.wrapping_add(offset as u16);
		self.cpu.set_cycles(12);
	}

	// Unconditional Subroutine Call
	fn call_imm16(&mut self) {
		let addr = self.cpu.fetch16();
		self.cpu.push(self.cpu.reg.pc);
		self.cpu.reg.pc = addr;
		self.cpu.set_cycles(24);
	}

	// Unconditional Subroutine Return
	fn ret(&mut self) {
		self.cpu.reg.pc = self.cpu.pop();
		self.cpu.set_cycles(16);
	}

	// Push a 16-bit register onto the stack
	fn push_r16(&mut self, reg: Register16) {
		let value = self.cpu.reg.get_r16(reg);
		self.cpu.push(value);
		self.cpu.set_cycles(16);
	}

	// Pop a value from the stack into a 16-bit register
	fn pop_r16(&mut self, reg: Register16) {
		let value = self.cpu.pop();
		self.cpu.reg.set_r16(reg, value);
		self.cpu.set_cycles(12);
	}

	// Conditional relative jump
	fn jr_cc_imm8(&mut self, cond: Condition) {
		let offset = self.cpu.fetch() as i8;
		if self.check_condition(cond) {
			self.cpu.reg.pc = self.cpu.reg.pc.wrapping_add(offset as u16);
			self.cpu.set_cycles(12);
		} else {
			self.cpu.set_cycles(8);
		}
	}

	// Conditional return
	fn ret_cc(&mut self, cond: Condition) {
		// Ciclos variam um pouco, mas a lógica é essa
		if self.check_condition(cond) {
			self.cpu.reg.pc = self.cpu.pop();
			self.cpu.set_cycles(20);
		} else {
			self.cpu.set_cycles(8);
		}
	}
}
