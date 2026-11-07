//! Loads language definitions from `data/languages/*.json`.
//!
//! Each file is kept as raw JSON (so the frontend gets every field that is
//! written there) and, when it has a `conjugation` block, also parsed into
//! typed structures the conjugation engine can use.

use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::conjugate::Conjugation;

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

pub struct Language {
    pub summary: Summary,
    pub raw: Value,
    pub conjugation: Option<Conjugation>,
}

pub struct Catalog {
    languages: BTreeMap<String, Language>,
}

impl Catalog {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let mut languages = BTreeMap::new();
        let entries = fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
        for entry in entries {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let raw: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            let summary: Summary =
                serde_json::from_value(raw.clone()).map_err(|e| format!("{}: {e}", path.display()))?;
            let conjugation = match raw.get("conjugation") {
                Some(c) => Some(
                    serde_json::from_value(c.clone()).map_err(|e| format!("{} conjugation: {e}", path.display()))?,
                ),
                None => None,
            };
            languages.insert(summary.slug.clone(), Language { summary, raw, conjugation });
        }
        Ok(Self { languages })
    }

    pub fn summaries(&self) -> Vec<Summary> {
        let mut list: Vec<Summary> = self.languages.values().map(|l| l.summary.clone()).collect();
        list.sort_by_key(|s| s.order);
        list
    }

    pub fn get(&self, slug: &str) -> Option<&Language> {
        self.languages.get(slug)
    }
}
