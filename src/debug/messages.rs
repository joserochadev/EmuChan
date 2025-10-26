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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmulatorState {
	Idle,
	Running,
	Paused,
	Stoped,
}
