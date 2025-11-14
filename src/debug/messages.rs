use std::path::PathBuf;

// Messages send from frontend to emulator core
#[derive(Debug, Clone)]
pub enum EmulatorCommand {
	LoadRom(PathBuf),
	TogglePause,
	Stop,
	Reset,
	SetSpeed(f32), // Define emulations speed (1.0 = normal, 2.0 = 2x)
	SaveState(PathBuf),
	LoadState(PathBuf),
	JoypadInput(JoypadButton, bool), // (button, isPressed?)

	Debug(DebugCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoypadButton {
	A,
	B,
	Start,
	Select,
	Up,
	Down,
	Left,
	Right,
}

// Messages send from emulation core to frontend
#[derive(Debug, Clone)]
pub enum EmulatorEvent {
	RomLoaded { title: String, size: usize }, // ROM Loaded successfully
	RomLoadError(String),                     // Error to load ROM
	FrameReady(Box<Vec<u8>>),                 // Rendered frame (160X144 pixels)
	StateChanged(EmulatorState),
	DebugInfo { fps: f32, cycles_per_frame: u64 },
	Error(String),

	Debug(DebugEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmulatorState {
	Idle,
	Running,
	Paused,
	Stoped,
}

#[derive(Debug, Clone)]
pub enum DebugCommand {
	StepInstruction,
	StepFrame,
	Continue, // Continue execution
	AddBreakpoint(u16),
	RemoveBreakpoint(u16),
	ToggleBreakpoint(u16),
	ClearBreakpoints,
	ReadMemory(u16, usize),
	RequestCpuState,
	RequestDisassembly {
		pc: u16,
		before: usize,
		after: usize,
	},
	SetTrace(bool),
	RunSM83Test(PathBuf),
}

#[derive(Debug, Clone)]
pub enum DebugEvent {
	BreakpoitHit {
		address: u16,
	},
	CpuState(CpuDebugState),
	MemoryRead {
		address: u16,
		data: Vec<u8>,
	},
	DisassemblyUpdate(DisassemblyView),
	BreakpointUpdated(Vec<u16>),
	InstructionExecuted {
		address: u16,
		mnemonic: String,
		operands: String,
		cycles: u8,
	},
	TestStarted {
		test_name: String,
	},
	TestResult {
		test_name: String,
		passed: bool,
		message: String,
	},
}

#[derive(Debug, Clone)]
pub struct CpuDebugState {
	pub a: u8,
	pub b: u8,
	pub c: u8,
	pub d: u8,
	pub e: u8,
	pub f: u8,
	pub h: u8,
	pub l: u8,
	pub pc: u16,
	pub sp: u16,
	pub flag_z: bool,
	pub flag_n: bool,
	pub flag_h: bool,
	pub flag_c: bool,
	pub ime: u8,
	pub total_cycles: usize,
}

#[derive(Debug, Clone)]
pub struct DisassemblyView {
	pub current_pc: u16,
	pub instructions: Vec<DisassemblyLine>,
}

#[derive(Debug, Clone)]
pub struct DisassemblyLine {
	pub address: u16,
	pub bytes: Vec<u8>,
	pub mnemonic: String,
	pub operands: String,
	pub cycles: u8,
}
