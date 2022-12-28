use super::Rule;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Error while parsing: {0}")]
	Parsing(#[from] pest::error::Error<Rule>),

	#[error("Expected {expected:?}, but got {got:?} at line {}, col {}", trace.0, trace.1)]
	Expected { expected: Rule, got: Rule, trace: (usize, usize) },

	#[error("Expected {0:?}, but got end of input.")]
	UnexpectedEOI(Rule),
}

pub type Result<'a, T> = std::result::Result<T, Error>;
