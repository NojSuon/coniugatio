//! Mati Mati conjugation engine.
//!
//! Pure Rust with no I/O, so it compiles both natively (site builder, tests)
//! and to WebAssembly (runs in the visitor's browser).

pub mod conjugate;
pub mod otomi;

use serde::{Deserialize, Serialize};

/// The fields every `data/languages/*.json` file must have. Used on the home
/// page cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub slug: String,
    pub name: String,
    pub endonym: String,
    pub variant: String,
    pub region: String,
    /// `"available"` or `"soon"`.
    pub status: String,
    pub order: u32,
    /// Mati Mati colour series used for the language card and page.
    pub series: String,
    pub summary: String,
}
