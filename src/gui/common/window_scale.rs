#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WindowScale {
	X1, // 1x
	X2, // 2x
	X3, // 3x
	X4, // 4x
}

impl WindowScale {
	pub fn as_factor(&self) -> f32 {
		match self {
			WindowScale::X1 => 1.0,
			WindowScale::X2 => 2.0,
			WindowScale::X3 => 3.0,
			WindowScale::X4 => 4.0,
		}
	}
}
