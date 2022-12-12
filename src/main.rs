#[macro_use] extern crate pest_derive;
use pest::Parser;

#[derive(Parser)]
#[grammar = "papyrus.pest"]
struct PapyrusParser;

fn main() {
	match PapyrusParser::parse(Rule::module, include_str!("script.psc")) {
		Ok(_ast) => println!("Ok."),
		Err(why) => println!("Failed: {why}")
	}
}
