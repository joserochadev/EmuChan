use std::collections::HashSet;

pub struct Debugger {
	breakpoints: HashSet<u16>,
	trace_enable: bool,
}

impl Debugger {
	pub fn new() -> Self {
		Self {
			breakpoints: HashSet::new(),
			trace_enable: false,
		}
	}

	pub fn add_breakpoint(&mut self, address: u16) -> bool {
		self.breakpoints.insert(address)
	}

	pub fn remove_breakpoint(&mut self, address: u16) -> bool {
		self.breakpoints.remove(&address)
	}

	pub fn toggle_breakpoint(&mut self, address: u16) {
		if self.breakpoints.contains(&address) {
			self.breakpoints.remove(&address);
		} else {
			self.breakpoints.insert(address);
		}
	}

	pub fn clear_breakpoints(&mut self) {
		self.breakpoints.clear();
	}

	pub fn get_breakpoints(&self) -> Vec<u16> {
		self.breakpoints.iter().copied().collect()
	}

	pub fn has_breakpoint(&self, address: u16) -> bool {
		self.breakpoints.contains(&address)
	}

	pub fn set_trace(&mut self, enable: bool) {
		self.trace_enable = enable;
	}

	pub fn is_trace_enable(&self) -> bool {
		self.trace_enable
	}

	pub fn should_break(&self, pc: u16) -> bool {
		self.breakpoints.contains(&pc)
	}
}
