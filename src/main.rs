mod bus;
mod cartridge;
mod cpu;
mod disassembler;
mod emuchan;
mod ppu;
mod tests;
mod ui;
mod utils;

use disassembler::{disassemble, parse_from_file};

use tests::sm83::SM83;

use clap::{Parser, Subcommand};
use env_logger;
use std::env;

use emuchan::EmuChan;

#[derive(Parser)]
#[command(
	bin_name = "cargo run --",
	author = "joserochadev",
	version = "0.1.0",
	about = "EmuChan Emulator CLI",
	long_about = "This CLI allows you to run the emulator, execute specific tests, or disassemble a memory section."
)]
struct CLI {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Starts the emulator and runs the loaded ROM.
	///
	/// Example:
	/// ```
	/// cargo run -- run path/to/rom.gb
	/// ```
	RUN { path: String },

	/// Runs a specific test from a JSON file.
	///
	/// Example:
	/// ```
	/// cargo run -- test ./roms/json_test/0e.json
	/// ```
	TEST { path: String },

	/// Disassembles a section of memory and prints the instructions.
	///
	/// Example:
	/// ```
	/// cargo run -- disassemble 0x100 256
	/// ```
	DISASSEMBLER {
		/// Starting memory address (e.g., 0x100)
		start: String,

		/// Number of bytes to disassemble
		length: usize,
	},
}

fn main() {
	//env::set_var("RUST_LOG", "trace"); // defice RUST_LOG env

	env_logger::init(); // Initialize logger

	let cli = CLI::parse();

	match cli.command {
		Commands::RUN { path } => {
			println!("🔄 Starting the emulator...");
			let mut emuchan = EmuChan::new(Some(path));
			emuchan.run();
		}

		Commands::TEST { path } => {
			println!("🔬 Running test: {}", path);
			let mut sm83 = SM83::new();
			sm83.run_test(path);
		}

		Commands::DISASSEMBLER { start, length } => {
			let start_num = usize::from_str_radix(start.trim_start_matches("0x"), 16)
				.expect("Invalid hexadecimal number");

			println!("🛠 Disassembling memory from 0x{:X} to 0x{:X}...", start_num, start_num + length);

			let start_addr = u16::from_str_radix(start.trim_start_matches("0x"), 16)
				.expect("Invalid hexadecimal number");

			let instructions = parse_from_file("./src/disassembler/instructions.json");

			let emuchan = EmuChan::new(None);
			disassemble(start_addr as usize, &emuchan.bus.lock().unwrap().memory, &instructions, length);
		}
	}
}
