//! Tree-sitter based code parsing for RPG entity and dependency extraction.
//!
//! Supports Python, Rust, TypeScript, JavaScript, Go, Java, C, and C++.
//! Extracts functions, classes, methods, traits, import statements,
//! function calls, and inheritance relationships.

pub mod deps;
pub mod entities;
pub mod languages;
pub mod treesitter;

use entities::RawEntity;
use languages::Language;
use rayon::prelude::*;
use std::path::PathBuf;

/// Parse multiple source files in parallel using rayon.
/// Each entry is `(relative_path, source_code)`.
/// Language is determined per-file from extension; files with unrecognized extensions are skipped.
pub fn parse_files_parallel(files: Vec<(PathBuf, String)>) -> Vec<RawEntity> {
    files
        .into_par_iter()
        .flat_map(|(rel_path, source)| {
            let lang = rel_path
                .extension()
                .and_then(|e| e.to_str())
                .and_then(Language::from_extension);
            match lang {
                Some(language) => entities::extract_entities(&rel_path, &source, language),
                None => Vec::new(),
            }
        })
        .collect()
}
