//! Conjugation for Paraguayan Guarani (avañe'ẽ), after Estigarribia, *A
//! Grammar of Paraguayan Guarani* (2020) and Guasch, *El idioma guaraní*.
//!
//! A verb is a person prefix and a root, followed by suffixes for tense,
//! aspect and mood:
//!
//! ```text
//! areal (active)   a- re- o- ja-/ña- ro- pe- o-    ajapo «hago»
//! aireal           ai- rei- oi- jai- roi- pei- oi- aikuaa «sé»
//! chendal          che- nde- i- ñande- ore- pende- i-   chepochy «me enojo»
//! ```
//!
//! Nasal roots (mba'apo, ñe'ẽ, mano) nasalize the prefixes: ña-, ne-,
//! ñane-, pene-, and the negation n-…-i instead of nd-…-i.
//!
//! Stress falls on the last syllable unless an acute says otherwise, and a
//! nasal tilde also marks it. When an unstressed suffix follows (-ta,
//! -kuri, -ma, -i), the root keeps its stress and gets an acute: ajapo →
//! ajapóta, ndajapói. Stressed suffixes (-se, -va'erã, -mo'ã, -pota) take
//! the stress: ajapose.

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// ajapo «hago, hice».
    Present,
    /// ajapókuri «hice (hace poco)».
    RecentPast,
    /// ajapova'ekue «hice (hace tiempo)».
    RemotePast,
    /// ajapóta «haré».
    Future,
    /// ajapova'erã «tengo que hacer».
    Obligative,
    /// ajapóma «ya hice».
    Already,
    /// ajapo hína «estoy haciendo».
    Progressive,
    /// ajapose «quiero hacer».
    Desiderative,
    /// ajapopota «estoy por hacer».
    Imminent,
    /// ajapomi «hacía».
    PastHabitual,
    /// ndajapói «no hago».
    Negative,
    /// ndajapomo'ãi «no haré».
    NegativeFuture,
    /// ejapo «¡haz!», pejapo «¡hagan!».
    Imperative,
    /// ani rejapo «¡no hagas!».
    Prohibitive,
    /// tajapo «que yo haga».
    Optative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    /// a-, re-, o-…
    #[default]
    Areal,
    /// ai-, rei-, oi-…: the root takes an i- (kuaa → aikuaa).
    Aireal,
    /// che-, nde-, i-… (inactive).
    Chendal,
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
    /// The root: "japo", "mba'apo", "kuaa".
    pub entry: String,
    pub meaning: String,
    #[serde(default)]
    pub es: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub class: Class,
    /// Overrides the nasal detection.
    #[serde(default)]
    pub nasal: Option<bool>,
    /// Irregular verbs: the seven forms (che, nde, ha'e, ñande, ore, peẽ,
    /// ha'ekuéra) and the 2nd singular imperative: aha, reho, oho… eho.
    #[serde(default)]
    pub forms: Option<Vec<String>>,
    #[serde(default)]
    pub imperative: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conjugation {
    pub persons: Vec<Person>,
    pub tenses: Vec<Tense>,
    #[serde(default)]
    pub lexicon: Vec<Entry>,
}

const NASAL_VOWELS: &str = "ãẽĩõũỹ";

fn is_vowel(c: char) -> bool {
    "aeiouyáéíóúýãẽĩõũỹ".contains(c)
}

/// A root is nasal if it has a nasal vowel or consonant (m, n, ñ, also in
/// mb, nd, ng): mba'apo, ñe'ẽ, hendu.
fn detect_nasal(root: &str) -> bool {
    root.chars().any(|c| NASAL_VOWELS.contains(c) || "mnñ".contains(c) || c == '\u{303}')
}

/// The seven persons: che, nde, ha'e, ñande, ore, peẽ, ha'ekuéra.
const AREAL: [&str; 7] = ["a", "re", "o", "ja", "ro", "pe", "o"];
const CHENDAL: [&str; 7] = ["che", "nde", "i", "ñande", "ore", "pende", "i"];
const CHENDAL_NASAL: [&str; 7] = ["che", "ne", "i", "ñane", "ore", "pene", "i"];

/// The verb before any suffix: the prefix and the root.
struct Verb {
    class: Class,
    nasal: bool,
    root: String,
    forms: Option<Vec<String>>,
    imperative: Option<String>,
}

impl Verb {
    fn prefix(&self, p: usize) -> String {
        match self.class {
            Class::Chendal if self.nasal => CHENDAL_NASAL[p].to_string(),
            Class::Chendal => CHENDAL[p].to_string(),
            _ if p == 3 && self.nasal => "ña".to_string(),
            _ => AREAL[p].to_string(),
        }
    }

    fn stem(&self) -> String {
        match self.class {
            Class::Aireal => format!("i{}", self.root),
            _ => self.root.clone(),
        }
    }

    /// The person form: ajapo, reikuaa, chepochy.
    fn base(&self, p: usize) -> String {
        if let Some(f) = self.forms.as_ref().and_then(|f| f.get(p)) {
            return f.clone();
        }
        format!("{}{}", self.prefix(p), self.stem())
    }

    /// nd-a, nde-re, nd-o, nda-ja, ndo-ro, nda-pe; nda-che for chendal.
    fn negation(&self, p: usize) -> String {
        let nd = if self.nasal { "n" } else { "nd" };
        let vowel = match self.class {
            Class::Chendal => "a",
            _ => ["", "e", "", "a", "o", "a", ""][p],
        };
        format!("{nd}{vowel}")
    }

    /// A puso before a chendal person that starts with a vowel: nda'i-,
    /// nda'ore- (nda'itarovái «no están locos»).
    fn glottal(&self, front: String, p: usize) -> String {
        let vowel = self.base(p).starts_with(|c: char| is_vowel(c));
        if self.class == Class::Chendal && vowel { format!("{front}'") } else { front }
    }

    /// t-a, te-re, t-o, ta-ja, to-ro, ta-pe; ta-che for chendal.
    fn optative(&self, p: usize) -> String {
        let vowel = match self.class {
            Class::Chendal => "a",
            _ => ["", "e", "", "a", "o", "a", ""][p],
        };
        format!("t{vowel}")
    }
}

/// A piece of a word and whether it carries the stress.
#[derive(Clone)]
struct Morph {
    text: String,
    stressed: bool,
}

fn m(text: impl Into<String>, stressed: bool) -> Morph {
    Morph { text: text.into(), stressed }
}

fn has_mark(s: &str) -> bool {
    s.chars().any(|c| "áéíóúý".contains(c) || NASAL_VOWELS.contains(c) || c == '\u{303}')
}

/// The acute on the last vowel: japo → japó.
fn accent_last(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let Some(i) = chars.iter().rposition(|&c| is_vowel(c)) else { return s.to_string() };
    let acute = match chars[i] {
        'a' => 'á',
        'e' => 'é',
        'i' => 'í',
        'o' => 'ó',
        'u' => 'ú',
        'y' => 'ý',
        other => other,
    };
    chars[..i].iter().chain(std::iter::once(&acute)).chain(chars[i + 1..].iter()).collect()
}

/// Joins the pieces, writing the acute when the stress isn't last.
fn join(parts: &[Morph]) -> String {
    let last_stressed = parts.iter().rposition(|p| p.stressed);
    parts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if Some(i) == last_stressed && i + 1 < parts.len() && !has_mark(&p.text) {
                accent_last(&p.text)
            } else {
                p.text.clone()
            }
        })
        .collect()
}

fn ends_in_i(parts: &[Morph]) -> bool {
    parts
        .last()
        .and_then(|p| p.text.chars().last())
        .is_some_and(|c| matches!(c, 'i' | 'í' | 'ĩ'))
}

/// -pota, or -mbota after a nasal root.
fn pota(nasal: bool) -> &'static str {
    if nasal { "mbota" } else { "pota" }
}

fn build(v: &Verb, form: Form, p: usize) -> Option<String> {
    let base = m(v.base(p), true);
    let they = if p == 6 { " hikuái" } else { "" };
    let word = |suffixes: &[Morph]| {
        let mut parts = vec![base.clone()];
        parts.extend_from_slice(suffixes);
        Some(format!("{}{they}", join(&parts)))
    };
    let negative = |middle: &[Morph]| {
        let mut parts = vec![m(v.glottal(v.negation(p), p), false), base.clone()];
        parts.extend_from_slice(middle);
        let i = if ends_in_i(&parts) { "ri" } else { "i" };
        parts.push(m(i, false));
        Some(format!("{}{they}", join(&parts)))
    };
    match form {
        Form::Present => word(&[]),
        Form::RecentPast => word(&[m("kuri", false)]),
        Form::RemotePast => word(&[m("va'ekue", true)]),
        Form::Future => word(&[m("ta", false)]),
        Form::Obligative => word(&[m("va'erã", true)]),
        Form::Already => word(&[m("ma", false)]),
        Form::Progressive => word(&[]).map(|w| match w.strip_suffix(" hikuái") {
            Some(w) => format!("{w} hína hikuái"),
            None => format!("{w} hína"),
        }),
        Form::Desiderative if v.class == Class::Chendal => None,
        Form::Desiderative => word(&[m("se", true)]),
        Form::Imminent => word(&[m(pota(v.nasal), true)]),
        Form::PastHabitual => word(&[m("mi", true)]),
        Form::Negative => negative(&[]),
        Form::NegativeFuture => negative(&[m("mo'ã", true)]),
        Form::Imperative => match (p, v.class) {
            (1, Class::Chendal) => Some(v.base(1)),
            (1, _) => Some(v.imperative.clone().unwrap_or_else(|| format!("e{}", v.stem()))),
            (5, _) => Some(v.base(5)),
            _ => None,
        },
        Form::Prohibitive => match p {
            1 | 5 => Some(format!("ani {}", v.base(p))),
            _ => None,
        },
        Form::Optative => Some(format!("{}{}{they}", v.glottal(v.optative(p), p), v.base(p))),
    }
}

fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb
        .trim()
        .to_lowercase()
        .replace(['’', 'ʼ', '´', '`', '‘'], "'")
        .nfc_simple();
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_alphabetic() || c == '\'' || c == '\u{303}')
        || w.chars().count() > 40
        || !w.chars().any(is_vowel)
    {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

/// Composes a vowel + combining tilde or acute typed separately (e + ̃).
trait Nfc {
    fn nfc_simple(&self) -> String;
}

impl Nfc for String {
    fn nfc_simple(&self) -> String {
        const TILDE: &str = "aãeẽiĩoõuũyỹ";
        const ACUTE: &str = "aáeéiíoóuúyý";
        let mut out = String::new();
        for c in self.chars() {
            let last = out.pop();
            let table = match c {
                '\u{303}' => Some(TILDE),
                '\u{301}' => Some(ACUTE),
                _ => None,
            };
            let joined = match (last, table) {
                (Some(v), Some(t)) => {
                    let cs: Vec<char> = t.chars().collect();
                    cs.iter().position(|&x| x == v).filter(|i| i % 2 == 0).map(|i| cs[i + 1])
                }
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

    fn find(&self, word: &str) -> Option<&Entry> {
        self.lexicon.iter().find(|e| {
            e.entry == word
                || (e.class == Class::Aireal && format!("i{}", e.entry) == word)
                || e.forms.as_ref().is_some_and(|f| f.iter().any(|x| x == word))
        })
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.find(&word);
        let class = match (entry, opts.kind.as_deref()) {
            (Some(e), _) => e.class,
            (None, Some("aireal")) => Class::Aireal,
            (None, Some("chendal")) => Class::Chendal,
            _ => Class::Areal,
        };
        let root = entry.map(|e| e.entry.clone()).unwrap_or_else(|| {
            // An aireal root typed with its i- (ikuaa): keep kuaa.
            match class {
                Class::Aireal => word.strip_prefix('i').filter(|r| !r.is_empty()).unwrap_or(&word).to_string(),
                _ => word.clone(),
            }
        });
        let verb = Verb {
            class,
            nasal: entry.and_then(|e| e.nasal).unwrap_or_else(|| detect_nasal(&root)),
            root: root.clone(),
            forms: entry.and_then(|e| e.forms.clone()),
            imperative: entry.and_then(|e| e.imperative.clone()),
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
                        // Rows: 1 (che / ñande), 1x (— / ore), 2 (nde / peẽ), 3.
                        let (sg, pl) = match person.id.as_str() {
                            "1" => (Some(0), Some(3)),
                            "1x" => (None, Some(4)),
                            "2" => (Some(1), Some(5)),
                            _ => (Some(2), Some(6)),
                        };
                        let sg = sg.and_then(|p| build(&verb, tense.form, p));
                        let pl = pl.and_then(|p| build(&verb, tense.form, p));
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

    fn verb(root: &str, class: Class) -> Verb {
        Verb { class, nasal: detect_nasal(root), root: root.into(), forms: None, imperative: None }
    }

    fn seven(v: &Verb, form: Form) -> Vec<String> {
        (0..7).map(|p| build(v, form, p).unwrap_or_default()).collect()
    }

    #[test]
    fn persons() {
        let japo = verb("japo", Class::Areal);
        assert_eq!(
            seven(&japo, Form::Present),
            ["ajapo", "rejapo", "ojapo", "jajapo", "rojapo", "pejapo", "ojapo hikuái"]
        );
        // Nasal roots: ña- (Estigarribia 4.1).
        assert_eq!(build(&verb("guahẽ", Class::Areal), Form::Present, 3).unwrap(), "ñaguahẽ");
        assert_eq!(build(&verb("mba'apo", Class::Areal), Form::Present, 3).unwrap(), "ñamba'apo");
        // Aireal: oikuaa hikuái.
        assert_eq!(build(&verb("kuaa", Class::Aireal), Form::Present, 6).unwrap(), "oikuaa hikuái");
        // Chendal: nderesarái, ñaneras~e, penememby.
        let pochy = verb("pochy", Class::Chendal);
        assert_eq!(seven(&pochy, Form::Present)[..6], ["chepochy", "ndepochy", "ipochy", "ñandepochy", "orepochy", "pendepochy"]);
        assert_eq!(build(&verb("kane'õ", Class::Chendal), Form::Present, 5).unwrap(), "penekane'õ");
    }

    #[test]
    fn negation() {
        // Estigarribia 4.9: jogua, maña, jopi, motĩ.
        assert_eq!(
            seven(&verb("jogua", Class::Areal), Form::Negative)[..6],
            ["ndajoguái", "nderejoguái", "ndojoguái", "ndajajoguái", "ndorojoguái", "ndapejoguái"]
        );
        assert_eq!(
            seven(&verb("maña", Class::Areal), Form::Negative)[..6],
            ["namañái", "neremañái", "nomañái", "nañamañái", "noromañái", "napemañái"]
        );
        assert_eq!(build(&verb("jopi", Class::Areal), Form::Negative, 0).unwrap(), "ndajopíri");
        assert_eq!(build(&verb("motĩ", Class::Areal), Form::Negative, 1).unwrap(), "neremotĩri");
        assert_eq!(build(&verb("pochy", Class::Chendal), Form::Negative, 0).unwrap(), "ndachepochýi");
        assert_eq!(build(&verb("tarova", Class::Chendal), Form::Negative, 2).unwrap(), "nda'itarovái");
        assert_eq!(build(&verb("kuaa", Class::Aireal), Form::Negative, 0).unwrap(), "ndaikuaái");
        assert_eq!(build(&verb("mba'apo", Class::Areal), Form::NegativeFuture, 0).unwrap(), "namba'apomo'ãi");
    }

    #[test]
    fn stress() {
        let v = verb("mba'apo", Class::Areal);
        assert_eq!(build(&v, Form::Future, 0).unwrap(), "amba'apóta");
        assert_eq!(build(&verb("guahẽ", Class::Areal), Form::Already, 2).unwrap(), "oguahẽma");
        assert_eq!(build(&verb("kuaa", Class::Aireal), Form::Already, 5).unwrap(), "peikuaáma");
        assert_eq!(build(&verb("ha", Class::Areal), Form::Desiderative, 0).unwrap(), "ahase");
        assert_eq!(build(&verb("japo", Class::Areal), Form::PastHabitual, 0).unwrap(), "ajapomi");
        assert_eq!(build(&verb("mano", Class::Areal), Form::Imminent, 2).unwrap(), "omanombota");
        assert_eq!(build(&verb("ju", Class::Areal), Form::Obligative, 1).unwrap(), "rejuva'erã");
    }

    #[test]
    fn moods() {
        let karu = verb("karu", Class::Areal);
        assert_eq!(build(&karu, Form::Imperative, 1).unwrap(), "ekaru");
        assert_eq!(build(&karu, Form::Imperative, 5).unwrap(), "pekaru");
        assert_eq!(build(&karu, Form::Imperative, 0), None);
        assert_eq!(build(&verb("ho", Class::Areal), Form::Prohibitive, 1).unwrap(), "ani reho");
        // Optative (4.10.3.4.2): tamba'apo, torokaru, tapeho, tereguahẽ.
        assert_eq!(build(&verb("mba'apo", Class::Areal), Form::Optative, 0).unwrap(), "tamba'apo");
        assert_eq!(build(&karu, Form::Optative, 4).unwrap(), "torokaru");
        assert_eq!(build(&verb("guahẽ", Class::Areal), Form::Optative, 1).unwrap(), "tereguahẽ");
    }

    #[test]
    fn normalizing() {
        assert_eq!(normalize("ñe\u{2019}e\u{303}").unwrap(), "ñe'ẽ");
        assert!(normalize("aj4po").is_err());
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/guarani.json")).unwrap();
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
        // Irregular ho «ir»: aha, reho, oho.
        let r = c.conjugate("ho").unwrap();
        assert_eq!(r.tables[0].rows[0].singular, "aha");
        assert_eq!(r.tables[0].rows[2].plural.as_deref(), Some("peho"));
    }
}
