use colored::Colorize;
use serde::Deserialize;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

use crate::core::bus::BUS;
use crate::core::cpu::CPU;

#[derive(Debug, Clone, Copy, Deserialize)]
struct RegisteState {
	pc: u16,
	sp: u16,
	a: u8,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	f: u8,
	h: u8,
	l: u8,
	ime: u8,
	ie: Option<u8>,
}

impl fmt::Display for RegisteState {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"PC: {:#06X}, SP: {:#06X}, A: {:#04X}, B: {:#04X}, C: {:#04X}, D: {:#04X}, E: {:#04X}, \
         F: {:#04X}, H: {:#04X}, L: {:#04X}, IME: {}, IE: {:?}",
			self.pc,
			self.sp,
			self.a,
			self.b,
			self.c,
			self.d,
			self.e,
			self.f,
			self.h,
			self.l,
			self.ime,
			self.ie
		)
	}
}

#[derive(Debug, Deserialize)]
struct MemoryState(u16, Option<u8>, String);

impl fmt::Display for MemoryState {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Address: {:#06X}, Value: {:?}, Description: {}", self.0, self.1, self.2)
	}
}

#[derive(Debug, Deserialize)]
struct Snapshot {
	name: String,
	initial: RegisteState,
	final_: RegisteState,
	cycles: Vec<MemoryState>,
}

impl fmt::Display for Snapshot {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{}: {}\n\
           {}:\n{}\n\
           {}:\n{}\n\
          {}:\n{}\n",
			"Name".bold().yellow(),
			self.name,
			"Initial State".bold().cyan(),
			self.initial,
			"Final State".bold().cyan(),
			self.final_,
			"Memory Cycles".bold().purple(),
			self
				.cycles
				.iter()
				.map(|cycle| format!("{}", cycle))
				.collect::<Vec<_>>()
				.join("\n")
		)
	}
}

pub struct SM83 {
	_bus: Arc<Mutex<BUS>>,
	cpu: CPU,
}

impl SM83 {
	pub fn new() -> Self {
		let bus = Arc::new(Mutex::new(BUS::new()));
		let cpu = CPU::new(Arc::clone(&bus));

		Self { _bus: bus, cpu }
	}

	fn inject(&mut self, intial_state: RegisteState) {
		let RegisteState {
			pc,
			sp,
			a,
			b,
			c,
			d,
			e,
			f,
			h,
			l,
			..
		} = intial_state;

		self.cpu.reg.a = a;
		self.cpu.reg.b = b;
		self.cpu.reg.c = c;
		self.cpu.reg.d = d;
		self.cpu.reg.e = e;
		self.cpu.reg.f = f;
		self.cpu.reg.h = h;
		self.cpu.reg.l = l;
		self.cpu.reg.pc = pc;
		self.cpu.reg.sp = sp;
	}

	fn retrive(&self) -> RegisteState {
		let a = self.cpu.reg.a;
		let b = self.cpu.reg.b;
		let c = self.cpu.reg.c;
		let d = self.cpu.reg.d;
		let e = self.cpu.reg.e;
		let f = self.cpu.reg.f;
		let h = self.cpu.reg.h;
		let l = self.cpu.reg.l;
		let pc = self.cpu.reg.pc;
		let sp = self.cpu.reg.sp;
		let ime = self.cpu.reg.ime;
		let ie = self.cpu.reg.ie;

		return RegisteState {
			a,
			b,
			c,
			d,
			e,
			f,
			h,
			l,
			pc,
			sp,
			ime,
			ie: Some(ie),
		};
	}

	fn compare_state(&self, expected: &RegisteState) -> bool {
		let actual = self.retrive();
		expected.pc == actual.pc
			&& expected.sp == actual.sp
			&& expected.a == actual.a
			&& expected.b == actual.b
			&& expected.c == actual.c
			&& expected.d == actual.d
			&& expected.e == actual.e
			&& expected.f == actual.f
			&& expected.h == actual.h
			&& expected.l == actual.l
	}

	pub fn run_test(&mut self, file_path: String) {
		let snapshots = load_json_test(file_path);

		for snapshot in snapshots {
			self.inject(snapshot.initial);

			for cycle in &snapshot.cycles {
				let MemoryState(addr, data, _description) = cycle;
				self.cpu.write(*addr, data.unwrap_or(0));
			}

			if let Err(e) = self.cpu.step() {
				panic!("{}", e);
			}

			if self.compare_state(&snapshot.final_) {
				let m = format!("Test: {} OK", snapshot.name);
				println!("{}\n", m.bold().green());
			} else {
				let m = format!("Test: {} FAILED", snapshot.name);
				println!("{}\n", m.bold().red());

				println!("Snapshot Final:\n{}", snapshot);
				println!("CPU Final State:\n{}", self.cpu);

				panic!("{}\n", m.bold().red());
			}
		}
	}
}

fn load_json_test(file_path: String) -> Vec<Snapshot> {
	let file = File::open(file_path).expect("\nErro ao abrir o arquivo JSON\nTest file not found.\n");
	let buffer = BufReader::new(file);

	let snapshots: Vec<Snapshot> = serde_json::from_reader(buffer)
		.expect("\nERRO: serde_json - can't convert json to Snapshot struct.\n");

	return snapshots;
}
