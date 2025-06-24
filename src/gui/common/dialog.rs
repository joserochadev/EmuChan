use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub fn open_rom_dialog(rom_sender: Sender<PathBuf>) {
	std::thread::spawn(move || {
		let current_directory = std::env::current_dir().unwrap_or_else(|_| ".".into());

		let file_dialog = rfd::FileDialog::new()
			.add_filter("Game Boy ROM", &["gb", "gbc"])
			.set_directory(&current_directory);

		if let Some(path) = file_dialog.pick_file() {
			let _ = rom_sender.send(path);
		}
	});
}
