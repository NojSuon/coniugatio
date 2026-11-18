//! Conjugation for Timucua, the language of northern Florida and southern
//! Georgia described by Francisco Pareja in his *Arte de la lengua
//! timuquana* (1614; ed. Adam & Vinson, 1886).
//!
//! The spelling is Mati Mati's: Pareja's c / q (before e, i) are written
//! k and his qu (/kʷ/) is q; his bu, u, i before a vowel are the glides
//! bw, w, y; g, x and j (variants of h) are h, and d (t after n) is t. A
//! macron marks a long vowel (tōma) and an acute an unexpected stress
//! (pwenohekó).
//!
//! The verb, after Pareja's model hubuasono «amar» (§ El verbo amar):
//!
//! ```text
//! person   stem      plural   tense / mood          example
//! ni-      hubwaso   (-bo-)   -ta-la  present        nihubwasotala   «yo amo»
//! chi-                        -bi-la  imperfect      chihubwasobila  «tú amabas»
//! —                  (-mo-)   -chunu  preterite      hubwasomochunu  «ellos amaron»
//!                             -bile-bila pluperfect, -habela future,
//!                             -te-hekó question, -ti-la negative
//! ```
//!
//! ni- and chi- mark «yo» and «tú»; the plural keeps them and adds -bo-
//! («nosotros», «ustedes»); the 3rd person has no prefix and -mo- in the
//! plural. Orders are the bare stem (hubwaso «¡ama!»), -tama for «ustedes»
//! and -mo for «amemos».

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// -tala: nihubwasotala «yo amo».
    Present,
    /// -bila: nihubwasobila «yo amaba».
    Imperfect,
    /// -chunu: nihubwasochunu «yo amé».
    Preterite,
    /// -bilebila: nihubwasobilebila «yo había amado».
    Pluperfect,
    /// -habela: nihubwasohabela «yo amaré».
    Future,
    /// -tehekó: nihubwasotehekó «¿amo yo?».
    Question,
    /// hubwaso «¡ama!», hubwasotama «¡amen!».
    Imperative,
    /// hubwasomo «¡amemos!».
    Hortative,
    /// -tila: nihubwasotila «no amo».
    Negative,
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
    /// Infinitive in -no or -nu: "hubwasono", "hebwano".
    pub entry: String,
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
    "aeiouāēīōūáéíóú".contains(c)
}

/// The stem: the infinitive without -no / -nu (hubwasono → hubwaso).
fn stem(word: &str) -> &str {
    word.strip_suffix("no")
        .or_else(|| word.strip_suffix("nu"))
        .filter(|s| s.chars().any(is_vowel))
        .unwrap_or(word)
}

#[derive(Clone, Copy)]
struct Slot {
    person: usize,
    plural: bool,
}

impl Slot {
    /// ni- yo / nosotros, chi- tú / ustedes, — él / ellos.
    fn prefix(self) -> &'static str {
        ["ni", "chi", ""][self.person]
    }

    /// -bo- nosotros, ustedes; -mo- ellos.
    fn plural(self) -> &'static str {
        match (self.plural, self.person) {
            (false, _) => "",
            (true, 2) => "mo",
            (true, _) => "bo",
        }
    }
}

fn build(stem: &str, form: Form, s: Slot) -> Option<String> {
    let full = |ending: &str| Some(format!("{}{stem}{}{ending}", s.prefix(), s.plural()));
    match form {
        Form::Present => full("tala"),
        Form::Imperfect => full("bila"),
        Form::Preterite => full("chunu"),
        Form::Pluperfect => full("bilebila"),
        Form::Future => full("habela"),
        Form::Question => full("tehekó"),
        Form::Negative => full("tila"),
        Form::Imperative => match (s.person, s.plural) {
            (1, false) => Some(stem.to_string()),
            (1, true) => Some(format!("{stem}tama")),
            _ => None,
        },
        Form::Hortative => match (s.person, s.plural) {
            (0, true) => Some(format!("{stem}mo")),
            _ => None,
        },
    }
}

fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb.trim().to_lowercase().nfc_simple();
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(char::is_alphabetic) || w.chars().count() > 40 || !w.chars().any(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

/// Composes a vowel + combining macron or acute typed separately (a + ̄).
trait Nfc {
    fn nfc_simple(&self) -> String;
}

impl Nfc for String {
    fn nfc_simple(&self) -> String {
        let mut out = String::new();
        for c in self.chars() {
            let last = out.pop();
            let joined = match (last, c) {
                (Some(v), '\u{304}') => "aāeēiīoōuū".find(v).map(|i| "aāeēiīoōuū"[i..].chars().nth(1).unwrap()),
                (Some(v), '\u{301}') => "aáeéiíoóuú".find(v).map(|i| "aáeéiíoóuú"[i..].chars().nth(1).unwrap()),
                _ => None,
            };
            match (joined, last) {
                (Some(j), _) => out.push(j),
                (None, Some(l)) => {
                    out.push(l);
                    out.push(c);
                }
                (None, None) => out.push(c),
            }
        }
        out
    }
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.lexicon.iter().find(|e| e.entry == word || stem(&e.entry) == word);
        let root = entry.map(|e| e.entry.clone()).unwrap_or_else(|| word.clone());
        let st = stem(&root).to_string();

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
                    .enumerate()
                    .map(|(i, person)| {
                        let sg = build(&st, tense.form, Slot { person: i.min(2), plural: false });
                        let pl = build(&st, tense.form, Slot { person: i.min(2), plural: true });
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
            root,
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

    fn six(verb: &str, form: Form) -> Vec<String> {
        let st = stem(verb);
        [(0, false), (1, false), (2, false), (0, true), (1, true), (2, true)]
            .iter()
            .map(|&(person, plural)| build(st, form, Slot { person, plural }).unwrap_or_default())
            .collect()
    }

    // Pareja (1614), «El verbo amar», respelled.
    #[test]
    fn amar() {
        // Diosima hubuasotala; nihubuasobohabele; chihubuasobohabela;
        // hubuasomohabela; nihubuasobobilebila; nihubuasobilebila.
        assert_eq!(six("hubwasono", Form::Present)[0], "nihubwasotala");
        assert_eq!(six("hubwasono", Form::Present)[4], "chihubwasobotala");
        assert_eq!(
            six("hubwasono", Form::Future),
            [
                "nihubwasohabela",
                "chihubwasohabela",
                "hubwasohabela",
                "nihubwasobohabela",
                "chihubwasobohabela",
                "hubwasomohabela"
            ]
        );
        assert_eq!(six("hubwasono", Form::Pluperfect)[3], "nihubwasobobilebila");
        assert_eq!(six("hubwasono", Form::Preterite)[2], "hubwasochunu");
        assert_eq!(six("hubwasono", Form::Imperfect)[0], "nihubwasobila");
        // nihubuasoteheco, nihubuasoboteheco: stressed -hekó.
        assert_eq!(six("hubwasono", Form::Question)[3], "nihubwasobotehekó");
        // hubuaso «ama tú», hubuasotama «amad», hubuasomo «amemos».
        assert_eq!(six("hubwasono", Form::Imperative)[1], "hubwaso");
        assert_eq!(six("hubwasono", Form::Imperative)[4], "hubwasotama");
        assert_eq!(six("hubwasono", Form::Hortative)[3], "hubwasomo");
        assert_eq!(six("hubwasono", Form::Imperative)[0], "");
        // nenatila «no veo»: -ti-la.
        assert_eq!(six("hubwasono", Form::Negative)[0], "nihubwasotila");
        assert_eq!(stem("ukunu"), "uku");
    }

    #[test]
    fn combining_marks() {
        assert_eq!(normalize("to\u{304}ma").unwrap(), "tōma");
        assert_eq!(normalize("heko\u{301}").unwrap(), "hekó");
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/timucua.json")).unwrap();
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
    }
}
