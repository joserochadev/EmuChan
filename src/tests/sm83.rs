use serde::Deserialize;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

use crate::core::bus::BUS;
use crate::core::cpu::CPU;

#[derive(Debug, Clone, Copy, Deserialize)]
struct RegisterState {
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

impl fmt::Display for RegisterState {
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

#[derive(Debug, Deserialize)]
struct Snapshot {
	name: String,
	initial: RegisterState,
	final_: RegisterState,
	cycles: Vec<MemoryState>,
}

pub struct TestReport {
	pub passed: usize,
	pub failed: usize,
	pub total: usize,
	pub failures: Vec<TestFailure>,
}

pub struct TestFailure {
	pub test_name: String,
	pub expected: String,
	pub actual: String,
}

pub struct SM83 {
	bus: Arc<Mutex<BUS>>,
	cpu: CPU,
}

impl SM83 {
	pub fn new() -> Self {
		let bus = Arc::new(Mutex::new(BUS::new()));
		let cpu = CPU::new(Arc::clone(&bus));

		Self { bus, cpu }
	}

	fn inject(&mut self, initial_state: RegisterState) {
		let RegisterState {
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
			ime,
			ie,
			..
		} = initial_state;

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
		self.cpu.reg.ime = ime;

		if let Some(ie_val) = ie {
			self.cpu.reg.ie = ie_val;
		}
	}

	fn retrieve(&self) -> RegisterState {
		RegisterState {
			a: self.cpu.reg.a,
			b: self.cpu.reg.b,
			c: self.cpu.reg.c,
			d: self.cpu.reg.d,
			e: self.cpu.reg.e,
			f: self.cpu.reg.f,
			h: self.cpu.reg.h,
			l: self.cpu.reg.l,
			pc: self.cpu.reg.pc,
			sp: self.cpu.reg.sp,
			ime: self.cpu.reg.ime,
			ie: Some(self.cpu.reg.ie),
		}
	}

	fn compare_state(&self, expected: &RegisterState) -> bool {
		let actual = self.retrieve();
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

	pub fn run_test(&mut self, file_path: String) -> Result<TestReport, String> {
		let snapshots = match load_json_test(&file_path) {
			Ok(snaps) => snaps,
			Err(e) => return Err(format!("Failed to load test file: {}", e)),
		};

		let mut passed = 0;
		let mut failed = 0;
		let mut failures = Vec::new();

		for snapshot in snapshots {
			// Reset CPU state
			self.inject(snapshot.initial);

			// Write memory cycles
			for cycle in &snapshot.cycles {
				let MemoryState(addr, data, _description) = cycle;
				if let Some(value) = data {
					self.cpu.write(*addr, *value);
				}
			}

			// Execute one instruction
			if let Err(e) = self.cpu.step() {
				failures.push(TestFailure {
					test_name: snapshot.name.clone(),
					expected: format!("{}", snapshot.final_),
					actual: format!("CPU Error: {}", e),
				});
				failed += 1;
				continue;
			}

			// Compare results
			if self.compare_state(&snapshot.final_) {
				passed += 1;
			} else {
				failures.push(TestFailure {
					test_name: snapshot.name.clone(),
					expected: format!("{}", snapshot.final_),
					actual: format!("{}", self.retrieve()),
				});
				failed += 1;
			}
		}

		let total = passed + failed;

		Ok(TestReport {
			passed,
			failed,
			total,
			failures,
		})
	}
}

fn load_json_test(file_path: &str) -> Result<Vec<Snapshot>, String> {
	let file =
		File::open(file_path).map_err(|e| format!("Failed to open file '{}': {}", file_path, e))?;

	let buffer = BufReader::new(file);

	let snapshots: Vec<Snapshot> =
		serde_json::from_reader(buffer).map_err(|e| format!("Failed to parse JSON: {}", e))?;

	Ok(snapshots)
}
