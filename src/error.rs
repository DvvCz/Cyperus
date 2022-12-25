use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Error while parsing: {0}")]
	Parsing(#[from] pest::error::Error<crate::ast::Rule>),

	#[error("Expected {0}, but got {1}")]
	Expected(&'static str, &'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
