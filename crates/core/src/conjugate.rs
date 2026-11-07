//! Clitic-based conjugation engine.
//!
//! A language lists its persons and paradigms (one clitic per person). Each
//! paradigm can say which root form a person takes — the primary root, or a
//! secondary form such as `"Y"` or `"BY"` (Y if it applies, otherwise B). The
//! language-specific morphology (see `otomi.rs`) builds those forms; a lexicon
//! of verbs from the dictionary overrides the rules when a verb is irregular.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{otomi, spanish};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: String,
    pub label: String,
    /// Whether this person takes the plural suffix.
    #[serde(default = "yes")]
    pub has_plural: bool,
    /// Spanish person for the translation: 0 = yo, 1 = tú, 2 = él/ella.
    /// The plural uses the matching plural person (nosotros, ustedes, ellos).
    #[serde(default)]
    pub es: Option<usize>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paradigm {
    pub id: String,
    pub group: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub forms: BTreeMap<String, String>,
    /// person id -> "Y" | "BY". Persons not listed use the primary root.
    #[serde(default)]
    pub stems: BTreeMap<String, String>,
    /// Secondary tenses take the n- form for N-verbs.
    #[serde(default)]
    pub secondary: bool,
    /// Spanish tense used for the translation.
    #[serde(default)]
    pub es: Option<spanish::Es>,
    /// Translate with "no".
    #[serde(default)]
    pub negative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconEntry {
    pub root: String,
    pub meaning: String,
    /// Spanish infinitive to conjugate for the translation ("comer",
    /// "reírse", "beber agua").
    #[serde(default)]
    pub es: Option<String>,
    /// Secondary form B, when the rule would get it wrong.
    #[serde(default)]
    pub b: Option<String>,
    /// Secondary form Y, when the rule would get it wrong.
    #[serde(default)]
    pub y: Option<String>,
    /// Takes n- in every person of the secondary tenses (§1.3).
    #[serde(default)]
    pub n: bool,
    /// Dictionary reference, e.g. "Pret. bi meni".
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conjugation {
    pub persons: Vec<Person>,
    pub paradigms: Vec<Paradigm>,
    #[serde(default)]
    pub lexicon: Vec<LexiconEntry>,
}

/// Extra information for a verb that isn't in the lexicon ("Conjuga tu
/// verbo"): its Spanish meaning and, optionally, its type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Options {
    /// Spanish infinitive used for the translation ("trabajar").
    #[serde(default)]
    pub es: Option<String>,
    /// Verb type, as listed in the language's `conjugator.types`
    /// (otomí: "n"; itza: "I", "T", "Tt"). Empty means "detect".
    #[serde(default)]
    pub kind: Option<String>,
}

impl Options {
    pub fn es(&self) -> Option<&str> {
        self.es.as_deref().map(str::trim).filter(|s| !s.is_empty())
    }

    pub fn kind(&self) -> Option<&str> {
        self.kind
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Row {
    pub person: String,
    pub clitic: String,
    pub singular: String,
    pub plural: Option<String>,
    /// Spanish equivalents, when the verb's meaning is known.
    pub es_singular: Option<String>,
    pub es_plural: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Table {
    pub id: String,
    pub group: String,
    pub name: String,
    pub description: String,
    pub rows: Vec<Row>,
}

#[derive(Debug, Serialize)]
pub struct Result {
    pub root: String,
    /// The verb is in the lexicon (forms checked against the dictionary).
    pub known: bool,
    pub meaning: Option<String>,
    pub note: Option<String>,
    pub tables: Vec<Table>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyVerb,
    InvalidVerb,
    /// The verb's class can't be told from the word (Itza).
    UnknownClass,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyVerb => write!(f, "Escribe la raíz de un verbo."),
            Error::InvalidVerb => write!(f, "La raíz solo puede llevar letras y apóstrofos."),
            Error::UnknownClass => write!(
                f,
                "No sabemos qué tipo de verbo es. Escríbelo como en el diccionario, con -Vl si es intransitivo (wenel, «dormir») o con -ik si es transitivo (ilik, «ver»), o elige su tipo en «Conjuga tu verbo»."
            ),
        }
    }
}

/// Lower-cases the root, trims it, and writes every saltillo as a straight
/// apostrophe, so "  'Ähä ", "ʼähä" and "’ähä" all become "'ähä".
fn normalize_root(verb: &str) -> std::result::Result<String, Error> {
    let root: String = verb
        .trim()
        .trim_start_matches('-')
        .to_lowercase()
        .chars()
        .map(|c| if c == 'ʼ' || c == '’' { '\'' } else { c })
        .collect();
    if root.is_empty() {
        return Err(Error::EmptyVerb);
    }
    let ok = root.chars().all(|c| {
        c.is_alphabetic() || c == '\''
            // combining marks such as the line below in "u̱"
            || ('\u{0300}'..='\u{036F}').contains(&c)
    });
    if !ok || root.chars().count() > 40 {
        return Err(Error::InvalidVerb);
    }
    Ok(root)
}

impl Conjugation {
    pub fn conjugate(&self, verb: &str) -> std::result::Result<Result, Error> {
        self.conjugate_with(verb, &Options::default())
    }

    pub fn conjugate_with(&self, verb: &str, opts: &Options) -> std::result::Result<Result, Error> {
        let root = normalize_root(verb)?;
        let entry = self.lexicon.iter().find(|e| e.root == root);
        // A meaning typed by the user wins over the lexicon's.
        let es_meaning = opts
            .es()
            .map(str::to_string)
            .or_else(|| entry.and_then(|e| e.es.clone()));

        let b = entry
            .and_then(|e| e.b.clone())
            .unwrap_or_else(|| otomi::form_b(&root));
        let y = entry
            .and_then(|e| e.y.clone())
            .or_else(|| otomi::form_y(&root));
        let takes_n = entry.is_some_and(|e| e.n) || opts.kind() == Some("n");

        let stem_for = |p: &Paradigm, person: &str| -> String {
            if p.secondary && takes_n {
                return otomi::form_n(&root);
            }
            match p.stems.get(person).map(String::as_str) {
                Some("Y") => y.clone().unwrap_or_else(|| root.clone()),
                Some("BY") => y.clone().unwrap_or_else(|| b.clone()),
                Some("B") => b.clone(),
                _ => root.clone(),
            }
        };

        let tables = self
            .paradigms
            .iter()
            .map(|p| Table {
                id: p.id.clone(),
                group: p.group.clone(),
                name: p.name.clone(),
                description: p.description.clone(),
                rows: self
                    .persons
                    .iter()
                    .filter_map(|person| {
                        let clitic = p.forms.get(&person.id)?;
                        let stem = stem_for(p, &person.id);
                        let es = |offset: usize| -> Option<String> {
                            let meaning = es_meaning.as_deref()?;
                            spanish::render(
                                meaning,
                                p.es.as_ref()?,
                                person.es? + offset,
                                p.negative,
                            )
                        };
                        Some(Row {
                            es_singular: es(0),
                            es_plural: es(3),
                            person: person.label.clone(),
                            clitic: clitic.clone(),
                            singular: format!("{clitic} {stem}"),
                            plural: person
                                .has_plural
                                .then(|| format!("{clitic} {}", otomi::plural(&stem))),
                        })
                    })
                    .collect(),
            })
            .collect();

        Ok(Result {
            known: entry.is_some(),
            meaning: entry
                .map(|e| e.meaning.clone())
                .or_else(|| opts.es().map(str::to_string)),
            note: entry.and_then(|e| e.note.clone()),
            root,
            tables,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn otomi() -> Conjugation {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/otomi.json")).unwrap();
        serde_json::from_value(raw["conjugation"].clone()).unwrap()
    }

    fn form(res: &Result, table: &str, person: usize) -> String {
        res.tables.iter().find(|t| t.id == table).unwrap().rows[person]
            .singular
            .clone()
    }

    #[test]
    fn present_uses_primary_root() {
        let r = otomi().conjugate("pädi").unwrap();
        assert_eq!(form(&r, "present", 0), "dí pädi");
        assert_eq!(form(&r, "present", 2), "di pädi");
    }

    #[test]
    fn third_person_preterite_uses_b_form() {
        let r = otomi().conjugate("pädi").unwrap();
        assert_eq!(form(&r, "preterite", 0), "dá pädi");
        assert_eq!(form(&r, "preterite", 2), "bi bädi");
        assert_eq!(form(&r, "future", 2), "da bädi");
    }

    #[test]
    fn y_form_on_second_and_third_person() {
        let r = otomi().conjugate("honi").unwrap();
        assert_eq!(form(&r, "preterite", 0), "dá honi");
        assert_eq!(form(&r, "preterite", 1), "gá hyoni");
        assert_eq!(form(&r, "preterite", 2), "bi hyoni");
    }

    #[test]
    fn lexicon_overrides_rules() {
        let r = otomi().conjugate("peni").unwrap();
        assert!(r.known);
        assert_eq!(form(&r, "preterite", 2), "bi meni");
    }

    #[test]
    fn n_verbs_take_n_in_secondary_tenses() {
        let r = otomi().conjugate("sagi").unwrap();
        assert_eq!(form(&r, "preterite", 0), "dá nsagi");
        assert_eq!(form(&r, "present", 0), "dí sagi");
    }

    #[test]
    fn plural_column() {
        let r = otomi().conjugate("tsi").unwrap();
        let t = r.tables.iter().find(|t| t.id == "future").unwrap();
        assert_eq!(t.rows[0].plural.as_deref(), Some("ga tsihu̱"));
    }

    #[test]
    fn normalizes_saltillo() {
        let r = otomi().conjugate(" ʼÄhä ").unwrap();
        assert_eq!(r.root, "'ähä");
        assert_eq!(form(&r, "preterite", 2), "bi 'ñähä");
    }

    #[test]
    fn spanish_translation() {
        let r = otomi().conjugate("pädi").unwrap();
        let t = r.tables.iter().find(|t| t.id == "preterite").unwrap();
        assert_eq!(t.rows[0].es_singular.as_deref(), Some("yo supe"));
        assert_eq!(t.rows[0].es_plural.as_deref(), Some("nosotros supimos"));
        assert_eq!(t.rows[2].es_plural.as_deref(), Some("ellos/ellas supieron"));
        let n = r.tables.iter().find(|t| t.id == "neg-present").unwrap();
        assert_eq!(n.rows[1].es_singular.as_deref(), Some("tú no sabes"));
        // Unknown verbs have no translation.
        let u = otomi().conjugate("zoni").unwrap();
        assert_eq!(u.tables[0].rows[0].es_singular, None);
    }

    #[test]
    fn options_for_your_own_verb() {
        let opts = Options {
            es: Some("cantar".into()),
            kind: None,
        };
        let r = otomi().conjugate_with("tuhu", &opts).unwrap();
        assert!(!r.known);
        assert_eq!(r.meaning.as_deref(), Some("cantar"));
        assert_eq!(form(&r, "preterite", 2), "bi duhu");
        let t = r.tables.iter().find(|t| t.id == "preterite").unwrap();
        assert_eq!(t.rows[0].es_singular.as_deref(), Some("yo canté"));
        // The N type adds n- in the secondary tenses.
        let n = Options {
            es: None,
            kind: Some("n".into()),
        };
        let r = otomi().conjugate_with("tsaya", &n).unwrap();
        assert_eq!(form(&r, "preterite", 0), "dá ntsaya");
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(otomi().conjugate("  ").unwrap_err(), Error::EmptyVerb);
        assert_eq!(otomi().conjugate("ts1").unwrap_err(), Error::InvalidVerb);
    }
}
