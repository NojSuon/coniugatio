//! Conjugation for Ch'ol (lakty'añ, Mayan), after Viola Warkentin and Ruby
//! Scott, *Gramática ch'ol* (ILV, 1980), the grammar of Aulie and Aulie's
//! *Diccionario ch'ol* (ILV, 3rd ed. 2009) and Jessica Coon, «Ch'ol» (2015).
//! Those sources write the old ILV spelling (c, qu, ʌ, ty); here every form
//! is in the INALI spelling of the Norma de escritura (k, ä, ty, ñ), as in
//! the Vocabulario de la lengua ch'ol (UNICH, 2013), and in the Tila forms
//! (tsa', chonkol, mi kej) the grammar gives next to Tumbalá's (tsi', woli,
//! mi kaj).
//!
//! The verb takes an aspect word and two sets of person markers: Set A
//! prefixes (k-, a-, i-) and Set B suffixes (-oñ, -ety, Ø). Ch'ol has a
//! split by aspect: in the perfective (and the perfect) an intransitive
//! subject is Set B; elsewhere it is Set A, on a nominal -el stem.
//!
//! ```text
//!               intransitive majl «go»     transitive mäñ «buy»
//! perfective    tsa' majliyoñ «I went»    tsa' kmäñä «I bought it»
//! imperfective  mi kmajlel «I go»          mi kmäñ «I buy it»
//! perfect       majlemoñ «I have gone»     mäñäl kcha'añ «I have bought it»
//! ```
//!
//! Plurals, as the Norma writes them: lak- / -oñla «we (with you)»,
//! k-…-lojoñ / -oñlojoñ «we (not you)», la'a- / -etyla «you all», -ob «they».
//! Root transitives take a vowel that echoes the root (k'ux-u, mek'-e; a
//! gives ä, except after j: mäñ-ä, tyaj-a) in the perfective and the
//! imperative; derived transitives take a set vowel and -Vñ (il-ä / il-äñ).

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
    /// mi kej kmajlel «I will go».
    Future,
    /// majlemoñ «I have gone», mäñäl kcha'añ «I have bought it».
    Perfect,
    /// majlemoñix «I had gone», mäñälix kcha'añ «I had bought it».
    Pluperfect,
    /// letsen «go up!», k'uxu «eat it!», plural -la (2nd person only).
    Imperative,
    /// ma'añ tsa' majliyoñ «I didn't go».
    NegativePerfective,
    /// ma'añ mi kmajlel «I don't go».
    NegativeImperfective,
    /// ma'añ mi kej kmajlel «I won't go».
    NegativeFuture,
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
/// Set A prefix and the plural clitic that goes with it.
fn set_a(p: usize, stem: &str) -> (&'static str, &'static str) {
    let v = starts_with_vowel(stem);
    let prefix = match p {
        // k- is j- before k: jk'ux, jkuch (the Norma writes kjats', kjol).
        0 | 4 if !v && stem.starts_with('k') => "j",
        0 | 4 => "k",
        3 => "lak",
        1 if v => "aw",
        1 => "a",
        5 if v => "la'aw",
        5 => "la'a",
        _ if v => "y",
        _ => "i",
    };
    let plural = match p {
        4 => "lojoñ",
        6 => "ob",
        _ => "",
    };
    (prefix, plural)
}

/// Set B for persons 0–6, as the person suffix and the plural clitic:
/// -oñ, -ety, Ø, -oñ-la, -oñ-lojoñ, -ety-la, -ob.
const SET_B: [(&str, &str); 7] = [
    ("oñ", ""),
    ("ety", ""),
    ("", ""),
    ("oñ", "la"),
    ("oñ", "lojoñ"),
    ("ety", "la"),
    ("ob", ""),
];

/// Adds a suffix, with y between vowels (majli-y-oñ, k'uxu-y-ob), and
/// -lojoñ after l as -ojoñ (kmajlel-ojoñ, as the Norma's mi kwäyelojoñ).
fn add(stem: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        return stem.to_string();
    }
    if suffix == "lojoñ" && stem.ends_with('l') {
        return format!("{stem}ojoñ");
    }
    let last = stem.chars().last().is_some_and(is_vowel);
    if last && starts_with_vowel(suffix) {
        format!("{stem}y{suffix}")
    } else {
        format!("{stem}{suffix}")
    }
}

/// The echo vowel of a root transitive: k'ux → u, mek' → e. An a gives ä
/// (mäñ-ä, jats'-ä), except after a final j (tyaj-a, as in tsa'ix ityaja).
fn echo(root: &str) -> String {
    match root.chars().filter(|&c| is_vowel(c)).last().unwrap_or('a') {
        'a' if !root.ends_with('j') => "ä".to_string(),
        v => v.to_string(),
    }
}

/// A verb with Set A for person `p`: k-mäñ, la'a-mäñ, i-mäñ-ob.
fn with_a(stem: &str, p: usize) -> String {
    let (a, pl) = set_a(p, stem);
    add(&format!("{a}{stem}"), pl)
}

/// A stem with Set B for person `p`, and -ix «already» before the plural
/// clitic: majlem-oñ-ix-la, majlem-ob-ix.
fn with_b(stem: &str, p: usize, ix: bool) -> String {
    let (person, clitic) = SET_B[p];
    let mut w = add(stem, person);
    if ix {
        w.push_str("ix");
    }
    add(&w, clitic)
}

/// The verb for person `p` (0–6), with the aspect word. Empty for a person
/// the form doesn't have (the imperative is 2nd person only).
fn build(root: &str, kind: Kind, vowel: &str, form: Form, p: usize) -> String {
    let transitive = kind != Kind::Intransitive;
    let (aspect, negative) = match form {
        Form::Perfective => ("tsa'", false),
        Form::PerfectiveAlready => ("tsa'ix", false),
        Form::Imperfective => ("mi", false),
        Form::Progressive => ("chonkol", false),
        Form::Future => ("mi kej", false),
        Form::Perfect | Form::Pluperfect | Form::Imperative => ("", false),
        Form::NegativePerfective => ("tsa'", true),
        Form::NegativeImperfective => ("mi", true),
        Form::NegativeFuture => ("mi kej", true),
    };
    let verb = match form {
        Form::Imperative => {
            if p != 1 && p != 5 {
                return String::new();
            }
            // letsen, ochen; k'uxu, mäñä; iläñ. Plural -la.
            let stem = match kind {
                Kind::Intransitive => format!("{root}en"),
                Kind::Transitive => format!("{root}{vowel}"),
                Kind::Derived => format!("{root}{vowel}ñ"),
            };
            if p == 5 { format!("{stem}la") } else { stem }
        }
        Form::Perfect | Form::Pluperfect => {
            let ix = form == Form::Pluperfect;
            if transitive {
                // Tila mäñäl kcha'añ «I have bought it» (Tumbalá mäñbil).
                let participle = format!("{root}äl{}", if ix { "ix" } else { "" });
                format!("{participle} {}", with_a("cha'añ", p))
            } else {
                // -em, but -eñ after a final m: majlem, chämeñ.
                let suffix = if root.ends_with('m') { "eñ" } else { "em" };
                with_b(&format!("{root}{suffix}"), p, ix)
            }
        }
        _ => {
            let perfective = matches!(form, Form::Perfective | Form::PerfectiveAlready | Form::NegativePerfective);
            match kind {
                // Perfective: Set B on the -i stem. Otherwise: Set A on -el.
                Kind::Intransitive if perfective => with_b(&format!("{root}i"), p, false),
                Kind::Intransitive => with_a(&format!("{root}el"), p),
                // Transitives: Set A subject, 3rd person object (Set B Ø).
                _ => {
                    let stem = match (kind, perfective) {
                        (_, true) => format!("{root}{vowel}"),
                        (Kind::Derived, false) => format!("{root}{vowel}ñ"),
                        _ => root.to_string(),
                    };
                    with_a(&stem, p)
                }
            }
        }
    };
    match (aspect, negative) {
        ("", _) => verb,
        (a, false) => format!("{a} {verb}"),
        (a, true) => format!("ma'añ {a} {verb}"),
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
                        let form = |p| Some(build(&root, kind, &vowel, tense.form, p)).filter(|s| !s.is_empty());
                        let sg = sg.and_then(form);
                        let pl = pl.and_then(form);
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
        // Warkentin & Scott §9.4: ti majliyon, majliyonla, majliyon lojon…;
        // the Norma writes -oñlojoñ as one word.
        assert_eq!(
            seven("majl", Kind::Intransitive, "", Form::Perfective),
            [
                "tsa' majliyoñ", "tsa' majliyety", "tsa' majli", "tsa' majliyoñla",
                "tsa' majliyoñlojoñ", "tsa' majliyetyla", "tsa' majliyob"
            ]
        );
        // §9.4.6 mic majlel, mi lac majlel, mi la' majlel, mi' majlelob; the
        // Norma: mi kwäyelojoñ, la'awäyel; the Vocabulario: mi kmajlelojoñ.
        assert_eq!(
            seven("majl", Kind::Intransitive, "", Form::Imperfective),
            [
                "mi kmajlel", "mi amajlel", "mi imajlel", "mi lakmajlel",
                "mi kmajlelojoñ", "mi la'amajlel", "mi imajlelob"
            ]
        );
        assert_eq!(build("och", Kind::Intransitive, "", Form::Imperfective, 2), "mi yochel");
        assert_eq!(build("och", Kind::Intransitive, "", Form::Imperfective, 5), "mi la'awochel");
        // §9.4.3 mi quej c majlel.
        assert_eq!(build("majl", Kind::Intransitive, "", Form::Future, 0), "mi kej kmajlel");
        assert_eq!(build("majl", Kind::Intransitive, "", Form::NegativeFuture, 0), "ma'añ mi kej kmajlel");
        // §9.4.4–5 majlemon, majlemonla, majlemob; majlemonix, majlemetixla, majlemobix.
        assert_eq!(
            seven("majl", Kind::Intransitive, "", Form::Perfect),
            ["majlemoñ", "majlemety", "majlem", "majlemoñla", "majlemoñlojoñ", "majlemetyla", "majlemob"]
        );
        assert_eq!(build("majl", Kind::Intransitive, "", Form::Pluperfect, 5), "majlemetyixla");
        assert_eq!(build("majl", Kind::Intransitive, "", Form::Pluperfect, 6), "majlemobix");
        // Diccionario §2.8: chʌmen «muerto»; the Vocabulario: chämeñ.
        assert_eq!(build("chäm", Kind::Intransitive, "", Form::Perfect, 2), "chämeñ");
        // Diccionario §2.9: letsen, ochen; plural -la.
        assert_eq!(build("lets", Kind::Intransitive, "", Form::Imperative, 1), "letsen");
        assert_eq!(build("lets", Kind::Intransitive, "", Form::Imperative, 5), "letsenla");
        assert_eq!(build("lets", Kind::Intransitive, "", Form::Imperative, 0), "");
    }

    #[test]
    fn transitives() {
        // §6.1: chancol c man, ti c maña, ti la' maña, ti' mañayob.
        assert_eq!(
            seven("mäñ", Kind::Transitive, "ä", Form::Perfective),
            [
                "tsa' kmäñä", "tsa' amäñä", "tsa' imäñä", "tsa' lakmäñä",
                "tsa' kmäñälojoñ", "tsa' la'amäñä", "tsa' imäñäyob"
            ]
        );
        assert_eq!(build("mäñ", Kind::Transitive, "ä", Form::Progressive, 0), "chonkol kmäñ");
        assert_eq!(build("mäñ", Kind::Transitive, "ä", Form::Future, 6), "mi kej imäñob");
        // §6.1.5 mañal c cha'an, mañal la' cha'an; §6.1.6 mañalix c cha'an.
        assert_eq!(build("mäñ", Kind::Transitive, "ä", Form::Perfect, 0), "mäñäl kcha'añ");
        assert_eq!(build("mäñ", Kind::Transitive, "ä", Form::Perfect, 5), "mäñäl la'acha'añ");
        assert_eq!(build("mäñ", Kind::Transitive, "ä", Form::Pluperfect, 6), "mäñälix icha'añob");
        assert_eq!(build("k'ux", Kind::Transitive, "u", Form::Perfective, 0), "tsa' jk'uxu");
        // §6.2: chancol quilan, ti a wilʌ, ti la' wilʌ, chancol i yilañob.
        assert_eq!(build("il", Kind::Derived, "ä", Form::Perfective, 1), "tsa' awilä");
        assert_eq!(build("il", Kind::Derived, "ä", Form::Perfective, 5), "tsa' la'awilä");
        assert_eq!(build("il", Kind::Derived, "ä", Form::Progressive, 6), "chonkol yiläñob");
        assert_eq!(build("ts'ijb", Kind::Derived, "u", Form::Imperfective, 2), "mi its'ijbuñ");
        // §7.1 / Diccionario §2.9: choño, quele, chalen; tajala, quelela.
        assert_eq!(build("choñ", Kind::Transitive, "o", Form::Imperative, 1), "choño");
        assert_eq!(build("k'el", Kind::Transitive, "e", Form::Imperative, 5), "k'elela");
        assert_eq!(build("cha'l", Kind::Derived, "e", Form::Imperative, 1), "cha'leñ");
        assert_eq!(build("cha'l", Kind::Derived, "e", Form::Perfective, 2), "tsa' icha'le");
        // Echo vowels: a → ä except after j (tsa'ix ityaja).
        assert_eq!(echo("k'ux"), "u");
        assert_eq!(echo("jats'"), "ä");
        assert_eq!(echo("tyaj"), "a");
        assert_eq!(echo("päk'"), "ä");
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
