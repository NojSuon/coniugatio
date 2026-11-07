//! Conjugation for Itza' Maya (Itzaj).
//!
//! Source: Charles Andrew Hofling with Fernando Tesucún, *Itzaj Maya Grammar*
//! (University of Utah Press, 2000), §2.1 (person markers) and §3.1–3.2
//! (aspect, status and verb root classes); negation from §19.1.
//!
//! A verb is entered the way Hofling's dictionary lists it, in the
//! incompletive status: `wenel` (intransitive, harmonic -Vl), `ilik` (root
//! transitive, -ik) or `jantik` (transitive derived with -t). The engine then
//! builds every form from the root:
//!
//! ```text
//! incompletive  k-in-wen-el      k-in-w-il-ik
//! durative      tan in-wen-el    tan in-w-il-ik
//! completive    wen-een          t-in-w-il-aj
//! perfect       wen-a'an-en      in-w-il-m-aj
//! assurative    je' in-wen-el-ej je' in-w-il-ik-ej
//! ```

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    /// Intransitive root: wen-el, jok'-ol.
    I,
    /// Root transitive: il-ik, uk'-ik.
    T,
    /// Transitive derived with -t: jan-t-ik, cha'an-t-ik.
    Tt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Aspect {
    /// k- … (habitual present)
    Incompletive,
    /// tan … (ongoing action)
    Durative,
    /// t- … -aj (transitive) / root + Set B (intransitive)
    Completive,
    /// … -m-aj (transitive) / -a'an + Set B (intransitive)
    Perfect,
    /// je' … -ej (the speaker is sure it will happen)
    Assurative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tense {
    pub id: String,
    pub group: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub aspect: Aspect,
    #[serde(default)]
    pub negative: bool,
    #[serde(default)]
    pub es: Option<spanish::Tense>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Dictionary (incompletive) form: "wenel", "ilik", "jantik".
    pub entry: String,
    pub class: Class,
    pub meaning: String,
    #[serde(default)]
    pub es: Option<String>,
    /// Irregular root for the completive and perfect (b'el → b'in).
    #[serde(default)]
    pub completive: Option<String>,
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

const VOWELS: &str = "aäeiou";

fn is_vowel(c: char) -> bool {
    VOWELS.contains(c)
}

fn has_vowel(s: &str) -> bool {
    s.chars().any(is_vowel)
}

/// The verb, analysed from its dictionary form.
struct Verb {
    class: Class,
    /// Bare root: "wen", "il", "jan".
    #[cfg_attr(not(test), allow(dead_code))]
    root: String,
    /// Root used in the completive and perfect (usually the same).
    completive_root: String,
    /// Stem used after the Set A prefix in non-completive aspects:
    /// "wenel", "ilik", "jantik", "b'el".
    incompletive: String,
}

impl Class {
    fn from_kind(kind: &str) -> Option<Class> {
        match kind {
            "I" => Some(Class::I),
            "T" => Some(Class::T),
            "Tt" => Some(Class::Tt),
            _ => None,
        }
    }
}

/// First vowel of the root, which the intransitive suffix repeats: wen-el.
fn harmonic_vowel(root: &str) -> char {
    root.chars().find(|c| is_vowel(*c)).unwrap_or('e')
}

/// Builds a verb once its class is known, from a bare root (wen → wenel,
/// il → ilik, jan → jantik) or from its dictionary form.
fn from_class(word: &str, class: Class) -> std::result::Result<Verb, Error> {
    if !has_vowel(word) {
        return Err(Error::InvalidVerb);
    }
    let verb = match class {
        Class::I => {
            let parsed = parse_as(word, None, Class::I)?;
            if parsed.root != word {
                return Ok(parsed);
            }
            let incompletive = format!("{word}{}l", harmonic_vowel(word));
            Verb { class, root: word.into(), completive_root: word.into(), incompletive }
        }
        Class::T => {
            let root = word.strip_suffix("ik").filter(|r| has_vowel(r)).unwrap_or(word);
            Verb { class, root: root.into(), completive_root: root.into(), incompletive: format!("{root}ik") }
        }
        Class::Tt => {
            let root = word
                .strip_suffix("tik")
                .or_else(|| word.strip_suffix('t'))
                .filter(|r| has_vowel(r))
                .unwrap_or(word);
            Verb { class, root: root.into(), completive_root: root.into(), incompletive: format!("{root}tik") }
        }
    };
    Ok(verb)
}

fn parse(word: &str, entry: Option<&Entry>) -> std::result::Result<Verb, Error> {
    let class = match entry {
        Some(e) => e.class,
        None => detect(word)?,
    };
    parse_as(word, entry, class)
}

/// Tells the class from the dictionary form's ending: -ik, -tik or -Vl.
fn detect(word: &str) -> std::result::Result<Class, Error> {
    Ok(match word {
        _ if word.ends_with("ik") && has_vowel(&word[..word.len() - 2]) => {
            // jan-t-ik, meyaj-t-ik: a consonant before -t marks a derived
            // transitive; root transitives end in vowel + t (sut-ik).
            let stem = &word[..word.len() - 2];
            let before_t = stem.strip_suffix('t').and_then(|s| s.chars().last());
            if before_t.is_some_and(|c| !is_vowel(c) && c != '\'') {
                Class::Tt
            } else {
                Class::T
            }
        }
        _ if word.ends_with('l') && word.chars().count() > 3 => {
            let without_l = &word[..word.len() - 1];
            let last = without_l.chars().last().unwrap();
            let root = &without_l[..without_l.len() - last.len_utf8()];
            if is_vowel(last) && has_vowel(root) {
                Class::I
            } else {
                return Err(Error::UnknownClass);
            }
        }
        _ => return Err(Error::UnknownClass),
    })
}

fn parse_as(word: &str, entry: Option<&Entry>, class: Class) -> std::result::Result<Verb, Error> {
    let root = match class {
        Class::T => word.strip_suffix("ik").unwrap_or(word).to_string(),
        Class::Tt => word.strip_suffix("tik").unwrap_or(word).to_string(),
        Class::I => {
            // Drop the harmonic -Vl (wenel → wen, nak'äl → nak'). Entries
            // without it, like b'el, keep the whole word.
            let w = word.strip_suffix('l').unwrap_or(word);
            match w.chars().last() {
                Some(v) if is_vowel(v) && w.len() > v.len_utf8() && has_vowel(&w[..w.len() - v.len_utf8()]) => {
                    w[..w.len() - v.len_utf8()].to_string()
                }
                _ => word.to_string(),
            }
        }
    };
    let completive_root = entry.and_then(|e| e.completive.clone()).unwrap_or_else(|| root.clone());
    Ok(Verb { class, root, completive_root, incompletive: word.to_string() })
}

/// Set A (ergative) prefix + stem, with the glide before vowels
/// (in-w-il, a-w-il, u-y-il, ki-w-il) and in- → im- before labials.
fn set_a(person: usize, plural: bool, stem: &str) -> String {
    let first = stem.chars().next().unwrap_or('x');
    let prefix = match (person, plural) {
        (0, false) if is_vowel(first) => "inw",
        (0, false) if "b'pm".contains(first) && !stem.starts_with('\'') => "im",
        (0, false) => "in",
        (0, true) if is_vowel(first) => "kiw",
        (0, true) => "ki",
        (1, _) if is_vowel(first) => "aw",
        (1, _) => "a",
        (_, _) if is_vowel(first) => "uy",
        _ => "u",
    };
    format!("{prefix}{stem}")
}

/// Plural suffix that goes with a Set A subject: 2pl -e'ex, 3pl -oo'.
/// The 1st person plural (exclusive) has none.
fn plural_suffix(person: usize, plural: bool) -> &'static str {
    match (person, plural) {
        (1, true) => "e'ex",
        (2, true) => "oo'",
        _ => "",
    }
}

/// Set B (absolutive) suffixes. The long -een/-eech appear on completive
/// intransitives (wen-een "I slept"); the short ones elsewhere.
fn set_b(person: usize, plural: bool, long: bool, third: &'static str) -> &'static str {
    match (person, plural) {
        (0, false) if long => "een",
        (0, false) => "en",
        (1, false) if long => "eech",
        (1, false) => "ech",
        (2, false) => third,
        (0, true) => "o'on",
        (1, true) => "e'ex",
        _ => "oo'",
    }
}

fn form(verb: &Verb, aspect: Aspect, person: usize, plural: bool) -> String {
    let subject = |stem: &str| format!("{}{}", set_a(person, plural, stem), plural_suffix(person, plural));
    let transitive = verb.class != Class::I;
    let t = if verb.class == Class::Tt { "t" } else { "" };
    match aspect {
        Aspect::Incompletive => {
            // k- merges with the 1st person plural ki-.
            let s = subject(&verb.incompletive);
            if s.starts_with("ki") && person == 0 && plural {
                s
            } else {
                format!("k{s}")
            }
        }
        Aspect::Durative => format!("tan {}", subject(&verb.incompletive)),
        Aspect::Assurative => format!("je' {}ej", subject(&verb.incompletive)),
        Aspect::Completive if transitive => {
            let s = subject(&format!("{}{t}aj", verb.completive_root));
            // t- has the form ti- before ki-.
            if person == 0 && plural {
                format!("ti{s}")
            } else {
                format!("t{s}")
            }
        }
        Aspect::Completive => format!("{}{}", verb.completive_root, set_b(person, plural, true, "ij")),
        // In the perfect, -t- is replaced by -m: jan-t-ik → in-jan-m-aj.
        Aspect::Perfect if transitive => subject(&format!("{}maj", verb.completive_root)),
        Aspect::Perfect => format!("{}a'an{}", verb.completive_root, set_b(person, plural, false, "")),
    }
}

/// Lower-cases and writes every glottal stop as a straight apostrophe.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c == 'ʼ' || c == '’' || c == '‘' { '\'' } else { c })
        .collect();
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_alphabetic() || c == '\'') || w.chars().count() > 40 {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let forced = opts.kind().and_then(Class::from_kind);
        // Look the verb up by its dictionary form, or by its bare root
        // (nak' → nak'äl), so lexicon exceptions win over the rules.
        let entry = self.lexicon.iter().find(|e| {
            forced.is_none_or(|c| c == e.class)
                && (e.entry == word || parse_as(&e.entry, Some(e), e.class).is_ok_and(|v| v.root == word))
        });
        let verb = match (entry, forced) {
            (Some(e), _) => parse(&e.entry, Some(e))?,
            (None, Some(class)) => from_class(&word, class)?,
            (None, None) => parse(&word, None)?,
        };
        // A meaning typed by the user wins over the lexicon's.
        let es_meaning = opts.es().map(str::to_string).or_else(|| entry.and_then(|e| e.es.clone()));

        let es = |tense: &Tense, person: &Person, plural: bool| -> Option<String> {
            let meaning = es_meaning.as_deref()?;
            spanish::conjugate(meaning, tense.es?, person.es? + if plural { 3 } else { 0 }, tense.negative)
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
                        // Under negation the incompletive is expressed with the
                        // durative: ma' tan inwenel "I don't sleep" (§19.1.5).
                        let aspect = match (tense.negative, tense.aspect) {
                            (true, Aspect::Incompletive) => Aspect::Durative,
                            (_, a) => a,
                        };
                        let neg = if tense.negative { "ma' " } else { "" };
                        let sg = format!("{neg}{}", form(&verb, aspect, i, false));
                        let pl = format!("{neg}{}", form(&verb, aspect, i, true));
                        Row {
                            person: person.label.clone(),
                            clitic: String::new(),
                            singular: sg,
                            plural: Some(pl),
                            es_singular: es(tense, person, false),
                            es_plural: es(tense, person, true),
                        }
                    })
                    .collect(),
            })
            .collect();

        Ok(Conjugated {
            root: word,
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

    fn itza() -> Conjugation {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/itza.json")).unwrap();
        serde_json::from_value(raw["conjugation"].clone()).unwrap()
    }

    fn v(word: &str) -> Verb {
        parse(word, None).unwrap()
    }

    fn all(word: &str, aspect: Aspect) -> Vec<String> {
        let verb = parse(word, None).unwrap();
        (0..3)
            .flat_map(|p| [form(&verb, aspect, p, false), form(&verb, aspect, p, true)])
            .collect()
    }

    #[test]
    fn parses_dictionary_forms() {
        assert_eq!(v("wenel").class, Class::I);
        assert_eq!(v("wenel").root, "wen");
        assert_eq!(v("nak'äl").root, "nak'");
        assert_eq!(v("ilik").class, Class::T);
        assert_eq!(v("ilik").root, "il");
        assert_eq!(v("jantik").class, Class::Tt);
        assert_eq!(v("jantik").root, "jan");
        assert_eq!(v("meyajtik").class, Class::Tt);
        assert_eq!(v("sutik").class, Class::T);
        assert!(parse("kol", None).is_err());
    }

    // Hofling 2000, §2.1.1 (2) and (4)
    #[test]
    fn incompletive() {
        assert_eq!(all("ilik", Aspect::Incompletive), [
            "kinwilik", "kiwilik", "kawilik", "kawilike'ex", "kuyilik", "kuyilikoo'"
        ]);
        assert_eq!(all("talel", Aspect::Incompletive), [
            "kintalel", "kitalel", "katalel", "katalele'ex", "kutalel", "kutaleloo'"
        ]);
    }

    // §2.1.1 (3), §3.2.2.2; Hofling 1991 §7.2.1.2 (k'och-een)
    #[test]
    fn completive() {
        assert_eq!(all("ilik", Aspect::Completive), [
            "tinwilaj", "tikiwilaj", "tawilaj", "tawilaje'ex", "tuyilaj", "tuyilajoo'"
        ]);
        assert_eq!(all("k'ochol", Aspect::Completive), [
            "k'ocheen", "k'ocho'on", "k'ocheech", "k'oche'ex", "k'ochij", "k'ochoo'"
        ]);
        assert_eq!(form(&v("jantik"), Aspect::Completive, 0, false), "tinjantaj");
    }

    // Hofling 1991 §7.3.1.3, §7.3.2 (in-han-m-ah); §3.1.2 (wen-a'an)
    #[test]
    fn perfect() {
        assert_eq!(form(&v("ilik"), Aspect::Perfect, 0, false), "inwilmaj");
        assert_eq!(form(&v("jantik"), Aspect::Perfect, 0, false), "injanmaj");
        assert_eq!(form(&v("wenel"), Aspect::Perfect, 2, false), "wena'an");
        assert_eq!(form(&v("wenel"), Aspect::Perfect, 0, false), "wena'anen");
    }

    #[test]
    fn durative_and_assurative() {
        assert_eq!(form(&v("wenel"), Aspect::Durative, 2, false), "tan uwenel");
        assert_eq!(form(&v("ilik"), Aspect::Durative, 0, true), "tan kiwilik");
        assert_eq!(form(&v("wenel"), Aspect::Assurative, 0, false), "je' inwenelej");
    }

    #[test]
    fn labial_assimilation() {
        // im-b'el, im-b'et-ik (§3.1.1.2, §19.1.7)
        let c = itza();
        let r = c.conjugate("b'el").unwrap();
        let inc = r.tables.iter().find(|t| t.id == "incompletive").unwrap();
        assert_eq!(inc.rows[0].singular, "kimb'el");
        let com = r.tables.iter().find(|t| t.id == "completive").unwrap();
        assert_eq!(com.rows[2].singular, "b'inij");
    }

    #[test]
    fn negatives_and_spanish() {
        let r = itza().conjugate("wenel").unwrap();
        let t = |id: &str| r.tables.iter().find(|t| t.id == id).unwrap();
        assert_eq!(t("neg-incompletive").rows[0].singular, "ma' tan inwenel");
        assert_eq!(t("neg-completive").rows[2].singular, "ma' wenij");
        assert_eq!(t("incompletive").rows[0].es_singular.as_deref(), Some("yo duermo"));
        assert_eq!(t("completive").rows[2].es_plural.as_deref(), Some("ellos/ellas durmieron"));
        assert_eq!(t("neg-incompletive").rows[0].es_singular.as_deref(), Some("yo no duermo"));
    }

    #[test]
    fn your_own_verb_with_type_and_meaning() {
        let opts = |kind: &str, es: &str| Options { es: Some(es.into()), kind: Some(kind.into()) };
        let c = itza();
        // Bare roots work once the type is chosen.
        let r = c.conjugate_with("wen", &opts("I", "dormir")).unwrap();
        assert_eq!(r.tables[0].rows[0].singular, "kinwenel");
        assert_eq!(r.tables[0].rows[0].es_singular.as_deref(), Some("yo duermo"));
        let r = c.conjugate_with("nak'", &opts("I", "subir")).unwrap();
        assert_eq!(r.tables[0].rows[0].singular, "kinnak'äl");
        let r = c.conjugate_with("meyaj", &opts("Tt", "trabajar")).unwrap();
        assert_eq!(r.tables[0].rows[0].singular, "kimmeyajtik");
        let com = r.tables.iter().find(|t| t.id == "completive").unwrap();
        assert_eq!(com.rows[2].singular, "tumeyajtaj");
        assert_eq!(com.rows[2].es_singular.as_deref(), Some("él/ella trabajó"));
        // The dictionary form still works with a type.
        let r = c.conjugate_with("sutik", &opts("T", "voltear")).unwrap();
        assert_eq!(r.tables[0].rows[2].singular, "kusutik");
        assert_eq!(r.tables[0].rows[2].es_singular.as_deref(), Some("él/ella voltea"));
        // Meaning only: the class is still detected from the ending.
        let r = c.conjugate_with("tuchik", &Options { es: Some("señalar".into()), kind: None }).unwrap();
        assert_eq!(r.tables[0].rows[0].es_singular.as_deref(), Some("yo señalo"));
        assert!(!r.known);
    }

    #[test]
    fn unknown_class_is_an_error() {
        // A bare root that isn't in the lexicon needs its type.
        assert_eq!(itza().conjugate("jop'").unwrap_err(), Error::UnknownClass);
        // A bare root from the lexicon is found by its root.
        assert_eq!(itza().conjugate("wen").unwrap().tables[0].rows[0].singular, "kinwenel");
    }
}
