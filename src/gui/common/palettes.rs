use eframe::egui;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ColorPalette {
	Default,
	Classic,
	Greyscale,
	Chocolate,
}

pub fn get_colors(palette: ColorPalette) -> [egui::Color32; 4] {
	match palette {
		ColorPalette::Default => [
			egui::Color32::from_rgb(223, 248, 208),
			egui::Color32::from_rgb(136, 192, 112),
			egui::Color32::from_rgb(54, 103, 82),
			egui::Color32::from_rgb(7, 24, 33),
		],
		ColorPalette::Classic => [
			egui::Color32::from_rgb(155, 188, 15),
			egui::Color32::from_rgb(139, 172, 15),
			egui::Color32::from_rgb(48, 98, 48),
			egui::Color32::from_rgb(15, 56, 15),
		],
		ColorPalette::Greyscale => [
			egui::Color32::from_gray(255),
			egui::Color32::from_gray(170),
			egui::Color32::from_gray(85),
			egui::Color32::from_gray(0),
		],
		ColorPalette::Chocolate => [
			egui::Color32::from_rgb(227, 202, 165),
			egui::Color32::from_rgb(184, 153, 114),
			egui::Color32::from_rgb(112, 85, 59),
			egui::Color32::from_rgb(41, 29, 20),
		],
	}
}
