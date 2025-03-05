#![allow(dead_code)]
use serde::{de::Error, Deserialize};
use std::collections::HashMap;

use std::fmt;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone, Debug, Deserialize)]
pub struct Operand {
	name: String,
	bytes: Option<u8>,
	increment: Option<bool>,
	decrement: Option<bool>,
	immediate: bool,
	value: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Instruction {
	mnemonic: String,
	bytes: u8,
	cycles: Vec<u8>,
	operands: Vec<Operand>,
	flags: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Opcode(u8);

impl<'de> Deserialize<'de> for Opcode {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s: String = Deserialize::deserialize(deserializer)?;

		let numbers = s.trim_start_matches("0x");
		u8::from_str_radix(numbers, 16)
			.map(Opcode)
			.map_err(D::Error::custom)
	}
}

type Instructions = HashMap<Opcode, Instruction>;

#[derive(Debug, Deserialize)]
pub struct InstructionBank {
	pub unprefixed: Instructions,
	pub cbprefixed: Instructions,
}

impl fmt::Display for Operand {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.value.is_some() {
			if self.immediate {
				write!(f, "{:#02X}", self.value.unwrap())
			} else {
				write!(f, "({:#02X})", self.value.unwrap())
			}
		} else {
			let mut name = self.name.clone();

			if self.increment.is_some() {
				name += "+";
			}

			if self.decrement.is_some() {
				name += "-";
			}

			if self.immediate {
				write!(f, "{}", name)
			} else {
				write!(f, "({})", name)
			}
		}
	}
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let operand_string = self
			.operands
			.iter()
			.map(|n| n.to_string())
			.collect::<Vec<String>>()
			.join(", ");

		if operand_string.is_empty() {
			write!(f, "{}", self.mnemonic)
		} else {
			write!(f, "{} {}", self.mnemonic, operand_string)
		}
	}
}

pub fn parse_from_file(location: &str) -> InstructionBank {
	let file = File::open(location).expect("Unable to read file.");
	let buffer = BufReader::new(file);

	let bank: InstructionBank = serde_json::from_reader(buffer).expect("Unable to parse data.");

	return bank;
}

// pub fn load_rom(location: &str) -> Vec<u8> {
//   let mut file = File::open(location).expect("Unable to read the path.");
//   let mut buffer: Vec<u8> = Vec::new();

//   file
//     .read_to_end(&mut buffer)
//     .expect("Unable to read the file");

//   return buffer;
// }

pub fn decode(
	mut addr: usize,
	bytes: &[u8],
	instruction: &InstructionBank,
) -> (usize, Instruction) {
	let opcode = bytes[addr];
	addr += 1;

	let meta_intruction = if opcode == 0xCB {
		let opc = bytes[addr];
		addr += 1;

		&instruction.cbprefixed[&Opcode(opc)]
	} else {
		&instruction.unprefixed[&Opcode(opcode)]
	};

	let decoded_operands = meta_intruction
		.operands
		.iter()
		.map(|o| match o.bytes {
			Some(2) => {
				let v: [u8; 2] = bytes[addr..addr + 2].try_into().expect("Out of bounds");
				addr += 2;

				Operand {
					name: o.name.clone(),
					bytes: o.bytes,
					increment: o.increment,
					decrement: o.decrement,
					immediate: o.immediate,
					value: Some(u16::from_le_bytes(v)),
				}
			}

			Some(1) => {
				let val = bytes[addr];
				addr += 1;

				Operand {
					name: o.name.clone(),
					bytes: o.bytes,
					increment: o.increment,
					decrement: o.decrement,
					immediate: o.immediate,
					value: Some(u16::from(val)),
				}
			}

			Some(_) => o.clone(),
			None => o.clone(),
		})
		.collect::<Vec<Operand>>();

	let decoded_instruction = Instruction {
		mnemonic: meta_intruction.mnemonic.clone(),
		bytes: meta_intruction.bytes,
		cycles: meta_intruction.cycles.clone(),
		operands: decoded_operands,
		flags: meta_intruction.flags.clone(),
	};

	return (addr, decoded_instruction);
}

pub fn disassemble(
	star_addr: usize,
	bytes: &[u8],
	instructions: &InstructionBank,
	amount_of_instruction: usize,
) {
	let mut addr = star_addr;

	for _ in 0..amount_of_instruction {
		let (new_addr, instruction) = decode(addr, bytes, instructions);

		println!("{:#02X} {}", addr, instruction);
		addr = new_addr;
	}
}
