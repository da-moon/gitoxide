//! Utilities for displaying frecency scores.
//!
//! This module keeps text and JSON printing logic separate from the main
//! analyzer to keep that file small and focused.

use super::Analyzer;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct PathScore {
    path: String,
    score: f64,
}

fn to_path_scores(list: &[(PathBuf, f64)]) -> Vec<PathScore> {
    list.iter()
        .map(|(p, s)| PathScore {
            path: p.display().to_string(),
            score: *s,
        })
        .collect()
}

#[derive(Serialize)]
struct PathList {
    paths: Vec<String>,
}

fn to_path_list(list: &[(PathBuf, f64)]) -> PathList {
    PathList {
        paths: list.iter().map(|(p, _)| p.display().to_string()).collect(),
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
            crate::sdk::print_json_or(self.globals.json, &to_path_list(scores), print_plain);
        } else {
            crate::sdk::print_json_or(self.globals.json, &to_path_scores(scores), print_plain);
        }
    }
}
