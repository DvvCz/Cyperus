#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "papyrus.pest"]
pub struct PapyrusParser;

mod ast;

fn main() {
	let out = ast::parse_papyrus("int x = 5");
	println!("{out:#?}");
}