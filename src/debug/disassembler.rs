use super::messages::DisassemblyLine;

#[derive(Debug, Clone)]
pub struct Disassembler;

impl Disassembler {
	pub fn disassemble_at(memory: &[u8], address: u16) -> DisassemblyLine {
		let addr = address as usize;
		if addr >= memory.len() {
			return Self::invalid_instruction(address);
		}

		let opcode = memory[addr];

		if opcode == 0xCB {
			return Self::disassemble_cb(memory, address);
		}

		Self::disassemble_opcode(memory, address, opcode)
	}

	pub fn disassemble_range(memory: &[u8], start: u16, count: usize) -> Vec<DisassemblyLine> {
		let mut instructions = Vec::new();
		let mut address = start;

		for _ in 0..count {
			if address as usize >= memory.len() {
				break;
			}

			let instr = Self::disassemble_at(memory, address);
			address = address.wrapping_add(instr.bytes.len() as u16);
			instructions.push(instr);
		}

		instructions
	}

	fn disassemble_opcode(memory: &[u8], address: u16, opcode: u8) -> DisassemblyLine {
		let addr = address as usize;

		// Immediate u8 and u16 values
		let imm8 = || memory.get(addr + 1).copied().unwrap_or(0);
		let imm16 = || {
			let low = memory.get(addr + 1).copied().unwrap_or(0);
			let high = memory.get(addr + 2).copied().unwrap_or(0);
			u16::from_be_bytes([high, low])
		};

		match opcode {
			// NOP
			0x00 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "NOP".to_string(),
				operands: String::new(),
				cycles: 4,
			},

			// LD r16, nn
			0x01 => {
				let nn = imm16();
				DisassemblyLine {
					address,
					bytes: vec![opcode, nn as u8, (nn >> 8) as u8],
					mnemonic: "LD".to_string(),
					operands: format!("BC, ${:04X}", nn),
					cycles: 12,
				}
			}

			0x11 => {
				let nn = imm16();
				DisassemblyLine {
					address,
					bytes: vec![opcode, nn as u8, (nn >> 8) as u8],
					mnemonic: "LD".to_string(),
					operands: format!("DE, ${:04X}", nn),
					cycles: 12,
				}
			}

			0x21 => {
				let nn = imm16();
				DisassemblyLine {
					address,
					bytes: vec![opcode, nn as u8, (nn >> 8) as u8],
					mnemonic: "LD".to_string(),
					operands: format!("HL, ${:04X}", nn),
					cycles: 12,
				}
			}

			0x31 => {
				let nn = imm16();
				DisassemblyLine {
					address,
					bytes: vec![opcode, nn as u8, (nn >> 8) as u8],
					mnemonic: "LD".to_string(),
					operands: format!("SP, ${:04X}", nn),
					cycles: 12,
				}
			}

			// LD r8, n
			0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => {
				let n = imm8();
				let reg = match opcode {
					0x06 => "B",
					0x0E => "C",
					0x16 => "D",
					0x1E => "E",
					0x26 => "H",
					0x2E => "L",
					0x3E => "A",
					_ => unreachable!(),
				};

				DisassemblyLine {
					address,
					bytes: vec![opcode, n],
					mnemonic: "LD".to_string(),
					operands: format!("{}, ${:02X}", reg, n),
					cycles: 8,
				}
			}

			// LD r8, r8
			0x4F | 0x57 | 0x67 => {
				let reg = match opcode {
					0x4F => "C",
					0x57 => "D",
					0x67 => "H",
					_ => unreachable!(),
				};

				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "LD".to_string(),
					operands: format!("{}, A", reg),
					cycles: 4,
				}
			}

			_ => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "???".to_string(),
				operands: format!("${:02X}", opcode),
				cycles: 4,
			},
		}
	}

	fn disassemble_cb(memory: &[u8], address: u16) -> DisassemblyLine {
		let addr = address as usize;
		let cb_opcode = memory.get(addr).copied().unwrap_or(0);

		let bit = (cb_opcode >> 3) & 0b00000111;
		let reg = cb_opcode & 0b00000111;
		let reg_name = ["B", "C", "D", "E", "H", "L", "(HL)", "A"][reg as usize];

		let (mnemonic, operands, cycles) = match cb_opcode >> 6 {
			0 => {
				let op = ["RLC", "RRC", "RL", "RR", "SLA", "SRA", "SWAP", "SRL"]
					[(cb_opcode >> 3) as usize & 0b00000111];
				(op.to_string(), reg_name.to_string(), if reg == 6 { 16 } else { 8 })
			}
			1 => (
				"BIT".to_string(),
				format!("{}, {}", bit, reg_name),
				if reg == 6 { 12 } else { 8 },
			),
			2 => (
				"RES".to_string(),
				format!("{}, {}", bit, reg_name),
				if reg == 6 { 16 } else { 8 },
			),
			3 => (
				"SET".to_string(),
				format!("{}, {}", bit, reg_name),
				if reg == 6 { 16 } else { 8 },
			),
			_ => unreachable!(),
		};

		DisassemblyLine {
			address,
			bytes: vec![0xCB, cb_opcode],
			mnemonic,
			operands,
			cycles,
		}
	}

	fn invalid_instruction(address: u16) -> DisassemblyLine {
		DisassemblyLine {
			address,
			bytes: vec![0xFF],
			mnemonic: "INVALID".to_string(),
			operands: String::new(),
			cycles: 0,
		}
	}
}
