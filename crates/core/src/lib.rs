//! Mati Mati conjugation engine.
//!
//! Pure Rust with no I/O, so it compiles both natively (site builder, tests)
//! and to WebAssembly (runs in the visitor's browser).

pub mod conjugate;
pub mod chinese;
pub mod english;
pub mod itza;
pub mod lacandon;
pub mod mopan;
pub mod nahuatl;
pub mod otomi;
pub mod purhepecha;
pub mod spanish;
pub mod wixarika;
pub mod yucatec;
pub mod zoque;

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
    /// Root + aspect + tense + mode + person suffixes. See `purhepecha`.
    Purhepecha(purhepecha::Conjugation),
    /// Aspect + set A/B + status, with three intransitive completives
    /// (j-, -naj-, -laj-). See `yucatec`.
    Yucatec(yucatec::Conjugation),
    /// Like Yucatec, with set A joined to the verb and uch for activity
    /// verbs. See `mopan`.
    Mopan(mopan::Conjugation),
    /// Southern Lacandon: set A joined after k- / t-, set B, -Vr / -Vk
    /// harmony. See `lacandon`.
    Lacandon(lacandon::Conjugation),
    /// Person prefixes and enclitics, -tam- / -yaj- plurals, -pa / -u. See
    /// `zoque`.
    Zoque(zoque::Conjugation),
    /// Subject + assertive + prefix chain + stem + tense/aspect, with
    /// suppletive stems. See `wixarika`.
    Wixarika(wixarika::Conjugation),
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
            Engine::Purhepecha(c) => c.conjugate_with(verb, opts),
            Engine::Yucatec(c) => c.conjugate_with(verb, opts),
            Engine::Mopan(c) => c.conjugate_with(verb, opts),
            Engine::Lacandon(c) => c.conjugate_with(verb, opts),
            Engine::Zoque(c) => c.conjugate_with(verb, opts),
            Engine::Wixarika(c) => c.conjugate_with(verb, opts),
        }
    }

    /// Dictionary forms of the verbs in the lexicon (used to check the data).
    pub fn lexicon_words(&self) -> Vec<String> {
        match self {
            Engine::Otomi(c) => c.lexicon.iter().map(|e| e.root.clone()).collect(),
            Engine::Itza(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
            Engine::Nahuatl(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
            Engine::Purhepecha(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
            Engine::Yucatec(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
            Engine::Mopan(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
            Engine::Lacandon(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
            Engine::Zoque(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
            Engine::Wixarika(c) => c.lexicon.iter().map(|e| e.entry.clone()).collect(),
        }
    }
}

/// The equivalent of a form in the page's language: Spanish, or English
/// when `opts.lang` is "en" (the English pages give `es` specs and
/// meanings in English, see `data/languages/en/`), or Chinese when it is
/// "zh" (the Spanish specs with Chinese meanings, see `chinese`).
pub fn gloss(
    opts: &conjugate::Options,
    meaning: &str,
    es: &spanish::Es,
    person: usize,
    negative: bool,
) -> Option<String> {
    if opts.english() {
        english::render(meaning, es, person, negative)
    } else if opts.chinese() {
        chinese::render(meaning, es, person, negative)
    } else {
        spanish::render(meaning, es, person, negative)
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
