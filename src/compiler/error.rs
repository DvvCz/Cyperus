use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Validation Error: {0}")]
	Validation(String),
}

pub type Result<T> = std::result::Result<T, Error>;
