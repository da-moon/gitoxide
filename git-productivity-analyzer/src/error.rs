use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error(transparent)]
pub struct Error(#[from] anyhow::Error);

pub type Result<T> = std::result::Result<T, Error>;
