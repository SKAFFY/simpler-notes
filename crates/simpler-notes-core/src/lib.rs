pub mod note_model;
pub mod parser;
pub mod buffer;
pub mod diagnostics;
pub mod index;
pub mod persistence;
pub mod search;
pub mod vault;
pub mod watcher;

#[cfg(feature = "git")]
pub mod git;

pub use note_model::*;
