use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Parsing error: {0}")]
	Parsing(#[from] crate::parse::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
