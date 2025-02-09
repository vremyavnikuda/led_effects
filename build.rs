use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// The main function for the build script.
///
/// This function performs the following operations:
/// 1. Retrieves the build directory path from the `OUT_DIR` environment variable.
/// 2. Copies the `memory.x` file into the build directory.
/// 3. Instructs the Rust compiler to search for `memory.x` in the build directory during linking.
/// 4. Sets up a trigger to rerun the build script if `memory.x` changes.
fn main() {
	// Получаем путь к каталогу сборки
	let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

	// Копируем memory.x в каталог сборки
	File::create(out.join("memory.x"))
		.unwrap()
		.write_all(include_bytes!("memory.x"))
		.unwrap();

	// Указываем линкеру искать memory.x в каталоге сборки
	println!("cargo:rustc-link-search={}", out.display());

	// Повторная сборка при изменении memory.x
	println!("cargo:rerun-if-changed=memory.x");
}