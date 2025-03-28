# EmuChan 🕹️

This is a **hobby project** to learn Rust and explore hardware emulation. The goal is to emulate the original GameBoy—not to create a final product, but as a way to learn and have fun in the process.

> **Note:** This project is not intended to be a production tool. The focus is on learning and exploring hardware emulation in a hands-on way.

## 🚀 Features

✅ Emulation of the GameBoy's Z80-like CPU  
✅ Implementation of the memory bus and register mapping  
✅ Rendering via PPU (in development)  
✅ Support for the GameBoy boot ROM (in development)  
✅ Compatibility with games and hardware tests (in development)  
✅ CLI to run the emulator, execute tests, or disassemble memory sections

## 📚 References

These resources have been essential for building the emulator:

- 📚 [PanDocs - GameBoy Specifications](https://gbdev.io/pandocs/Specifications.html)
- 🛠️ [Mooneye - Testing and debugging tools](https://github.com/Gekkio/mooneye-gb)
- 🔢 [GameBoy OpCode Table](https://izik1.github.io/gbops/index.html)
- 🏁 [Disassembly of the GameBoy Boot ROM](https://gist.github.com/drhelius/6063288)
- 🏁 [sm83 - GameBoy CPU JSON tests](https://github.com/SingleStepTests/sm83)

## 💻 How to Run

1️⃣ **Clone the repository:**

```sh
git clone https://github.com/joserochadev/EmuChan.git
cd EmuChan
```

2️⃣ **Compile the project:**

```sh
cargo build --release
```

3️⃣ Run the CLI:

The CLI allows you to:

- Run the emulator
- Execute a specific test
- Disassemble a memory section

**Run the emulator with a ROM:**

```sh
cargo run --release -- run path/to/rom.gb
```

**Run a specific test from a JSON file:**

```sh
cargo run --release -- test path/to/test.json
```

**Disassemble a section of memory:**

```sh
cargo run --release -- disassembler 0x0000 256
```

(This disassembles 256 bytes starting at address 0x0000.)

**Show help message:**

```sh
cargo run --release -- help

```

## 📌 Contributions

This project is not commercial and is intended for learning purposes. However, if you have suggestions or improvements, feel free to open an issue or submit a pull request! 🚀
