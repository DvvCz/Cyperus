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