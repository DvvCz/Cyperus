#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "papyrus.pest"]
pub struct PapyrusParser;

mod ast;
mod error;

fn main() {
	let src = "
		int[] x = new int[5];
		int function test(string foo = 5)
			return 5
		endfunction
	";

	let out = ast::parse_module(src);
	println!("{out:#?}");
}
