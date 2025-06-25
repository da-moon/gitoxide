//! Simple type alias used throughout the crate for convenience.
//!
//! Using [`miette::Result`] makes all our functions return rich diagnostic
//! errors without repeating the full type signature.
pub type Result<T> = miette::Result<T>;
