#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "papyrus.pest"]
pub struct PapyrusParser;

mod error;
mod parse;

fn wrapper() -> error::Result<()> {
	let src = "x = 55[ff].xx(22, 33)";

	let out = parse::parse_module(src)?;
	println!("{out:#?}");

	Ok(())
}

fn main() {
	if let Err(err) = wrapper() {
		panic!("{err}");
	}
}
