#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "papyrus.pest"]
pub struct PapyrusParser;

mod error;
mod ast;

fn main() {
	let src = "
		int[] x = new int[5];
		if True else endif
	";

	let out = ast::parse_module(src);
	println!("{out:#?}");
}