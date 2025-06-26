//! SDK types implementing the frecency analysis logic.
//!
//! The [`Analyzer`] walks the repository history using `gix` and ranks files by
//! how recently and frequently they were changed.

pub mod analyzer;
mod printer;

pub use analyzer::{Analyzer, Options, Order, DEFAULT_AGE_EXPONENT, DEFAULT_SIZE_PENALTY_REF};
