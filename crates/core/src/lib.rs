//! Mati Mati conjugation engine.
//!
//! Pure Rust with no I/O, so it compiles both natively (site builder, tests)
//! and to WebAssembly (runs in the visitor's browser).

pub mod conjugate;
pub mod itza;
pub mod nahuatl;
pub mod otomi;
pub mod spanish;

use serde::{Deserialize, Serialize};

/// A language's `conjugation` block. The `engine` field picks the rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "engine", rename_all = "lowercase")]
pub enum Engine {
    /// Proclitic + root (hñähñu). See `conjugate` and `otomi`.
    Otomi(conjugate::Conjugation),
    /// Aspect + Set A/B person markers + status suffixes. See `itza`.
    Itza(itza::Conjugation),
    /// Subject + object prefixes, tense suffixes, four past classes.
    /// See `nahuatl`.
    Nahuatl(nahuatl::Conjugation),
}

impl Engine {
    pub fn conjugate(&self, verb: &str) -> Result<conjugate::Result, conjugate::Error> {
        self.conjugate_with(verb, &conjugate::Options::default())
    }

    /// Conjugates with extra information typed by the user (meaning, type).
    pub fn conjugate_with(
        &self,
        verb: &str,
        opts: &conjugate::Options,
    ) -> Result<conjugate::Result, conjugate::Error> {
        match self {
            Engine::Otomi(c) => c.conjugate_with(verb, opts),
            Engine::Itza(c) => c.conjugate_with(verb, opts),
            Engine::Nahuatl(c) => c.conjugate_with(verb, opts),
        }
    }

    /// Dictionary forms of the verbs in the lexicon (used to check the data).
    pub fn lexicon_words(&self) -> Vec<String> {
        match self {
            Engine::Otomi(c) => c.lexicon.iter().map(|e| e.root.clone()).collect(),
            Engine::Itza(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
            Engine::Nahuatl(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
        }
    }
}

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
