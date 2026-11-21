//! Conjugation for Cusco Quechua (qheswa simi, Cuzco-Collao), after Antonio
//! Cusihuamán G., *Gramática quechua Cuzco-Collao* (Cusco: CBC, 2nd ed.
//! 2001), ch. 6, in the spelling of the Academia Mayor de la Lengua Quechua
//! (*Diccionario quechua–español–quechua*, Cusco 2005): five vowels, q for
//! the uvular, aspirates (ph, th, chh, kh, qh) and ejectives (p', t', ch',
//! k', q').
//!
//! A verb is the root (the infinitive without -y), an optional tense
//! suffix and a person ending:
//!
//! ```text
//! present   rima-ni  rima-nki  rima-n  rima-nchis  rima-yku  rima-nkichis  rima-nku
//! past      rima-rqa-ni  …  rima-rqa-n  …
//! future    rimasaq  rimanki  rimanqa  rimasunchis  rimasaqku  rimankichis  rimanqaku
//! ```
//!
//! The persons are ñoqa, qan, pay, ñoqanchis («we, with you»), ñoqayku («we,
//! not you»), qankuna and paykuna. Stress falls on the second-to-last
//! syllable and is not written.

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// rimani «hablo».
    Present,
    /// rimarqani «hablé» (witnessed past, -rqa-).
    Past,
    /// rimasqani «había hablado, dice que hablé» (-sqa-).
    Reportative,
    /// rimaq kani «solía hablar».
    PastHabitual,
    /// rimashani «estoy hablando».
    Progressive,
    /// rimasharqani «estaba hablando».
    PastProgressive,
    /// rimasaq «hablaré».
    Future,
    /// rimayman «hablaría».
    Conditional,
    /// rimayman karqan «hubiera hablado».
    PastConditional,
    /// rimay «¡habla!», rimaychis «¡hablen!».
    Imperative,
    /// rimasunchis «¡hablemos!».
    Hortative,
    /// rimachun «¡que hable!», rimachunku «¡que hablen!».
    Jussive,
    /// ama rimaychu «¡no hables!».
    Prohibitive,
    /// mana rimanichu «no hablo».
    Negative,
    /// mana rimarqanichu «no hablé».
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
    /// The infinitive: "rimay", "riy", "llank'ay".
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
    "aeiou".contains(c)
}

/// Endings for ñoqa, qan, pay, ñoqanchis, ñoqayku, qankuna, paykuna.
const PRESENT: [&str; 7] = ["ni", "nki", "n", "nchis", "yku", "nkichis", "nku"];
const FUTURE: [&str; 7] = ["saq", "nki", "nqa", "sunchis", "saqku", "nkichis", "nqaku"];
/// After -sqa- the 3rd persons drop -n: rimasqa, rimasqaku (Cusihuamán 6.23.22).
const SQA: [&str; 7] = ["ni", "nki", "", "nchis", "yku", "nkichis", "ku"];
/// Cusihuamán 6.24.21: apayman, apawaq, apanman, apasunman, apaykuman,
/// apawaqchis, apankuman.
const CONDITIONAL: [&str; 7] = ["yman", "waq", "nman", "sunman", "ykuman", "waqchis", "nkuman"];
/// The auxiliary kay «ser» in the present, for the past habitual.
const KA: [&str; 7] = ["kani", "kanki", "", "kanchis", "kayku", "kankichis", ""];

/// The form for person `p` (0–6), or None where the form has no such person.
fn build(r: &str, form: Form, p: usize) -> Option<String> {
    Some(match form {
        Form::Present => format!("{r}{}", PRESENT[p]),
        Form::Past => format!("{r}rqa{}", PRESENT[p]),
        Form::Reportative => format!("{r}sqa{}", SQA[p]),
        // 3rd persons drop kay: rimaq, rimaqku (Cusihuamán 6.23.23).
        Form::PastHabitual => match p {
            2 => format!("{r}q"),
            6 => format!("{r}qku"),
            _ => format!("{r}q {}", KA[p]),
        },
        Form::Progressive => format!("{r}sha{}", PRESENT[p]),
        Form::PastProgressive => format!("{r}sharqa{}", PRESENT[p]),
        Form::Future => format!("{r}{}", FUTURE[p]),
        Form::Conditional => format!("{r}{}", CONDITIONAL[p]),
        Form::PastConditional => format!("{r}{} karqan", CONDITIONAL[p]),
        Form::Imperative => match p {
            1 => format!("{r}y"),
            5 => format!("{r}ychis"),
            _ => return None,
        },
        Form::Hortative => match p {
            3 => format!("{r}sunchis"),
            _ => return None,
        },
        Form::Jussive => match p {
            2 => format!("{r}chun"),
            6 => format!("{r}chunku"),
            _ => return None,
        },
        Form::Prohibitive => match p {
            1 => format!("ama {r}ychu"),
            5 => format!("ama {r}ychischu"),
            _ => return None,
        },
        Form::Negative => format!("mana {r}{}chu", PRESENT[p]),
        Form::NegativePast => format!("mana {r}rqa{}chu", PRESENT[p]),
    })
}

/// Lowercase, one kind of apostrophe, Quechua letters only.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w = verb.trim().to_lowercase().replace(['’', 'ʼ', '´', '`', '‘'], "'");
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_alphabetic() || c == '\'')
        || w.chars().count() > 40
        || !w.chars().any(is_vowel)
    {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

/// The root of a typed word: rimay → rima; a bare root stays (rima).
fn root_of(word: &str) -> std::result::Result<String, Error> {
    let root = match word.strip_suffix('y') {
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

    /// The entry for a typed word: the infinitive (rimay) or the root (rima).
    fn find(&self, word: &str) -> Option<&Entry> {
        let with_y = format!("{word}y");
        self.lexicon.iter().find(|e| {
            let key = e.entry.to_lowercase();
            key == word || key == with_y
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
                        // Rows: 1 (ñoqa / ñoqanchis), 1x (— / ñoqayku),
                        // 2 (qan / qankuna), 3 (pay / paykuna).
                        let (sg, pl) = match person.id.as_str() {
                            "1" => (Some(0), Some(3)),
                            "1x" => (None, Some(4)),
                            "2" => (Some(1), Some(5)),
                            _ => (Some(2), Some(6)),
                        };
                        let sg = sg.and_then(|p| build(&root, tense.form, p));
                        let pl = pl.and_then(|p| build(&root, tense.form, p));
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
            root: entry.map(|e| e.entry.clone()).unwrap_or_else(|| format!("{root}y")),
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

    fn seven(root: &str, form: Form) -> Vec<String> {
        (0..7).map(|p| build(root, form, p).unwrap_or_default()).collect()
    }

    #[test]
    fn indicative() {
        // Cusihuamán 6.23.1: llank'ani, llank'anki, llank'an, llank'anchis,
        // llank'ayku, llank'ankichis, llank'anku.
        assert_eq!(
            seven("llank'a", Form::Present),
            ["llank'ani", "llank'anki", "llank'an", "llank'anchis", "llank'ayku", "llank'ankichis", "llank'anku"]
        );
        // 6.24.1: rirqani … rirqanku.
        assert_eq!(
            seven("ri", Form::Past),
            ["rirqani", "rirqanki", "rirqan", "rirqanchis", "rirqayku", "rirqankichis", "rirqanku"]
        );
        // 6.24.1: risqani … risqa, risqaku.
        assert_eq!(seven("ri", Form::Reportative)[2], "risqa");
        assert_eq!(seven("ri", Form::Reportative)[6], "risqaku");
        // 6.24.1: riq kani, riq, riqku.
        assert_eq!(seven("ri", Form::PastHabitual), ["riq kani", "riq kanki", "riq", "riq kanchis", "riq kayku", "riq kankichis", "riqku"]);
        // 6.23.3: qelqasaq, qelqanki, qelqanqa, qelqasunchis, qelqasaqku, qelqankichis, qelqanqaku.
        assert_eq!(
            seven("qelqa", Form::Future),
            ["qelqasaq", "qelqanki", "qelqanqa", "qelqasunchis", "qelqasaqku", "qelqankichis", "qelqanqaku"]
        );
        // 6.25.1: purishani, purishanku; 6.25.2: mikhusharqani.
        assert_eq!(seven("puri", Form::Progressive)[6], "purishanku");
        assert_eq!(seven("mikhu", Form::PastProgressive)[0], "mikhusharqani");
        // 6.24.21: apayman, apawaq, apanman, apasunman, apaykuman, apawaqchis, apankuman.
        assert_eq!(
            seven("apa", Form::Conditional),
            ["apayman", "apawaq", "apanman", "apasunman", "apaykuman", "apawaqchis", "apankuman"]
        );
        assert_eq!(seven("apa", Form::PastConditional)[1], "apawaq karqan");
    }

    #[test]
    fn moods() {
        assert_eq!(build("hamu", Form::Imperative, 1).unwrap(), "hamuy");
        assert_eq!(build("hamu", Form::Imperative, 5).unwrap(), "hamuychis");
        assert_eq!(build("hamu", Form::Imperative, 0), None);
        assert_eq!(build("llank'a", Form::Hortative, 3).unwrap(), "llank'asunchis");
        assert_eq!(build("hamu", Form::Jussive, 2).unwrap(), "hamuchun");
        assert_eq!(build("hamu", Form::Jussive, 6).unwrap(), "hamuchunku");
        assert_eq!(build("ri", Form::Prohibitive, 1).unwrap(), "ama riychu");
        assert_eq!(build("muna", Form::Negative, 0).unwrap(), "mana munanichu");
    }

    #[test]
    fn input() {
        assert_eq!(normalize(" Llank\u{2019}ay ").unwrap(), "llank'ay");
        assert!(normalize("rim4y").is_err());
        assert_eq!(root_of("rimay").unwrap(), "rima");
        assert_eq!(root_of("riy").unwrap(), "ri");
        assert_eq!(root_of("rima").unwrap(), "rima");
        assert!(root_of("rim").is_err());
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/qheswa.json")).unwrap();
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
        assert!(c.conjugate("rima").unwrap().known);
        assert_eq!(c.conjugate("qoy").unwrap().tables[0].rows[0].singular, "qoni");
    }
}
