use thiserror::Error;
use super::Rule;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Error while parsing: {0}")]
	Parsing(#[from] pest::error::Error<Rule>),

	#[error("Expected {0:?}, but got {1:?}")]
	Expected(Rule, Rule),
}

pub type Result<T> = std::result::Result<T, Error>;
