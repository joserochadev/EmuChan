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

		// Helper para mapear índices de registro 0-7 para nomes
		let reg_name = |idx: u8| match idx {
			0 => "B",
			1 => "C",
			2 => "D",
			3 => "E",
			4 => "H",
			5 => "L",
			6 => "(HL)",
			7 => "A",
			_ => unreachable!(),
		};

		// Helper para condições de pulo (NZ, Z, NC, C)
		let cc_name = |idx: u8| match idx {
			0 => "NZ",
			1 => "Z",
			2 => "NC",
			3 => "C",
			_ => unreachable!(),
		};

		match opcode {
			// --- Controle / Misc ---
			0x00 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "NOP".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0x10 => DisassemblyLine {
				address,
				bytes: vec![opcode, imm8()],
				mnemonic: "STOP".to_string(),
				operands: format!("${:02X}", imm8()),
				cycles: 4,
			},
			0x76 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "HALT".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0xF3 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "DI".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0xFB => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "EI".to_string(),
				operands: String::new(),
				cycles: 4,
			},

			// --- Cargas de 16 bits (LD rr, nn) ---
			0x01 | 0x11 | 0x21 | 0x31 => {
				let nn = imm16();
				let reg = match opcode >> 4 {
					0 => "BC",
					1 => "DE",
					2 => "HL",
					3 => "SP",
					_ => unreachable!(),
				};
				DisassemblyLine {
					address,
					bytes: vec![opcode, (nn & 0xFF) as u8, (nn >> 8) as u8],
					mnemonic: "LD".to_string(),
					operands: format!("{}, ${:04X}", reg, nn),
					cycles: 12,
				}
			}

			// --- Cargas Indiretas (LD (rr), A e LD A, (rr)) ---
			0x02 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "(BC), A".to_string(),
				cycles: 8,
			},
			0x12 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "(DE), A".to_string(),
				cycles: 8,
			},
			0x0A => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "A, (BC)".to_string(),
				cycles: 8,
			},
			0x1A => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "A, (DE)".to_string(),
				cycles: 8,
			},

			// --- LD HL+/- (LDI/LDD) ---
			0x22 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "(HL+), A".to_string(),
				cycles: 8,
			},
			0x32 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "(HL-), A".to_string(),
				cycles: 8,
			},
			0x2A => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "A, (HL+)".to_string(),
				cycles: 8,
			},
			0x3A => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "A, (HL-)".to_string(),
				cycles: 8,
			},

			// --- Cargas de 8 bits Imediatas (LD r, n) ---
			0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => {
				let n = imm8();
				let reg = reg_name((opcode >> 3) & 0b111);
				DisassemblyLine {
					address,
					bytes: vec![opcode, n],
					mnemonic: "LD".to_string(),
					operands: format!("{}, ${:02X}", reg, n),
					cycles: 8,
				} // 12 ciclos para (HL)
			}
			0x36 => {
				// LD (HL), n (Tratamento especial pois é instrução de escrita na memória)
				let n = imm8();
				DisassemblyLine {
					address,
					bytes: vec![opcode, n],
					mnemonic: "LD".to_string(),
					operands: format!("(HL), ${:02X}", n),
					cycles: 12,
				}
			}

			// --- Cargas de 8 bits (LD r, r') - Cobre de 0x40 a 0x7F (exceto 0x76 HALT) ---
			0x40..=0x75 | 0x77..=0x7F => {
				let dest = reg_name((opcode >> 3) & 0b111);
				let src = reg_name(opcode & 0b111);
				let cycles = if src == "(HL)" || dest == "(HL)" { 8 } else { 4 };
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "LD".to_string(),
					operands: format!("{}, {}", dest, src),
					cycles,
				}
			}

			// --- ALU 8 bits (ADD, ADC, SUB, SBC, AND, XOR, OR, CP) - Cobre 0x80 a 0xBF ---
			0x80..=0xBF => {
				let op_idx = (opcode >> 3) & 0b111;
				let src_idx = opcode & 0b111;
				let mnemonic = ["ADD", "ADC", "SUB", "SBC", "AND", "XOR", "OR", "CP"][op_idx as usize];
				let operand = reg_name(src_idx);
				let cycles = if src_idx == 6 { 8 } else { 4 };
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: mnemonic.to_string(),
					operands: match op_idx {
						0 | 1 | 3 => format!("A, {}", operand), // ADD/ADC/SBC A, r (SBC A explicitado as vezes)
						_ => operand.to_string(),               // SUB/AND/XOR/OR/CP r
					},
					cycles,
				}
			}

			// --- ALU 8 bits Imediato (ADD A, n etc) ---
			0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
				let n = imm8();
				let op_idx = (opcode >> 3) & 0b111; // Mapeia para 0, 1, 2... similar ao bloco acima
				let mnemonic = ["ADD", "ADC", "SUB", "SBC", "AND", "XOR", "OR", "CP"][op_idx as usize];
				DisassemblyLine {
					address,
					bytes: vec![opcode, n],
					mnemonic: mnemonic.to_string(),
					operands: format!("${:02X}", n),
					cycles: 8,
				}
			}

			// --- Aritmética 16 bits (INC/DEC rr) ---
			0x03 | 0x13 | 0x23 | 0x33 => {
				let reg = match opcode >> 4 {
					0 => "BC",
					1 => "DE",
					2 => "HL",
					3 => "SP",
					_ => unreachable!(),
				};
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "INC".to_string(),
					operands: reg.to_string(),
					cycles: 8,
				}
			}
			0x0B | 0x1B | 0x2B | 0x3B => {
				let reg = match opcode >> 4 {
					0 => "BC",
					1 => "DE",
					2 => "HL",
					3 => "SP",
					_ => unreachable!(),
				};
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "DEC".to_string(),
					operands: reg.to_string(),
					cycles: 8,
				}
			}

			// --- Aritmética 8 bits (INC/DEC r) ---
			0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
				let reg = reg_name((opcode >> 3) & 0b111);
				let cycles = if reg == "(HL)" { 12 } else { 4 };
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "INC".to_string(),
					operands: reg.to_string(),
					cycles,
				}
			}
			0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
				let reg = reg_name((opcode >> 3) & 0b111);
				let cycles = if reg == "(HL)" { 12 } else { 4 };
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "DEC".to_string(),
					operands: reg.to_string(),
					cycles,
				}
			}

			// --- ADD HL, rr ---
			0x09 | 0x19 | 0x29 | 0x39 => {
				let reg = match opcode >> 4 {
					0 => "BC",
					1 => "DE",
					2 => "HL",
					3 => "SP",
					_ => unreachable!(),
				};
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "ADD".to_string(),
					operands: format!("HL, {}", reg),
					cycles: 8,
				}
			}

			// --- Rotações (RLCA, RRCA, RLA, RRA) ---
			0x07 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "RLCA".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0x0F => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "RRCA".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0x17 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "RLA".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0x1F => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "RRA".to_string(),
				operands: String::new(),
				cycles: 4,
			},

			// --- Jumps Relativos (JR) ---
			0x18 => {
				let n = imm8() as i8;
				let dest = (address as i32 + 2 + n as i32) as u16;
				DisassemblyLine {
					address,
					bytes: vec![opcode, n as u8],
					mnemonic: "JR".to_string(),
					operands: format!("${:04X}", dest),
					cycles: 12,
				}
			}
			0x20 | 0x28 | 0x30 | 0x38 => {
				let n = imm8() as i8;
				let dest = (address as i32 + 2 + n as i32) as u16;
				let cc = cc_name((opcode >> 3) & 0b11);
				DisassemblyLine {
					address,
					bytes: vec![opcode, n as u8],
					mnemonic: "JR".to_string(),
					operands: format!("{}, ${:04X}", cc, dest),
					cycles: 8,
				} // Ciclos variam (12/8)
			}

			// --- Jumps Absolutos (JP) ---
			0xC3 => {
				let nn = imm16();
				DisassemblyLine {
					address,
					bytes: vec![opcode, (nn & 0xFF) as u8, (nn >> 8) as u8],
					mnemonic: "JP".to_string(),
					operands: format!("${:04X}", nn),
					cycles: 16,
				}
			}
			0xC2 | 0xCA | 0xD2 | 0xDA => {
				let nn = imm16();
				let cc = cc_name((opcode >> 3) & 0b11);
				DisassemblyLine {
					address,
					bytes: vec![opcode, (nn & 0xFF) as u8, (nn >> 8) as u8],
					mnemonic: "JP".to_string(),
					operands: format!("{}, ${:04X}", cc, nn),
					cycles: 12,
				} // (16/12)
			}
			0xE9 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "JP".to_string(),
				operands: "(HL)".to_string(),
				cycles: 4,
			},

			// --- Calls ---
			0xCD => {
				let nn = imm16();
				DisassemblyLine {
					address,
					bytes: vec![opcode, (nn & 0xFF) as u8, (nn >> 8) as u8],
					mnemonic: "CALL".to_string(),
					operands: format!("${:04X}", nn),
					cycles: 24,
				}
			}
			0xC4 | 0xCC | 0xD4 | 0xDC => {
				let nn = imm16();
				let cc = cc_name((opcode >> 3) & 0b11);
				DisassemblyLine {
					address,
					bytes: vec![opcode, (nn & 0xFF) as u8, (nn >> 8) as u8],
					mnemonic: "CALL".to_string(),
					operands: format!("{}, ${:04X}", cc, nn),
					cycles: 12,
				} // (24/12)
			}

			// --- Returns ---
			0xC9 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "RET".to_string(),
				operands: String::new(),
				cycles: 16,
			},
			0xD9 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "RETI".to_string(),
				operands: String::new(),
				cycles: 16,
			},
			0xC0 | 0xC8 | 0xD0 | 0xD8 => {
				let cc = cc_name((opcode >> 3) & 0b11);
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "RET".to_string(),
					operands: cc.to_string(),
					cycles: 8,
				} // (20/8)
			}

			// --- RST ---
			0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
				let vec = opcode & 0x38;
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "RST".to_string(),
					operands: format!("${:02X}", vec),
					cycles: 16,
				}
			}

			// --- Stack (PUSH/POP) ---
			0xC5 | 0xD5 | 0xE5 | 0xF5 => {
				let reg = match (opcode >> 4) & 0b11 {
					0 => "BC",
					1 => "DE",
					2 => "HL",
					3 => "AF",
					_ => unreachable!(),
				};
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "PUSH".to_string(),
					operands: reg.to_string(),
					cycles: 16,
				}
			}
			0xC1 | 0xD1 | 0xE1 | 0xF1 => {
				let reg = match (opcode >> 4) & 0b11 {
					0 => "BC",
					1 => "DE",
					2 => "HL",
					3 => "AF",
					_ => unreachable!(),
				};
				DisassemblyLine {
					address,
					bytes: vec![opcode],
					mnemonic: "POP".to_string(),
					operands: reg.to_string(),
					cycles: 12,
				}
			}

			// --- Stack e SP Misc ---
			0xE8 => {
				let n = imm8() as i8;
				DisassemblyLine {
					address,
					bytes: vec![opcode, n as u8],
					mnemonic: "ADD".to_string(),
					operands: format!("SP, ${:02X}", n),
					cycles: 16,
				}
			}
			0xF8 => {
				let n = imm8() as i8;
				DisassemblyLine {
					address,
					bytes: vec![opcode, n as u8],
					mnemonic: "LD".to_string(),
					operands: format!("HL, SP+${:02X}", n),
					cycles: 12,
				}
			}
			0xF9 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "SP, HL".to_string(),
				cycles: 8,
			},

			// --- Loads High Memory (LDH) e Direct Address ---
			0xE0 => {
				let n = imm8();
				DisassemblyLine {
					address,
					bytes: vec![opcode, n],
					mnemonic: "LDH".to_string(),
					operands: format!("($FF{:02X}), A", n),
					cycles: 12,
				}
			}
			0xF0 => {
				let n = imm8();
				DisassemblyLine {
					address,
					bytes: vec![opcode, n],
					mnemonic: "LDH".to_string(),
					operands: format!("A, ($FF{:02X})", n),
					cycles: 12,
				}
			}
			0xE2 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "($FF00+C), A".to_string(),
				cycles: 8,
			},
			0xF2 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "LD".to_string(),
				operands: "A, ($FF00+C)".to_string(),
				cycles: 8,
			},
			0xEA => {
				let nn = imm16();
				DisassemblyLine {
					address,
					bytes: vec![opcode, (nn & 0xFF) as u8, (nn >> 8) as u8],
					mnemonic: "LD".to_string(),
					operands: format!("(${:04X}), A", nn),
					cycles: 16,
				}
			}
			0xFA => {
				let nn = imm16();
				DisassemblyLine {
					address,
					bytes: vec![opcode, (nn & 0xFF) as u8, (nn >> 8) as u8],
					mnemonic: "LD".to_string(),
					operands: format!("A, (${:04X})", nn),
					cycles: 16,
				}
			}

			// --- Misc Op ---
			0x27 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "DAA".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0x2F => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "CPL".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0x37 => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "SCF".to_string(),
				operands: String::new(),
				cycles: 4,
			},
			0x3F => DisassemblyLine {
				address,
				bytes: vec![opcode],
				mnemonic: "CCF".to_string(),
				operands: String::new(),
				cycles: 4,
			},

			// Fallback para desconhecidos
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
