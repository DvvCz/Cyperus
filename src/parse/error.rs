use super::Rule;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Error while parsing: {0}")]
	Parsing(#[from] pest::error::Error<Rule>),

	#[error("Expected {0:?}, but got {1:?} (chars {2}-{3})")]
	Expected(Rule, Rule, usize, usize),

	#[error("Expected {0:?}, but got end of input.")]
	UnexpectedEOI(Rule),
}

pub type Result<'a, T> = std::result::Result<T, Error>;
