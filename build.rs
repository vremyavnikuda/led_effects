use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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