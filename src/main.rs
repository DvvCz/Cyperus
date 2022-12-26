use pest::Parser;

#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "papyrus.pest"]
pub struct PapyrusParser;

mod parse;
mod error;

fn wrapper() -> error::Result<()> {
	let src = "
		foo[5][3].bar = 55
	";

	let out = parse::parse_module(src)?;
	println!("{out:#?}");

	Ok(())
}


fn main() {
	if let Err(err) = wrapper() {
		panic!("{err}");
	}
}