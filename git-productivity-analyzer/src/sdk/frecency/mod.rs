//! SDK types implementing the frecency analysis logic.
//!
//! The [`Analyzer`] walks the repository history using `gix` and ranks files by
//! how recently and frequently they were changed.

pub mod analyzer;

pub use analyzer::Options;
