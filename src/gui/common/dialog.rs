use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::tests::sm83::SM83;

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

pub fn open_json_test_dialog(test_sender: Sender<String>) {
	let current_directory = std::env::current_dir().unwrap_or_else(|_| ".".into());

	std::thread::spawn(move || {
		let test_file_path = rfd::FileDialog::new()
			.add_filter("JSON Test File", &["json"])
			.set_directory(&current_directory)
			.pick_file();

		if let Some(path) = test_file_path {
			let mut test_runner = SM83::new();

			test_sender
				.send(format!("Teste selecionado: {}", path.display()))
				.unwrap();
			test_sender
				.send("Executando testes...".to_string())
				.unwrap();

			let result_message = test_runner.run_test(path.display().to_string());

			if let Err(e) = result_message {
				test_sender.send(e).unwrap();
			} else {
				test_sender
					.send("\nAll tests passed!\n".to_uppercase())
					.unwrap();
			}
		}
	});
}
