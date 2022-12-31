#[derive(Parser)]
#[grammar = "parser/papyrus.pest"]
pub struct PestParser;

pub(crate) mod ast;
mod error;
mod expression;
mod statement;

use std::borrow::Cow;

use ast::{Ast, Expression, Type, ScriptInfo, Statement};
pub use error::Error;
use pest::{
	iterators::{Pair, Pairs},
	Parser,
};

use self::statement::ParseStatement;

type Result<T> = error::Result<T>;

trait PestWalker {
	fn expect_rule(&mut self, rule: Rule) -> Result<Pair<Rule>>;
	fn opt_rule(&mut self, rule: Rule) -> Option<Pair<Rule>>;
}

/// All of these functions assume they are on a node with the correct matching [Rule].
/// Should only use these after using [PestWalker::expect_rule].
pub(crate) trait PestNode {
	fn ident(self) -> String;
	fn ty(self) -> Type;
}

impl<'a> PestNode for Pair<'a, Rule> {
	#[inline(always)]
	fn ident(self) -> String {
		self.as_str().to_owned()
	}

	#[inline(always)]
	fn ty(self) -> Type {
		let mut inner = self.into_inner();

		let frag = inner.next().unwrap().as_str();
		let is_array = inner.peek().is_some();

		Type::new(Cow::Owned(frag.to_owned()), is_array)
	}
}

impl<'a> PestWalker for Pairs<'a, Rule> {
	fn expect_rule(&mut self, expecting: Rule) -> Result<Pair<Rule>> {
		match self.peek() {
			Some(pair) => {
				let got = pair.as_rule();
				if got == expecting {
					self.next();
					Ok(pair)
				} else {
					Err(Error::Expected {
						expected: expecting,
						got,
						trace: pair.as_span().start_pos().line_col(),
					})
				}
			}
			None => Err(Error::UnexpectedEOI(expecting)),
		}
	}

	fn opt_rule(&mut self, expecting: Rule) -> Option<Pair<Rule>> {
		match self.peek() {
			Some(pair) if pair.as_rule() == expecting => {
				self.next();
				Some(pair)
			}
			_ => None,
		}
	}
}

pub fn parse_module(source: impl AsRef<str>) -> Result<Ast> {
	let source = source.as_ref();
	let pairs = PestParser::parse(Rule::module, source)?;

	let mut statements = vec![];
	let mut script_info = ScriptInfo::default();

	for item in pairs {
		match item.as_rule() {
			Rule::header => {
				let mut inner = item.into_inner();
				script_info.script_name = inner.expect_rule(Rule::ident)?.ident();
				script_info.extended_type = inner.opt_rule(Rule::r#type).map(PestNode::ty);
				script_info.is_conditional = inner.peek().is_some();
			}

			Rule::body => {
				statements.extend(item.body()?);
			}

			Rule::EOI => (),
			unknown => todo!("parse_module {unknown:#?} {}", item.as_str()),
		}
	}

	Ok(Ast {
		script_info,
		statements,
	})
}
