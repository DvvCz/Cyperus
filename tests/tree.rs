/*!
	Testing the AST builder with actual widely used Papyrus code.
*/

use cyperus::parse_module;

macro_rules! github {
	($path:literal) => {
		concat!("https://raw.githubusercontent.com/", $path, "/master/")
	};
}

#[test]
fn test_f4se() {
	macro_rules! p {
		($path:literal) => {
			concat!(github!("ianpatt/f4se"), $path)
		};
	}

	for url in [
		p!("scripts/vanilla/DogmeatIdles.psc"),
		p!("scripts/modified/InstanceData.psc"),
	] {
		let script = ureq::get(url).call().unwrap().into_string().unwrap();

		if let Err(why) = parse_module(script) {
			panic!("{url}: {why}");
		}
	}
}

#[test]
#[cfg(not(feature = "ci"))]
fn test_fallout_4() {
	let dir = std::env::var("FALLOUT_4_DIR").expect("FALLOUT_4_DIR not set");
	let dir = std::path::Path::new(&dir);

	let scripts_dir = dir.join("Data").join("Scripts").join("Source");
	for file in std::fs::read_dir(scripts_dir).unwrap() {
		let file = file.unwrap();
		let path = file.path();

		if path.is_file() {
			let script = std::fs::read_to_string(&path).unwrap();
			if let Err(why) = parse_module(&script) {
				panic!("{}: {why}", path.display());
			}
		}
	}
}

#[test]
#[cfg(not(feature = "ci"))]
fn test_skyrim() {
	let dir = std::env::var("SKYRIM_DIR").expect("SKYRIM_DIR not set");
	let dir = std::path::Path::new(&dir);

	let scripts_dir = dir.join("Data").join("Scripts").join("Source");
	for file in std::fs::read_dir(scripts_dir).unwrap() {
		let file = file.unwrap();
		let path = file.path();

		if path.is_file() {
			let script = std::fs::read_to_string(&path).unwrap();
			if let Err(why) = parse_module(&script) {
				panic!("{}: {why}", path.display());
			}
		}
	}
}
