mod bus;
mod cpu;
mod disassembler;
mod emuchan;
mod tests;
mod utils;

use disassembler::{disassemble, parse_from_file};

use tests::sm83::SM83;

use emuchan::EmuChan;

fn main() {
	// let mut sm83 = SM83::new();
	// sm83.run_test("./roms/json_tests/0e.json".to_string());
	// panic!("aa");

	let instructions = parse_from_file("./src/disassembler/instructions.json");

	let mut emuchan = EmuChan::new();
	disassemble(0x00, &emuchan.bus.memory, &instructions, 256);
	emuchan.run();

	// let mut emuchan = EmuChan::new();
	// emuchan.run();
}
