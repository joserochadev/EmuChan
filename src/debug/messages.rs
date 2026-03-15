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

	RequestPpuState,
	RequestTileData {
		tile_index: u8,
	},
	RequestTileMap {
		map_select: bool,
	}, // false = 0x9800, true = 0x9C00
	RequestVramBuffer,
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

	PpuState(PpuDebugState),
	TileData(TileDebugData),
	TileMapData(TileMapDebugData),
	VramBufferData(VramDebugData),
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

#[derive(Debug, Clone)]
pub struct PpuDebugState {
	pub lcd_enable: bool,
	pub window_tile_map: bool,
	pub window_enable: bool,
	pub bg_window_tile_data: bool,
	pub bg_tile_map: bool,
	pub obj_size: bool,
	pub obj_enable: bool,
	pub bg_window_enable: bool,

	pub mode: u8, // 0-3
	pub lyc_ly_flag: bool,
	pub mode0_interrupt: bool,
	pub mode1_interrupt: bool,
	pub mode2_interrupt: bool,
	pub lyc_interrupt: bool,

	pub scy: u8,
	pub scx: u8,
	pub ly: u8,
	pub lyc: u8,
	pub wy: u8,
	pub wx: u8,

	pub bgp: u8,
	pub obp0: u8,
	pub obp1: u8,

	pub current_mode: String,
	pub cycles: u32,
}

#[derive(Debug, Clone)]
pub struct TileDebugData {
	pub tile_index: u8,
	pub pixels: Vec<u8>, // 8x8 = 64 pixels, valores 0-3
}

#[derive(Debug, Clone)]
pub struct TileMapDebugData {
	pub map_select: bool,
	pub tiles: Vec<u8>, // 32x32 = 1024 tile indices
}

#[derive(Debug, Clone)]
pub struct VramDebugData {
	pub buffer: Vec<u8>, // VRAM completa (8KB)
	pub width: usize,    // Largura da visualização
	pub height: usize,   // Altura da visualização
}
