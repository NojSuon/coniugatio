//! Conjugation for Ch'ol (lakty'añ, Mayan), after Jessica Coon, «Ch'ol»
//! (2015, after Vázquez Álvarez 2002, 2011), in the INALI spelling of the
//! Norma de escritura (ty, ñ, ä, ch', ts', ty'), as in the Vocabulario de la
//! lengua ch'ol (Sánchez Álvarez et al., 2013).
//!
//! The verb takes an aspect word and two sets of person markers: Set A
//! prefixes (k-, a-, i-) and Set B suffixes (-oñ, -ety, Ø). Ch'ol has a
//! split by aspect: in the perfective an intransitive subject is Set B, in
//! the imperfective and progressive it is Set A, on a nominal -el stem.
//!
//! ```text
//!               intransitive majl «go»     transitive k'ux «eat»
//! perfective    tsa' majli-yoñ «I went»    tsa' jk'ux-u «I ate it»
//! imperfective  mi k-majl-el «I go»        mi jk'ux «I eat it»
//! ```
//!
//! Root transitives take a vowel that echoes the root (k'ux-u, mek'-e) in
//! the perfective and nothing in the imperfective; derived transitives take
//! a set vowel and -Vñ (il-ä / il-äñ, ts'ijb-u / ts'ijb-uñ).

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// tsa' majliyoñ «I went», tsa' jk'uxu «I ate it».
    Perfective,
    /// tsa'ix majliyoñ «I already went».
    PerfectiveAlready,
    /// mi kmajlel «I go», mi jk'ux «I eat it».
    Imperfective,
    /// chonkol kmajlel «I am going».
    Progressive,
    /// ma'añ tsa' majliyoñ «I didn't go».
    NegativePerfective,
    /// ma'añ mi kmajlel «I don't go».
    NegativeImperfective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Intransitive: majl «go» (majlel).
    #[default]
    Intransitive,
    /// Root transitive, echo vowel: k'ux-u, mek'-e.
    Transitive,
    /// Derived transitive, set vowel and -Vñ: il-ä / il-äñ.
    Derived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tense {
    pub id: String,
    pub group: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub form: Form,
    #[serde(default)]
    pub negative: bool,
    #[serde(default)]
    pub es: Option<spanish::Es>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The dictionary form: "majlel" (intransitive), "k'ux", "il".
    pub entry: String,
    /// The root, when it isn't the entry: majlel → majl.
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub kind: Kind,
    /// The stem vowel of a transitive, when it isn't the root's own: il → ä.
    #[serde(default)]
    pub vowel: Option<String>,
    pub meaning: String,
    #[serde(default)]
    pub es: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conjugation {
    pub persons: Vec<Person>,
    pub tenses: Vec<Tense>,
    #[serde(default)]
    pub lexicon: Vec<Entry>,
}

fn is_vowel(c: char) -> bool {
    "aeiouä".contains(c)
}

fn starts_with_vowel(s: &str) -> bool {
    s.chars().next().is_some_and(is_vowel)
}

/// Persons 0–6: 1sg, 2sg, 3sg, 1pl (with you), 1pl (not you), 2pl, 3pl.
/// Set A prefix and the plural suffix that goes with it.
fn set_a(p: usize, stem: &str) -> (String, &'static str) {
    let v = starts_with_vowel(stem);
    let prefix = match p {
        // k- is j- before a velar: jk'ux, jkuch.
        0 | 4 if !v && stem.starts_with('k') => "j",
        0 | 4 => "k",
        3 => "lak",
        1 | 5 => {
            if v {
                "aw"
            } else {
                "a"
            }
        }
        _ => {
            if v {
                "y"
            } else {
                "i"
            }
        }
    };
    let plural = match p {
        4 => "lojoñ",
        5 => "la",
        6 => "ob",
        _ => "",
    };
    (prefix.to_string(), plural)
}

/// Set B suffixes for persons 0–6: -oñ, -ety, Ø, -oñla, -oñlojoñ, -etyla, -ob.
const SET_B: [&str; 7] = ["oñ", "ety", "", "oñla", "oñlojoñ", "etyla", "ob"];

/// Adds a suffix, with y between vowels: majli-y-oñ, k'uxu-y-ob.
fn add(stem: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        return stem.to_string();
    }
    let last = stem.chars().last().is_some_and(is_vowel);
    if last && starts_with_vowel(suffix) {
        format!("{stem}y{suffix}")
    } else {
        format!("{stem}{suffix}")
    }
}

/// The echo vowel of a root transitive: k'ux → u, mek' → e.
fn echo(root: &str) -> String {
    root.chars().filter(|&c| is_vowel(c)).last().unwrap_or('a').to_string()
}

/// The verb for person `p` (0–6), with the aspect word.
fn build(root: &str, kind: Kind, vowel: &str, form: Form, p: usize) -> String {
    let (aspect, negative) = match form {
        Form::Perfective => ("tsa'", false),
        Form::PerfectiveAlready => ("tsa'ix", false),
        Form::Imperfective => ("mi", false),
        Form::Progressive => ("chonkol", false),
        Form::NegativePerfective => ("tsa'", true),
        Form::NegativeImperfective => ("mi", true),
    };
    let perfective = matches!(form, Form::Perfective | Form::PerfectiveAlready | Form::NegativePerfective);
    let verb = match kind {
        // Perfective: Set B on the -i stem. Otherwise: Set A on -el.
        Kind::Intransitive if perfective => add(&format!("{root}i"), SET_B[p]),
        Kind::Intransitive => {
            let (a, pl) = set_a(p, root);
            add(&format!("{a}{root}el"), pl)
        }
        // Transitives: Set A subject, 3rd person object (Set B Ø).
        Kind::Transitive | Kind::Derived => {
            let (a, pl) = set_a(p, root);
            let stem = match (kind, perfective) {
                (_, true) => format!("{root}{vowel}"),
                (Kind::Derived, false) => format!("{root}{vowel}ñ"),
                _ => root.to_string(),
            };
            add(&format!("{a}{stem}"), pl)
        }
    };
    if negative {
        format!("ma'añ {aspect} {verb}")
    } else {
        format!("{aspect} {verb}")
    }
}

/// Lowercase, one apostrophe, Ch'ol letters only.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w = verb.trim().to_lowercase().replace(['’', 'ʼ', '´', '`', '‘', '‟'], "'");
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_alphabetic() || c == '\'') || w.chars().count() > 30 || !w.chars().any(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

/// A typed word that isn't in the lexicon: -el is intransitive (majlel),
/// anything else a root transitive (k'ux).
fn guess(word: &str) -> (String, Kind) {
    match word.strip_suffix("el") {
        Some(r) if r.chars().any(is_vowel) => (r.to_string(), Kind::Intransitive),
        _ => (word.to_string(), Kind::Transitive),
    }
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    fn root_of(e: &Entry) -> String {
        e.root.clone().unwrap_or_else(|| guess(&e.entry.to_lowercase()).0)
    }

    fn find(&self, word: &str) -> Option<&Entry> {
        self.lexicon
            .iter()
            .find(|e| e.entry.to_lowercase() == word)
            .or_else(|| self.lexicon.iter().find(|e| Self::root_of(e) == word))
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.find(&word);
        let (root, kind) = match entry {
            Some(e) => (Self::root_of(e), e.kind),
            None => guess(&word),
        };
        let vowel = entry.and_then(|e| e.vowel.clone()).unwrap_or_else(|| echo(&root));

        let es_meaning = opts.es().map(str::to_string).or_else(|| entry.and_then(|e| e.es.clone()));
        let es = |tense: &Tense, person: &Person, plural: bool| -> Option<String> {
            let meaning = es_meaning.as_deref()?;
            let p = person.es? + if plural { 3 } else { 0 };
            crate::gloss(opts, meaning, tense.es.as_ref()?, p, tense.negative)
        };

        let tables = self
            .tenses
            .iter()
            .map(|tense| Table {
                id: tense.id.clone(),
                group: tense.group.clone(),
                name: tense.name.clone(),
                description: tense.description.clone(),
                rows: self
                    .persons
                    .iter()
                    .map(|person| {
                        // Rows: 1 (yo / nosotros con ustedes), 1x (— / sin
                        // ustedes), 2, 3.
                        let (sg, pl) = match person.id.as_str() {
                            "1" => (Some(0), Some(3)),
                            "1x" => (None, Some(4)),
                            "2" => (Some(1), Some(5)),
                            _ => (Some(2), Some(6)),
                        };
                        let sg = sg.map(|p| build(&root, kind, &vowel, tense.form, p));
                        let pl = pl.map(|p| build(&root, kind, &vowel, tense.form, p));
                        Row {
                            person: person.label.clone(),
                            clitic: String::new(),
                            es_singular: sg.as_ref().and_then(|_| es(tense, person, false)),
                            es_plural: pl.as_ref().and_then(|_| es(tense, person, true)),
                            singular: sg.unwrap_or_default(),
                            plural: pl,
                        }
                    })
                    .collect(),
            })
            .collect();

        Ok(Conjugated {
            root: entry.map(|e| e.entry.clone()).unwrap_or_else(|| word.clone()),
            known: entry.is_some(),
            meaning: entry.map(|e| e.meaning.clone()).or_else(|| opts.es().map(str::to_string)),
            note: entry.and_then(|e| e.note.clone()),
            tables,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seven(root: &str, kind: Kind, vowel: &str, form: Form) -> Vec<String> {
        (0..7).map(|p| build(root, kind, vowel, form, p)).collect()
    }

    #[test]
    fn intransitives() {
        // Coon (34a) tyi wäy-i-yoñ «I slept»; Vocabulario: tsa' majliyoñ,
        // tsa' majli, tsa' majliyob.
        assert_eq!(
            seven("majl", Kind::Intransitive, "", Form::Perfective),
            [
                "tsa' majliyoñ", "tsa' majliyety", "tsa' majli", "tsa' majliyoñla",
                "tsa' majliyoñlojoñ", "tsa' majliyetyla", "tsa' majliyob"
            ]
        );
        // (34b) mi k-majl-el; (14a) mi i-wejl-el; (14b) k-uch'-el.
        assert_eq!(build("majl", Kind::Intransitive, "", Form::Imperfective, 0), "mi kmajlel");
        assert_eq!(build("wejl", Kind::Intransitive, "", Form::Imperfective, 2), "mi iwejlel");
        assert_eq!(build("uch'", Kind::Intransitive, "", Form::Imperfective, 0), "mi kuch'el");
        assert_eq!(build("och", Kind::Intransitive, "", Form::Imperfective, 2), "mi yochel");
        assert_eq!(build("majl", Kind::Intransitive, "", Form::Imperfective, 6), "mi imajlelob");
        // (56b) ma'añ mi i-majl-el.
        assert_eq!(build("majl", Kind::Intransitive, "", Form::NegativeImperfective, 2), "ma'añ mi imajlel");
    }

    #[test]
    fn transitives() {
        // (8b) tyi k-päk'-ä «I planted»; (2) k-il-ä-yob «I saw them»;
        // (9a) mi k-päk'; (9b) chonkol k-mek'.
        assert_eq!(build("päk'", Kind::Transitive, "ä", Form::Perfective, 0), "tsa' kpäk'ä");
        assert_eq!(build("päk'", Kind::Transitive, "ä", Form::Imperfective, 0), "mi kpäk'");
        assert_eq!(build("mek'", Kind::Transitive, "e", Form::Progressive, 0), "chonkol kmek'");
        assert_eq!(build("k'ux", Kind::Transitive, "u", Form::Perfective, 0), "tsa' jk'uxu");
        assert_eq!(build("k'ux", Kind::Transitive, "u", Form::Perfective, 6), "tsa' ik'uxuyob");
        // (11c) tyi i-ts'ijb-u; (12c) i-ts'ijb-uñ; (11d)/(12d) aw-il-ä, aw-il-äñ.
        assert_eq!(build("ts'ijb", Kind::Derived, "u", Form::Perfective, 2), "tsa' its'ijbu");
        assert_eq!(build("ts'ijb", Kind::Derived, "u", Form::Progressive, 2), "chonkol its'ijbuñ");
        assert_eq!(build("il", Kind::Derived, "ä", Form::Perfective, 1), "tsa' awilä");
        assert_eq!(build("il", Kind::Derived, "ä", Form::Imperfective, 1), "mi awiläñ");
        assert_eq!(echo("k'ux"), "u");
    }

    #[test]
    fn input() {
        assert_eq!(normalize(" Majlel ").unwrap(), "majlel");
        assert_eq!(normalize("k’ux").unwrap(), "k'ux");
        assert!(normalize("maj1").is_err());
        assert_eq!(guess("wäyel"), ("wäy".to_string(), Kind::Intransitive));
        assert_eq!(guess("mek'"), ("mek'".to_string(), Kind::Transitive));
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/chol.json")).unwrap();
        let c: Conjugation = serde_json::from_value(raw["conjugation"].clone()).unwrap();
        for e in &c.lexicon {
            let r = c.conjugate(&e.entry).unwrap();
            assert!(r.known, "{}", e.entry);
            for t in &r.tables {
                for row in &t.rows {
                    assert!(row.singular.is_empty() || row.es_singular.is_some(), "{} {}", e.entry, t.id);
                    assert!(row.plural.as_deref().map_or(true, str::is_empty) || row.es_plural.is_some(), "{} {}", e.entry, t.id);
                }
            }
        }
        assert!(c.conjugate("majl").unwrap().known);
        assert_eq!(c.conjugate("majlel").unwrap().tables[0].rows[0].singular, "tsa' majliyoñ");
    }
}
