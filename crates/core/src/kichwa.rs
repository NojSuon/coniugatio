//! Conjugation for Ecuadorian Kichwa (Northern Kichwa, kichwa shimi), in the
//! unified spelling (shukyachishka killkay) of the Ministerio de Educación
//! and the *Kichwa Yachakukkunapak Shimiyuk Kamu* (2009): only the vowels
//! a, i, u; k for every /k/; w and y for the glides; -nchik, -nki, -pak
//! written alike in every province, however they are said.
//!
//! Grammar after Mons. Alejandro Múgica, *Aprendamos kichwa* (gramática
//! kichwa actualizada, Napo), with *Aprenda usted mismo el quichua* for the
//! highland forms.
//!
//! A verb is the root (the infinitive without -na), an optional tense
//! suffix and a person ending:
//!
//! ```text
//! present   miku-ni  miku-nki  miku-n  miku-nchik  miku-nkichik  miku-nkuna
//! past      miku-rka-ni  …  miku-rka  …  miku-rka-kuna
//! future    mikusha  mikunki  mikunka  mikushun  mikunkichik  mikunkakuna
//! ```
//!
//! The persons are ñuka, kan, pay, ñukanchik, kankuna and paykuna. Unlike
//! southern Quechua, Ecuadorian Kichwa has a single «we» (ñukanchik).
//! Stress falls on the second-to-last syllable, so no accents are written.

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// mikuni «como».
    Present,
    /// mikurkani «comí».
    Past,
    /// mikushkani «he comido, había comido».
    Perfect,
    /// mikushka karkani «había comido».
    Pluperfect,
    /// mikukuni «estoy comiendo».
    Progressive,
    /// mikukurkani «estaba comiendo».
    PastProgressive,
    /// mikuk karkani «solía comer».
    PastHabitual,
    /// mikusha «comeré».
    Future,
    /// mikuyman «comería».
    Conditional,
    /// mikuy «¡come!», mikuychik «¡coman!».
    Imperative,
    /// mikushun «¡comamos!».
    Hortative,
    /// mikuchun «¡que coma!», mikuchunkuna «¡que coman!».
    Jussive,
    /// ama mikuychu «¡no comas!».
    Prohibitive,
    /// mana mikunichu «no como».
    Negative,
    /// mana mikurkanichu «no comí».
    NegativePast,
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
    /// The infinitive: "mikuna", "rina".
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
    "aiu".contains(c)
}

/// Endings for ñuka, kan, pay, ñukanchik, kankuna, paykuna.
const PRESENT: [&str; 6] = ["ni", "nki", "n", "nchik", "nkichik", "nkuna"];
/// After -rka-, -shka- the 3rd persons are -Ø and -kuna: mikurka, mikurkakuna.
const PAST: [&str; 6] = ["ni", "nki", "", "nchik", "nkichik", "kuna"];
const FUTURE: [&str; 6] = ["sha", "nki", "nka", "shun", "nkichik", "nkakuna"];
const CONDITIONAL: [&str; 6] = ["yman", "nkiman", "nman", "nchikman", "nkichikman", "nkunaman"];
/// The auxiliary kana «ser, estar» in the past.
const KARKA: [&str; 6] = ["karkani", "karkanki", "karka", "karkanchik", "karkankichik", "karkakuna"];

/// The form for person `p` (0–5), or None where the form has no such person.
fn build(r: &str, form: Form, p: usize) -> Option<String> {
    Some(match form {
        Form::Present => format!("{r}{}", PRESENT[p]),
        Form::Past => format!("{r}rka{}", PAST[p]),
        Form::Perfect => format!("{r}shka{}", PAST[p]),
        Form::Pluperfect => format!("{r}shka {}", KARKA[p]),
        Form::Progressive => format!("{r}ku{}", PRESENT[p]),
        Form::PastProgressive => format!("{r}kurka{}", PAST[p]),
        Form::PastHabitual => format!("{r}k {}", KARKA[p]),
        Form::Future => format!("{r}{}", FUTURE[p]),
        Form::Conditional => format!("{r}{}", CONDITIONAL[p]),
        Form::Imperative => match p {
            1 => format!("{r}y"),
            4 => format!("{r}ychik"),
            _ => return None,
        },
        Form::Hortative => match p {
            3 => format!("{r}shun"),
            _ => return None,
        },
        Form::Jussive => match p {
            2 => format!("{r}chun"),
            5 => format!("{r}chunkuna"),
            _ => return None,
        },
        Form::Prohibitive => match p {
            1 => format!("ama {r}ychu"),
            4 => format!("ama {r}ychikchu"),
            _ => return None,
        },
        Form::Negative => format!("mana {r}{}chu", PRESENT[p]),
        Form::NegativePast => format!("mana {r}rka{}chu", PAST[p]),
    })
}

/// Lowercase, trimmed, Kichwa letters only.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w = verb.trim().to_lowercase();
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(char::is_alphabetic) || w.chars().count() > 40 || !w.chars().any(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

/// The root of an infinitive: mikuna → miku, rina → ri. A word without
/// -na is taken as the root itself.
fn root_of(word: &str) -> std::result::Result<String, Error> {
    let root = match word.strip_suffix("na") {
        Some(r) if r.chars().last().is_some_and(is_vowel) => r,
        _ => word,
    };
    if !root.chars().last().is_some_and(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(root.to_string())
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    /// The entry for a typed word: the infinitive (mikuna) or the root (miku).
    fn find(&self, word: &str) -> Option<&Entry> {
        let with_na = format!("{word}na");
        self.lexicon.iter().find(|e| {
            let key = e.entry.to_lowercase();
            key == word || key == with_na
        })
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.find(&word);
        let root = root_of(&entry.map(|e| e.entry.to_lowercase()).unwrap_or(word))?;

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
                        // Rows 1, 2, 3: singular p, plural p + 3.
                        let i = match person.id.as_str() {
                            "1" => 0,
                            "2" => 1,
                            _ => 2,
                        };
                        let sg = build(&root, tense.form, i);
                        let pl = build(&root, tense.form, i + 3);
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
            root: entry.map(|e| e.entry.clone()).unwrap_or_else(|| format!("{root}na")),
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

    fn six(root: &str, form: Form) -> Vec<String> {
        (0..6).map(|p| build(root, form, p).unwrap_or_default()).collect()
    }

    #[test]
    fn indicative() {
        assert_eq!(
            six("miku", Form::Present),
            ["mikuni", "mikunki", "mikun", "mikunchik", "mikunkichik", "mikunkuna"]
        );
        assert_eq!(
            six("miku", Form::Past),
            ["mikurkani", "mikurkanki", "mikurka", "mikurkanchik", "mikurkankichik", "mikurkakuna"]
        );
        assert_eq!(
            six("miku", Form::Future),
            ["mikusha", "mikunki", "mikunka", "mikushun", "mikunkichik", "mikunkakuna"]
        );
        assert_eq!(six("miku", Form::Perfect)[2], "mikushka");
        assert_eq!(six("miku", Form::Pluperfect)[0], "mikushka karkani");
        assert_eq!(six("miku", Form::Progressive)[0], "mikukuni");
        assert_eq!(six("miku", Form::PastProgressive)[5], "mikukurkakuna");
        assert_eq!(six("miku", Form::PastHabitual)[0], "mikuk karkani");
        assert_eq!(six("miku", Form::Conditional)[0], "mikuyman");
    }

    #[test]
    fn moods() {
        assert_eq!(build("miku", Form::Imperative, 1).unwrap(), "mikuy");
        assert_eq!(build("miku", Form::Imperative, 4).unwrap(), "mikuychik");
        assert_eq!(build("miku", Form::Hortative, 3).unwrap(), "mikushun");
        assert_eq!(build("miku", Form::Imperative, 0), None);
        assert_eq!(build("miku", Form::Jussive, 5).unwrap(), "mikuchunkuna");
        assert_eq!(build("waka", Form::Prohibitive, 1).unwrap(), "ama wakaychu");
        assert_eq!(build("miku", Form::Negative, 0).unwrap(), "mana mikunichu");
        assert_eq!(build("miku", Form::NegativePast, 2).unwrap(), "mana mikurkachu");
    }

    #[test]
    fn input() {
        assert_eq!(root_of("mikuna").unwrap(), "miku");
        assert_eq!(root_of("rina").unwrap(), "ri");
        assert_eq!(root_of("miku").unwrap(), "miku");
        assert!(root_of("mikun").is_err());
        assert!(normalize("miku-na").is_err());
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/kichwa.json")).unwrap();
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
        // «munana» (querer) is found by its root too, and kuna (dar) stays ku-.
        assert!(c.conjugate("muna").unwrap().known);
        assert_eq!(c.conjugate("kuna").unwrap().tables[0].rows[0].singular, "kuni");
    }
}
