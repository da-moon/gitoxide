//! Utilities for displaying frecency scores.
//!
//! This module keeps text and JSON printing logic separate from the main
//! analyzer to keep that file small and focused.

use super::Analyzer;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
/// Helper struct to make printing and JSON serialization uniform.
struct SerializableScores {
    scores: Vec<(String, f64)>,
}

impl SerializableScores {
    fn from(list: &[(PathBuf, f64)]) -> Self {
        Self {
            scores: list.iter().map(|(p, s)| (p.display().to_string(), *s)).collect(),
        }
    }
}

#[derive(Serialize)]
/// Simple JSON representation for path-only output.
struct SerializablePaths {
    paths: Vec<String>,
}

impl SerializablePaths {
    fn from(list: &[(PathBuf, f64)]) -> Self {
        Self {
            paths: list.iter().map(|(p, _)| p.display().to_string()).collect(),
        }
    }
}

impl Analyzer {
    /// Print the computed scores either as text table or JSON.
    pub fn print_scores(&self, scores: &[(PathBuf, f64)]) {
        let print_plain = || {
            if self.opts.path_only {
                for (path, _) in scores {
                    println!("{}", path.display());
                }
            } else {
                for (path, score) in scores {
                    println!("{:.4}\t{}", score, path.display());
                }
            }
        };

        if self.opts.path_only {
            crate::sdk::print_json_or(self.globals.json, &SerializablePaths::from(scores), print_plain);
        } else {
            crate::sdk::print_json_or(self.globals.json, &SerializableScores::from(scores), print_plain);
        }
    }
}
