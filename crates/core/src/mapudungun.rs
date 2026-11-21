//! Conjugation for Mapudungun (mapuzugun), after Arturo Hernández Sallés,
//! Nelly Ramos Pizarro and Rosa Huenchulaf Cayuqueo, *Gramática básica de la
//! lengua mapuche*, tomo 1 (Temuco: UC Temuco / CONADI, 2006), §7, in the
//! Azümchefe alphabet it uses (z, ü, tx, nh, lh, q, g for the velar nasal).
//!
//! A verb is the root, optional suffixes (negation, tense, aspect) and an
//! ending for mood, person and number. Verbs are named by the 1st person
//! singular: txipan «salí», küzawün «trabajé». There are three numbers,
//! singular, dual and plural:
//!
//! ```text
//! indicative  txipan  txipaymi  txipay  txipayu  txipaymu  txipaygu  txipayiñ  txipaymün  txipaygün
//! hypothetic  txipali txipalmi  txipale txipaliyu txipalmu txipalegu txipaliñ  txipalmün  txipalegün
//! ```
//!
//! After a root ending in a consonant the endings start with ü or i
//! (küzaw-ün, küzaw-imi, küzaw-i, küzaw-ül-mi); after i the indicative -y-
//! drops (chali-mi). Persons are numbered 0–8: 1sg, 2sg, 3sg, 1du, 2du,
//! 3du, 1pl, 2pl, 3pl.

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// txipan «salí», nien «tengo»: no tense marked.
    Indicative,
    /// amulen «estoy yendo», kutxankülen «estoy enfermo» (-le, -küle).
    Progressive,
    /// amuken «voy siempre» (-ke).
    Habitual,
    /// amuwyen «ya he ido» (-wye, -üwye).
    Perfective,
    /// niefun «tenía», amufun «había ido (pero…)» (-fu).
    Past,
    /// küzawan «trabajaré» (-a, -ya after a).
    Future,
    /// txipayafun «saldría» (-afu).
    Conditional,
    /// txipali «si salgo» (-l, -ül).
    Hypothetical,
    /// txipage «¡sal!», txipamu, txipamün.
    Imperative,
    /// txipayu «¡salgamos (los dos)!», txipayiñ.
    Hortative,
    /// txipachi «¡déjenme salir!», txipape «¡que salga!».
    Jussive,
    /// txekakilge «¡no camines!» (-kil).
    Prohibitive,
    /// txipalan «no salí» (-la).
    Negative,
    /// txipalayan «no saldré» (-la-ya).
    NegativeFuture,
    /// txipanoli «si no salgo» (-no-l).
    NegativeHypothetical,
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
    /// The 1st person singular, the dictionary form: "txipan", "küzawün".
    pub entry: String,
    /// The root, when the entry doesn't show it plainly: anün → anü.
    #[serde(default)]
    pub root: Option<String>,
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
    "aeiouü".contains(c)
}

fn ends_in_consonant(s: &str) -> bool {
    s.chars().last().is_some_and(|c| !is_vowel(c))
}

/// The indicative endings for a stem (Hernández et al. §7.2, §7.4 b–c):
/// txipa-n, txipa-ymi; lef-ün, lef-imi; chali-n, chali-mi.
fn indicative(stem: &str, p: usize) -> String {
    const VOWEL: [&str; 9] = ["n", "ymi", "y", "yu", "ymu", "ygu", "yiñ", "ymün", "ygün"];
    const AFTER_I: [&str; 9] = ["n", "mi", "", "yu", "mu", "gu", "yiñ", "mün", "gün"];
    const CONSONANT: [&str; 9] = ["ün", "imi", "i", "iyu", "imu", "igu", "iyiñ", "imün", "igün"];
    let ending = if ends_in_consonant(stem) {
        CONSONANT[p]
    } else if stem.ends_with('i') {
        AFTER_I[p]
    } else {
        VOWEL[p]
    };
    format!("{stem}{ending}")
}

/// The hypothetical endings after -l (§7.3): txipali, txipalmi, txipale…
const HYPOTHETICAL: [&str; 9] = ["i", "mi", "e", "iyu", "mu", "egu", "iñ", "mün", "egün"];

/// ü between a final consonant and a consonant: kon-ü-ge, af-ül-mu (§7.4 a).
fn join(stem: &str, suffix: &str) -> String {
    if ends_in_consonant(stem) && suffix.chars().next().is_some_and(|c| !is_vowel(c)) {
        format!("{stem}ü{suffix}")
    } else {
        format!("{stem}{suffix}")
    }
}

/// The future stem: küzaw-a, amu-a, txipa-ya (§7.5.2.1).
fn future(stem: &str) -> String {
    if stem.ends_with('a') {
        format!("{stem}ya")
    } else {
        format!("{stem}a")
    }
}

/// The form for person `p` (0–8), or None where the form has no such person.
fn build(r: &str, form: Form, p: usize) -> Option<String> {
    Some(match form {
        Form::Indicative => indicative(r, p),
        // -le after a vowel, -küle after a consonant: amulen, kutxankülen.
        Form::Progressive => {
            let le = if ends_in_consonant(r) { "küle" } else { "le" };
            indicative(&format!("{r}{le}"), p)
        }
        Form::Habitual => indicative(&format!("{r}ke"), p),
        Form::Perfective => indicative(&join(r, "wye"), p),
        Form::Past => indicative(&format!("{r}fu"), p),
        Form::Future => indicative(&future(r), p),
        Form::Conditional => {
            let afu = if r.ends_with('a') { "yafu" } else { "afu" };
            indicative(&format!("{r}{afu}"), p)
        }
        Form::Hypothetical => format!("{}{}", join(r, "l"), HYPOTHETICAL[p]),
        Form::Imperative => match p {
            1 => join(r, "ge"),
            4 => join(r, "mu"),
            7 => join(r, "mün"),
            _ => return None,
        },
        Form::Hortative => match p {
            3 | 6 => indicative(r, p),
            _ => return None,
        },
        Form::Jussive => match p {
            0 => join(r, "chi"),
            2 => format!("{r}pe"),
            5 => format!("{r}pegu"),
            8 => format!("{r}pegün"),
            _ => return None,
        },
        Form::Prohibitive => match p {
            1 => format!("{r}kilge"),
            4 => format!("{r}kilmu"),
            7 => format!("{r}kilmün"),
            _ => return None,
        },
        Form::Negative => indicative(&format!("{r}la"), p),
        Form::NegativeFuture => indicative(&format!("{r}laya"), p),
        Form::NegativeHypothetical => format!("{r}nol{}", HYPOTHETICAL[p]),
    })
}

/// Lowercase, Mapudungun letters only.
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

/// The root of a dictionary form: txipan → txipa, küzawün → küzaw,
/// katxün → katxü (tx can't end a syllable). A bare root stays as it is.
fn root_of(word: &str) -> std::result::Result<String, Error> {
    // Consonants a root can end in: lef, kon, kim, küzaw, magel, nag, chupaz…
    const FINALS: &str = "dfghklmnñqrswyz";
    let root = if let Some(r) = word.strip_suffix("ün") {
        match r.chars().last() {
            Some(c) if FINALS.contains(c) => r.to_string(),
            _ => format!("{r}ü"),
        }
    } else if let Some(r) = word.strip_suffix('n').filter(|r| r.chars().last().is_some_and(is_vowel)) {
        r.to_string()
    } else {
        word.to_string()
    };
    if root.is_empty() || !root.chars().any(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(root)
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    fn entry_root(e: &Entry) -> Option<String> {
        e.root.clone().or_else(|| root_of(&e.entry.to_lowercase()).ok())
    }

    /// The entry for a typed word: the dictionary form (küzawün) or the root (küzaw).
    fn find(&self, word: &str) -> Option<&Entry> {
        self.lexicon
            .iter()
            .find(|e| e.entry.to_lowercase() == word)
            .or_else(|| self.lexicon.iter().find(|e| Self::entry_root(e).as_deref() == Some(word)))
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.find(&word);
        let root = match entry {
            Some(e) => Self::entry_root(e).ok_or(Error::InvalidVerb)?,
            None => root_of(&word)?,
        };

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
                        // Rows: 1, 2, 3 (singular / plural) and 1d, 2d, 3d
                        // (dual, in the plural column).
                        let (sg, pl) = match person.id.as_str() {
                            "1" => (Some(0), Some(6)),
                            "1d" => (None, Some(3)),
                            "2" => (Some(1), Some(7)),
                            "2d" => (None, Some(4)),
                            "3d" => (None, Some(5)),
                            _ => (Some(2), Some(8)),
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
            root: entry.map(|e| e.entry.clone()).unwrap_or_else(|| indicative(&root, 0)),
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

    fn nine(root: &str, form: Form) -> Vec<String> {
        (0..9).map(|p| build(root, form, p).unwrap_or_default()).collect()
    }

    #[test]
    fn indicative_forms() {
        // §7.2: txipan … txipaygün (1pl txipaiñ ~ txipayiñ).
        assert_eq!(
            nine("txipa", Form::Indicative),
            ["txipan", "txipaymi", "txipay", "txipayu", "txipaymu", "txipaygu", "txipayiñ", "txipaymün", "txipaygün"]
        );
        // §7.4 b, c: lefimi, chalimi; §3.1: küzawi «trabaja».
        assert_eq!(build("lef", Form::Indicative, 1).unwrap(), "lefimi");
        assert_eq!(build("lef", Form::Indicative, 0).unwrap(), "lefün");
        assert_eq!(build("chali", Form::Indicative, 1).unwrap(), "chalimi");
        assert_eq!(build("küzaw", Form::Indicative, 2).unwrap(), "küzawi");
        assert_eq!(build("kim", Form::Indicative, 6).unwrap(), "kimiyiñ");
    }

    #[test]
    fn tenses() {
        // §7.5.2.1: küzawan, küzawayiñ, küzawaygün; txipayan.
        assert_eq!(build("küzaw", Form::Future, 0).unwrap(), "küzawan");
        assert_eq!(build("küzaw", Form::Future, 6).unwrap(), "küzawayiñ");
        assert_eq!(build("küzaw", Form::Future, 8).unwrap(), "küzawaygün");
        assert_eq!(build("txipa", Form::Future, 0).unwrap(), "txipayan");
        // §7.5.2.2: niefun, konfuygün; §7.5.2.3: txipayafun, nieafun, akuafuy.
        assert_eq!(build("nie", Form::Past, 0).unwrap(), "niefun");
        assert_eq!(build("kon", Form::Past, 8).unwrap(), "konfuygün");
        assert_eq!(build("txipa", Form::Conditional, 0).unwrap(), "txipayafun");
        assert_eq!(build("nie", Form::Conditional, 0).unwrap(), "nieafun");
        assert_eq!(build("aku", Form::Conditional, 2).unwrap(), "akuafuy");
        // §7.6.2: amulen, kutxanküley; §7.5.4.2: amuken, akuwyeygün, amuwyen.
        assert_eq!(build("amu", Form::Progressive, 0).unwrap(), "amulen");
        assert_eq!(build("kutxan", Form::Progressive, 2).unwrap(), "kutxanküley");
        assert_eq!(build("amu", Form::Habitual, 0).unwrap(), "amuken");
        assert_eq!(build("aku", Form::Perfective, 8).unwrap(), "akuwyeygün");
        assert_eq!(build("amu", Form::Perfective, 0).unwrap(), "amuwyen");
    }

    #[test]
    fn moods() {
        // §7.3: txipali … txipalegün; §7.4 a: afülmu.
        assert_eq!(
            nine("txipa", Form::Hypothetical),
            ["txipali", "txipalmi", "txipale", "txipaliyu", "txipalmu", "txipalegu", "txipaliñ", "txipalmün", "txipalegün"]
        );
        assert_eq!(build("af", Form::Hypothetical, 4).unwrap(), "afülmu");
        // §7.4: txipachi, txipage, txipamu, txipamün; konüge, lelige.
        assert_eq!(build("txipa", Form::Jussive, 0).unwrap(), "txipachi");
        assert_eq!(build("txipa", Form::Imperative, 1).unwrap(), "txipage");
        assert_eq!(build("txipa", Form::Imperative, 7).unwrap(), "txipamün");
        assert_eq!(build("kon", Form::Imperative, 1).unwrap(), "konüge");
        assert_eq!(build("leli", Form::Imperative, 1).unwrap(), "lelige");
        assert_eq!(build("txipa", Form::Imperative, 0), None);
        // Answers to ex. 17: konpape, lefpeyegün (here lefpe, lefpegün).
        assert_eq!(build("lef", Form::Jussive, 2).unwrap(), "lefpe");
        assert_eq!(build("txipa", Form::Hortative, 3).unwrap(), "txipayu");
    }

    #[test]
    fn negation() {
        // §7.5.1: txipalan, txipalaymi, txipalay, txipalayiñ; leflan, kimlan.
        assert_eq!(
            nine("txipa", Form::Negative),
            ["txipalan", "txipalaymi", "txipalay", "txipalayu", "txipalaymu", "txipalaygu", "txipalayiñ", "txipalaymün", "txipalaygün"]
        );
        assert_eq!(build("lef", Form::Negative, 0).unwrap(), "leflan");
        // txipalayaymi «no saldrás», txipanoli «si no salgo», küzawnoli,
        // txekakilge «no camines», amukilmu.
        assert_eq!(build("txipa", Form::NegativeFuture, 1).unwrap(), "txipalayaymi");
        assert_eq!(build("txipa", Form::NegativeHypothetical, 0).unwrap(), "txipanoli");
        assert_eq!(build("küzaw", Form::NegativeHypothetical, 0).unwrap(), "küzawnoli");
        assert_eq!(build("txeka", Form::Prohibitive, 1).unwrap(), "txekakilge");
        assert_eq!(build("amu", Form::Prohibitive, 4).unwrap(), "amukilmu");
    }

    #[test]
    fn input() {
        assert_eq!(normalize(" Küzawün ").unwrap(), "küzawün");
        assert!(normalize("amu1").is_err());
        assert_eq!(root_of("txipan").unwrap(), "txipa");
        assert_eq!(root_of("küzawün").unwrap(), "küzaw");
        assert_eq!(root_of("katxün").unwrap(), "katxü");
        assert_eq!(root_of("in").unwrap(), "i");
        assert_eq!(root_of("kim").unwrap(), "kim");
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/mapudungun.json")).unwrap();
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
        assert!(c.conjugate("küzaw").unwrap().known);
        assert_eq!(c.conjugate("anün").unwrap().tables[0].rows[0].singular, "anün");
        assert_eq!(c.conjugate("küzawün").unwrap().tables[0].rows[0].singular, "küzawün");
    }
}
