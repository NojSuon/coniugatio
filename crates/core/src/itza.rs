//! Conjugation for Itza' Maya (Itzaj).
//!
//! Source: Charles Andrew Hofling with Fernando Tesucún, *Itzaj Maya Grammar*
//! (University of Utah Press, 2000): §2.1 person markers; §3.1–3.2 aspect,
//! status and verb classes; §15 tense, aspect and mood (auxiliaries,
//! dependent status, imperative, admonitive, hortative, ka'aj future);
//! §19.1 negation. Long vowels are not written (Mati Mati spelling).
//!
//! A verb is entered the way Hofling's dictionary lists it, in the
//! incompletive status: `wenel` (intransitive, harmonic -Vl), `ilik` (root
//! transitive, -ik) or `jantik` (transitive derived with -t). Every
//! construction is built from the verb's stems:
//!
//! ```text
//!                 intransitive (wenel)   transitive (ilik)
//! incompletive    k-in-wen-el            k-in-w-il-ik
//! completive      wen-en                 t-in-w-il-aj
//! perfect         wen-a'an-en            in-w-il-m-aj
//! dependent       (ka') wen-ek-en        (ka') in-w-il-a'
//! imperative      wen-en!                il-a'!
//! auxiliaries     k'ab'et in-wen-el      k'ab'et in-w-il-ik
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

/// One construction (a table on the page).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Aspect {
    /// k- … : habitual present (§15.2.1)
    Incompletive,
    /// tan … : ongoing action (§15.2.2)
    Durative,
    /// je' … -ej : the speaker is sure it will happen (§15.2.3)
    Assurative,
    /// kil … -ej : "when" (§3.1.1.1)
    Punctual,
    /// tantoj … : just happened (§3.1.1.1)
    Immediate,
    /// <word> + Set A verb: k'ab'et, olak, patal, suk, tak, yan (§3.1.1.2.1)
    Auxiliary,
    /// <word>-ij + Set A verb: kajij, jop'ij, jo'mij, ts'o'kij (§3.1.1.2.2)
    Phasal,
    /// t- … -aj (transitive) / root + Set B (intransitive) (§3.1.2)
    Completive,
    /// … -m-aj (transitive) / -a'an + Set B (intransitive) (§3.3.1.1.1.2)
    Perfect,
    /// ka'aj ti … / ka'aj + dependent: "going to" (§15.4.2.1)
    Go,
    /// ka' + dependent status: "that …" (§3.1.3)
    Subjunctive,
    /// -en (intransitive) / -V', -ej (transitive), 2nd person (§15.4.1)
    Imperative,
    /// b'ikij + dependent: warning, "you could …" (§15.4.2.2)
    Admonitive,
    /// ko'ox: "let's …", 1st person plural (§15.4.2.3)
    Hortative,
    /// ko'on / ko'one'ex: "come and …", 2nd person (§15.4.2.3)
    Come,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tense {
    pub id: String,
    pub group: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub aspect: Aspect,
    /// The auxiliary for `auxiliary` and `phasal` aspects.
    #[serde(default)]
    pub word: Option<String>,
    #[serde(default)]
    pub negative: bool,
    #[serde(default)]
    pub es: Option<spanish::Es>,
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
    /// Irregular dependent stem: intransitive before Set B (b'el → xi'ik,
    /// tal → tak); transitive after Set A (il → ila').
    #[serde(default)]
    pub dependent: Option<String>,
    /// Irregular singular imperative (b'el → xen).
    #[serde(default)]
    pub imperative: Option<String>,
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
    root: String,
    /// Root used in the completive and perfect (usually the same).
    completive_root: String,
    /// Stem used after the Set A prefix in non-completive aspects:
    /// "wenel", "ilik", "jantik", "b'el".
    incompletive: String,
    /// Dependent stem: "wenek", "xi'ik" (intransitive, before Set B) or
    /// "uk'u'", "ila'", "jantej" (transitive, after Set A).
    dependent: String,
    /// Singular imperative: "wenen", "xen", "uk'u'", "jantej".
    imperative: String,
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

/// First vowel of the root, which harmonic suffixes repeat: wen-el, wen-ek,
/// uk'-u'.
fn harmonic_vowel(root: &str) -> char {
    root.chars().find(|c| is_vowel(*c)).unwrap_or('e')
}

impl Verb {
    fn new(class: Class, root: String, incompletive: String, entry: Option<&Entry>) -> Verb {
        let v = harmonic_vowel(&root);
        let completive_root = entry
            .and_then(|e| e.completive.clone())
            .unwrap_or_else(|| root.clone());
        let dependent = entry
            .and_then(|e| e.dependent.clone())
            .unwrap_or_else(|| match class {
                Class::I => format!("{root}{v}k"),
                Class::T => format!("{root}{v}'"),
                Class::Tt => format!("{root}tej"),
            });
        let imperative = entry
            .and_then(|e| e.imperative.clone())
            .unwrap_or_else(|| match class {
                Class::I => format!("{root}en"),
                _ => dependent.clone(),
            });
        Verb {
            class,
            root,
            completive_root,
            incompletive,
            dependent,
            imperative,
        }
    }
}

/// Builds a verb once its class is known, from a bare root (wen → wenel,
/// il → ilik, jan → jantik) or from its dictionary form.
fn from_class(word: &str, class: Class) -> std::result::Result<Verb, Error> {
    if !has_vowel(word) {
        return Err(Error::InvalidVerb);
    }
    match class {
        Class::I => {
            let parsed = parse_as(word, None, Class::I)?;
            if parsed.root != word {
                return Ok(parsed);
            }
            let incompletive = format!("{word}{}l", harmonic_vowel(word));
            Ok(Verb::new(class, word.into(), incompletive, None))
        }
        Class::T => {
            let root = word
                .strip_suffix("ik")
                .filter(|r| has_vowel(r))
                .unwrap_or(word);
            Ok(Verb::new(class, root.into(), format!("{root}ik"), None))
        }
        Class::Tt => {
            let root = word
                .strip_suffix("tik")
                .or_else(|| word.strip_suffix('t'))
                .filter(|r| has_vowel(r))
                .unwrap_or(word);
            Ok(Verb::new(class, root.into(), format!("{root}tik"), None))
        }
    }
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
                Some(v)
                    if is_vowel(v)
                        && w.len() > v.len_utf8()
                        && has_vowel(&w[..w.len() - v.len_utf8()]) =>
                {
                    w[..w.len() - v.len_utf8()].to_string()
                }
                _ => word.to_string(),
            }
        }
    };
    Ok(Verb::new(class, root, word.to_string(), entry))
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

/// Plural suffix that goes with a Set A subject: 2pl -e'ex, 3pl -o'.
/// The 1st person plural (exclusive) has none.
fn plural_suffix(person: usize, plural: bool) -> &'static str {
    match (person, plural) {
        (1, true) => "e'ex",
        (2, true) => "o'",
        _ => "",
    }
}

/// Set B (absolutive) suffixes; `third` is -ij in the completive, -Ø
/// elsewhere. (Hofling writes -een/-eech and -oo'; long vowels
/// are not written here.)
fn set_b(person: usize, plural: bool, third: &'static str) -> &'static str {
    match (person, plural) {
        (0, false) => "en",
        (1, false) => "ech",
        (2, false) => third,
        (0, true) => "o'on",
        (1, true) => "e'ex",
        _ => "o'",
    }
}

/// Builds one form, or `None` when the construction has no such person
/// (imperatives are 2nd person, ko'ox is 1st person plural).
fn form(verb: &Verb, tense: &Tense, person: usize, plural: bool) -> Option<String> {
    let transitive = verb.class != Class::I;
    let t = if verb.class == Class::Tt { "t" } else { "" };
    let word = tense.word.as_deref().unwrap_or("");
    // Set A + incompletive stem + plural: inwenel, kuyiliko'.
    let a_verb = || {
        format!(
            "{}{}",
            set_a(person, plural, &verb.incompletive),
            plural_suffix(person, plural)
        )
    };
    // Dependent status: wenek-en (Set B) / inw-il-a' (Set A).
    let dependent = || {
        if transitive {
            format!(
                "{}{}",
                set_a(person, plural, &verb.dependent),
                plural_suffix(person, plural)
            )
        } else {
            format!("{}{}", verb.dependent, set_b(person, plural, ""))
        }
    };
    let second = person == 1;

    if tense.negative {
        return match tense.aspect {
            // The negative present uses the durative (§19.1.5).
            Aspect::Incompletive => Some(format!("ma' tan {}", a_verb())),
            // Negative imperative: ma' + incompletive stem (§19.1.3).
            Aspect::Imperative => second.then(|| format!("ma' {}", a_verb())),
            // Unrealized perfect: ma' + dependent status (§19.1.6).
            Aspect::Perfect => Some(format!("ma' {}", dependent())),
            _ => {
                let plain = Tense {
                    negative: false,
                    ..tense.clone()
                };
                form(verb, &plain, person, plural).map(|f| format!("ma' {f}"))
            }
        };
    }

    Some(match tense.aspect {
        Aspect::Incompletive => {
            // k- merges with the 1st person plural ki-.
            let s = a_verb();
            if person == 0 && plural {
                s
            } else {
                format!("k{s}")
            }
        }
        Aspect::Durative => format!("tan {}", a_verb()),
        Aspect::Assurative => format!("je' {}ej", a_verb()),
        Aspect::Punctual => format!("kil {}ej", a_verb()),
        Aspect::Immediate => format!("tantoj {}", a_verb()),
        Aspect::Auxiliary => format!("{word} {}", a_verb()),
        Aspect::Phasal => format!("{word}ij {}", a_verb()),
        Aspect::Completive if transitive => {
            let s = format!(
                "{}{}",
                set_a(person, plural, &format!("{}{t}aj", verb.completive_root)),
                plural_suffix(person, plural)
            );
            // t- has the form ti- before ki-.
            if person == 0 && plural {
                format!("ti{s}")
            } else {
                format!("t{s}")
            }
        }
        Aspect::Completive => format!("{}{}", verb.completive_root, set_b(person, plural, "ij")),
        // In the perfect, -t- is replaced by -m: jan-t-ik → in-jan-m-aj.
        Aspect::Perfect if transitive => format!(
            "{}{}",
            set_a(person, plural, &format!("{}maj", verb.completive_root)),
            plural_suffix(person, plural)
        ),
        Aspect::Perfect => format!("{}a'an{}", verb.completive_root, set_b(person, plural, "")),
        // Intransitives: (b'el) inka'aj ti wenel; transitives take the
        // dependent status: inka'aj inwila'.
        Aspect::Go if transitive => format!("{} {}", set_a(person, plural, "ka'aj"), dependent()),
        Aspect::Go => format!(
            "{}{} ti {}",
            set_a(person, plural, "ka'aj"),
            plural_suffix(person, plural),
            verb.incompletive
        ),
        Aspect::Subjunctive => format!("ka' {}", dependent()),
        // b'ikij contracts to b'ik before ki- (§15.4.2.2, note 7).
        Aspect::Admonitive if transitive && person == 0 && plural => {
            format!("b'ik {}", dependent())
        }
        Aspect::Admonitive => format!("b'ikij {}", dependent()),
        Aspect::Imperative if !second => return None,
        Aspect::Imperative => {
            if plural {
                format!("{}e'ex!", verb.imperative)
            } else {
                format!("{}!", verb.imperative)
            }
        }
        Aspect::Hortative if !(person == 0 && plural) => return None,
        Aspect::Hortative if transitive => format!("ko'ox {}", dependent()),
        Aspect::Hortative => format!("ko'ox ti {}", verb.incompletive),
        Aspect::Come if !second => return None,
        Aspect::Come => {
            let ko = if plural { "ko'one'ex" } else { "ko'on" };
            if transitive {
                format!("{ko} {}", dependent())
            } else {
                format!("{ko} ti {}", verb.incompletive)
            }
        }
    })
}

/// Lower-cases and writes every glottal stop as a straight apostrophe.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c == 'ʼ' || c == '’' || c == '‘' {
                '\''
            } else {
                c
            }
        })
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

    pub fn conjugate_with(
        &self,
        input: &str,
        opts: &Options,
    ) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let forced = opts.kind().and_then(Class::from_kind);
        // Look the verb up by its dictionary form, or by its bare root
        // (nak' → nak'äl), so lexicon exceptions win over the rules.
        let entry = self.lexicon.iter().find(|e| {
            forced.is_none_or(|c| c == e.class)
                && (e.entry == word
                    || parse_as(&e.entry, Some(e), e.class).is_ok_and(|v| v.root == word))
        });
        let verb = match (entry, forced) {
            (Some(e), _) => parse(&e.entry, Some(e))?,
            (None, Some(class)) => from_class(&word, class)?,
            (None, None) => parse(&word, None)?,
        };
        // A meaning typed by the user wins over the lexicon's.
        let es_meaning = opts
            .es()
            .map(str::to_string)
            .or_else(|| entry.and_then(|e| e.es.clone()));

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
                    .filter_map(|(i, person)| {
                        let sg = form(&verb, tense, i, false);
                        let pl = form(&verb, tense, i, true);
                        if sg.is_none() && pl.is_none() {
                            return None;
                        }
                        Some(Row {
                            person: person.label.clone(),
                            clitic: String::new(),
                            es_singular: sg.as_ref().and_then(|_| es(tense, person, false)),
                            es_plural: pl.as_ref().and_then(|_| es(tense, person, true)),
                            singular: sg.unwrap_or_default(),
                            plural: pl,
                        })
                    })
                    .collect(),
            })
            .collect();

        Ok(Conjugated {
            root: word,
            known: entry.is_some(),
            meaning: entry
                .map(|e| e.meaning.clone())
                .or_else(|| opts.es().map(str::to_string)),
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

    fn tense(aspect: Aspect) -> Tense {
        Tense {
            id: String::new(),
            group: String::new(),
            name: String::new(),
            description: String::new(),
            aspect,
            word: None,
            negative: false,
            es: None,
        }
    }

    fn f(word: &str, aspect: Aspect, person: usize, plural: bool) -> String {
        form(&v(word), &tense(aspect), person, plural).unwrap()
    }

    fn all(word: &str, aspect: Aspect) -> Vec<String> {
        (0..3)
            .flat_map(|p| [f(word, aspect, p, false), f(word, aspect, p, true)])
            .collect()
    }

    /// Singular and plural of one table for one person, from the real data.
    fn row(verb: &str, id: &str, person: usize) -> (String, Option<String>, Option<String>) {
        let r = itza().conjugate(verb).unwrap();
        let t = r
            .tables
            .iter()
            .find(|t| t.id == id)
            .unwrap_or_else(|| panic!("no table {id}"));
        let row = t
            .rows
            .iter()
            .find(|r| r.person.starts_with(&format!("{}.", person + 1)))
            .unwrap();
        (
            row.singular.clone(),
            row.plural.clone(),
            row.es_singular.clone(),
        )
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
        assert_eq!(
            all("ilik", Aspect::Incompletive),
            [
                "kinwilik",
                "kiwilik",
                "kawilik",
                "kawilike'ex",
                "kuyilik",
                "kuyiliko'"
            ]
        );
        assert_eq!(
            all("talel", Aspect::Incompletive),
            [
                "kintalel",
                "kitalel",
                "katalel",
                "katalele'ex",
                "kutalel",
                "kutalelo'"
            ]
        );
    }

    // §2.1.1 (3), §3.2.2.2 (long -een written short)
    #[test]
    fn completive() {
        assert_eq!(
            all("ilik", Aspect::Completive),
            [
                "tinwilaj",
                "tikiwilaj",
                "tawilaj",
                "tawilaje'ex",
                "tuyilaj",
                "tuyilajo'"
            ]
        );
        assert_eq!(
            all("k'ochol", Aspect::Completive),
            [
                "k'ochen",
                "k'ocho'on",
                "k'ochech",
                "k'oche'ex",
                "k'ochij",
                "k'ocho'"
            ]
        );
        assert_eq!(f("jantik", Aspect::Completive, 0, false), "tinjantaj");
    }

    #[test]
    fn perfect() {
        assert_eq!(f("ilik", Aspect::Perfect, 0, false), "inwilmaj");
        assert_eq!(f("jantik", Aspect::Perfect, 0, false), "injanmaj");
        assert_eq!(f("wenel", Aspect::Perfect, 2, false), "wena'an");
        assert_eq!(f("wenel", Aspect::Perfect, 0, false), "wena'anen");
    }

    #[test]
    fn uninflected_aspects() {
        assert_eq!(f("wenel", Aspect::Durative, 2, false), "tan uwenel");
        assert_eq!(f("ilik", Aspect::Durative, 0, true), "tan kiwilik");
        assert_eq!(f("wenel", Aspect::Assurative, 0, false), "je' inwenelej");
        // kil u-wen-el-ej, tantoj u-wen-el (§3.1.1.1)
        assert_eq!(f("wenel", Aspect::Punctual, 2, false), "kil uwenelej");
        assert_eq!(f("wenel", Aspect::Immediate, 2, false), "tantoj uwenel");
    }

    // §3.1.1.2: k'ab'eet im-b'et-ik, pat-al u-wen-el, kaj-ij im-meyaj-t-ik
    #[test]
    fn auxiliaries() {
        assert_eq!(row("wenel", "kabet", 0).0, "k'ab'et inwenel");
        assert_eq!(row("wenel", "patal", 2).0, "patal uwenel");
        assert_eq!(
            row("wenel", "patal", 2).2.as_deref(),
            Some("él/ella puede dormir")
        );
        assert_eq!(row("wenel", "yan", 2).1.as_deref(), Some("yan uwenelo'"));
        assert_eq!(row("wenel", "olak", 0).2.as_deref(), Some("yo casi dormí"));
        assert_eq!(row("wenel", "kaj", 0).0, "kajij inwenel");
        assert_eq!(
            row("wenel", "kaj", 0).2.as_deref(),
            Some("yo empecé a dormir")
        );
        assert_eq!(row("wenel", "tsok", 2).0, "ts'o'kij uwenel");
    }

    // §15.4.2.1: (b'el) u-ka'aj ti wen-el; u-ka'aj u-jan-t-ej
    #[test]
    fn go_future() {
        assert_eq!(row("wenel", "go", 2).0, "uka'aj ti wenel");
        assert_eq!(
            row("wenel", "go", 2).1.as_deref(),
            Some("uka'ajo' ti wenel")
        );
        assert_eq!(row("wenel", "go", 0).2.as_deref(), Some("yo voy a dormir"));
        assert_eq!(row("jantik", "go", 2).0, "uka'aj ujantej");
        assert_eq!(row("ilik", "go", 0).0, "inka'aj inwila'");
    }

    // §3.1.3, §15.4.2: ka' wen-ek-ech, ka' uy-uk'-u', ka' u-jan-t-ej
    #[test]
    fn subjunctive() {
        assert_eq!(row("wenel", "subjunctive", 1).0, "ka' wenekech");
        assert_eq!(row("wenel", "subjunctive", 2).0, "ka' wenek");
        assert_eq!(row("k'ochol", "subjunctive", 0).0, "ka' k'ochoken");
        assert_eq!(row("uk'ik", "subjunctive", 2).0, "ka' uyuk'u'");
        assert_eq!(row("ilik", "subjunctive", 0).0, "ka' inwila'");
        assert_eq!(row("jantik", "subjunctive", 2).0, "ka' ujantej");
        assert_eq!(
            row("wenel", "subjunctive", 0).2.as_deref(),
            Some("que yo duerma")
        );
        // b'el is irregular: xi'ik (§3.2.2.1, note 10)
        assert_eq!(row("b'el", "subjunctive", 1).0, "ka' xi'ikech");
    }

    // §15.4.1: wen-en!, il-a'!, jan-t-ej!, xen!; ma' a-wen-el!
    #[test]
    fn imperative() {
        let (sg, pl, es) = row("wenel", "imperative", 1);
        assert_eq!(
            (sg.as_str(), pl.as_deref(), es.as_deref()),
            ("wenen!", Some("wenene'ex!"), Some("¡duerme!"))
        );
        assert_eq!(row("ilik", "imperative", 1).0, "ila'!");
        assert_eq!(row("jantik", "imperative", 1).0, "jantej!");
        assert_eq!(row("b'el", "imperative", 1).0, "xen!");
        assert_eq!(row("wenel", "neg-imperative", 1).0, "ma' awenel");
        assert_eq!(
            row("wenel", "neg-imperative", 1).2.as_deref(),
            Some("¡no duermas!")
        );
        // Only the 2nd person has an imperative.
        let r = itza().conjugate("wenel").unwrap();
        let t = r.tables.iter().find(|t| t.id == "imperative").unwrap();
        assert_eq!(t.rows.len(), 1);
    }

    // §15.4.2.2: B'ikij lub'-uk-ech!; b'iik ki-kim-es
    #[test]
    fn admonitive() {
        assert_eq!(row("lub'ul", "admonitive", 1).0, "b'ikij lub'ukech");
        assert_eq!(
            row("ilik", "admonitive", 0).1.as_deref(),
            Some("b'ik kiwila'")
        );
    }

    // §15.4.2.3: Ko'ox ti p'at-äl; ko'ox kiw-il-a'; ko'on-e'ex ti jan-al
    #[test]
    fn hortative_and_come() {
        let (sg, pl, _) = row("wenel", "hortative", 0);
        assert_eq!((sg.as_str(), pl.as_deref()), ("", Some("ko'ox ti wenel")));
        assert_eq!(
            row("ilik", "hortative", 0).1.as_deref(),
            Some("ko'ox kiwila'")
        );
        assert_eq!(row("wenel", "come", 1).0, "ko'on ti wenel");
        assert_eq!(
            row("wenel", "come", 1).1.as_deref(),
            Some("ko'one'ex ti wenel")
        );
        assert_eq!(row("wenel", "come", 1).2.as_deref(), Some("¡ven a dormir!"));
    }

    #[test]
    fn negatives() {
        assert_eq!(row("wenel", "neg-incompletive", 0).0, "ma' tan inwenel");
        assert_eq!(
            row("wenel", "neg-incompletive", 0).2.as_deref(),
            Some("yo no duermo")
        );
        assert_eq!(row("wenel", "neg-completive", 2).0, "ma' wenij");
        // Unrealized perfect: Ma' jan-ak-en "I haven't eaten" (§19.1.6)
        assert_eq!(row("wenel", "neg-perfect", 0).0, "ma' weneken");
        assert_eq!(
            row("wenel", "neg-perfect", 0).2.as_deref(),
            Some("yo no he dormido")
        );
    }

    #[test]
    fn labial_assimilation_and_irregular_b_el() {
        assert_eq!(row("b'el", "incompletive", 0).0, "kimb'el");
        assert_eq!(row("b'el", "completive", 2).0, "b'inij");
    }

    #[test]
    fn your_own_verb_with_type_and_meaning() {
        let opts = |kind: &str, es: &str| Options {
            es: Some(es.into()),
            kind: Some(kind.into()),
            ..Default::default()
        };
        let c = itza();
        // Bare roots work once the type is chosen.
        let r = c.conjugate_with("wen", &opts("I", "dormir")).unwrap();
        assert_eq!(r.tables[0].rows[0].singular, "kinwenel");
        assert_eq!(
            r.tables[0].rows[0].es_singular.as_deref(),
            Some("yo duermo")
        );
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
        assert_eq!(
            r.tables[0].rows[2].es_singular.as_deref(),
            Some("él/ella voltea")
        );
        // Meaning only: the class is still detected from the ending.
        let r = c
            .conjugate_with(
                "tuchik",
                &Options {
                    es: Some("señalar".into()),
                    kind: None,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            r.tables[0].rows[0].es_singular.as_deref(),
            Some("yo señalo")
        );
        assert!(!r.known);
    }

    #[test]
    fn unknown_class_is_an_error() {
        // A bare root that isn't in the lexicon needs its type.
        assert_eq!(itza().conjugate("jop'").unwrap_err(), Error::UnknownClass);
        // A bare root from the lexicon is found by its root.
        assert_eq!(
            itza().conjugate("wen").unwrap().tables[0].rows[0].singular,
            "kinwenel"
        );
    }
}
