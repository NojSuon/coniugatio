//! Conjugation for Zoque (otetsame / tsunitsame) of Copainalá, Chiapas
//! (zoque del centro), written with the INALI *Norma de escritura de la
//! lengua otetzame* (2014), with ts for the affricate as the community
//! prefers: a e i o u ä; j k l m n nh p r s t ts w y '.
//!
//! The morphology follows Engel & Engel, *Diccionario zoque de Copainalá*
//! (ILV, 1981), Gramática §7–20:
//!
//! ```text
//! prefix   root   plural      tense   person (enclitic)
//! Ny-      poy    -tam-       -pa     -mij     mpyoytyampamij «ustedes corren»
//! —        poy                -u      -'äj     poyu'äj        «corrí»
//! y-       tsäjk  -yaj-       -u               tsyäkyaju      «lo hicieron»
//! ```
//!
//! - Person: enclitics after the verb (-'äj / -'äs «yo», -mij / -mis «tú,
//!   ustedes», -täj / -täs «nosotros» excl., -tä incl.), prefixes N- (1st
//!   and 2nd of transitive verbs), Ny- (2nd of intransitive verbs) and y-
//!   (3rd of transitive verbs), which slips in after the first consonant:
//!   tsäjk → tsyäjku.
//! - Plural: -tam- for «ustedes» and «nosotros» inclusive, -yaj- for «ellos».
//! - Tense: -pa not finished, -u finished; auxiliaries nä (ongoing), manhpa
//!   (future), sunpa (want); inäk after the verb for the past.
//! - Negation: ja'n + the verb with -I (-i / -e); in the past, jay- + -A.
//! - Orders: the root + -A (-ä / -a); «no» uy + -I.

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    /// Intransitive: poy- «correr», pejt- «barrer».
    I,
    /// Transitive, with a 3rd person object: tsäjk- «hacerlo».
    T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// -pa: popya'äj «corro».
    Present,
    /// -pa … inäk: popya'äj inäk «corría».
    Imperfect,
    /// -u: poyu'äj «corrí».
    Past,
    /// nä + verb: nä'äj mpoyu «estoy corriendo».
    Progressive,
    /// nä … inäk + verb: nä'äj inäk mpoyu «estaba corriendo».
    PastProgressive,
    /// manhpa + verb: manhpa'äj mpoyu «voy a correr».
    Future,
    /// minpa + verb: «vengo a…».
    Come,
    /// sunpa + verb: «quiero…».
    Want,
    /// muspa + verb: «puedo…».
    Can,
    /// nyempe + verb with -A: «aunque corra».
    Subjunctive,
    /// ja'n + verb with -I: ja'näj mpoye «no corro».
    NegPresent,
    /// ja'n + manhu + verb: «no voy a…».
    NegFuture,
    /// jay- + verb with -A: japyoya'äj «no corrí».
    NegPast,
    /// root + -A: poya «¡corre!»; «we» poytya'mitäy «¡corramos!».
    Imperative,
    /// uy + root + -I: uy poye «¡no corras!».
    NegImperative,
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
    pub es: Option<spanish::Es>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Citation form, the 3rd person past: "poyu", "tsyäjku".
    pub entry: String,
    /// Root: "poy", "tsäjk".
    pub root: String,
    pub class: Class,
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

// ---------- sounds ----------

fn is_vowel(c: char) -> bool {
    "aeiouä".contains(c)
}

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

/// The first consonant of a word (ts counts as one) and the rest.
fn split_first(word: &str) -> Option<(String, String)> {
    let c = chars(word);
    let first = *c.first()?;
    if is_vowel(first) || first == '\'' {
        return None;
    }
    let n = if first == 't' && c.get(1) == Some(&'s') {
        2
    } else {
        1
    };
    Some((c[..n].iter().collect(), c[n..].iter().collect()))
}

/// y- (3rd person, and jay-): slips in after the first consonant,
/// tsäjk → tsyäjk, poy → pyoy; before a vowel it is 'y: änh → 'yänh.
fn y_prefix(word: &str) -> String {
    match split_first(word) {
        None => format!("'y{}", word.trim_start_matches('\'')),
        Some((_, rest)) if rest.starts_with('y') => word.to_string(),
        Some((c, rest)) => format!("{c}y{rest}"),
    }
}

/// N- (1st person): assimilates to the consonant (mp, nt, nts, nhk), is
/// lost before a vowel, s, m, n or l, and is n before w, y, j (§20.5.1).
fn n_prefix(word: &str) -> String {
    match word.chars().next() {
        Some('p') => format!("m{word}"),
        Some('k') => format!("nh{word}"),
        Some('t' | 'w' | 'y' | 'j') => format!("n{word}"),
        _ => word.to_string(),
    }
}

/// Ny- (2nd person): N- + y-: poy → mpyoy, ken → nhkyen.
fn ny_prefix(word: &str) -> String {
    if split_first(word).is_none() {
        return format!("ny{}", word.trim_start_matches('\''));
    }
    n_prefix(&y_prefix(word))
}

/// Joins a root (or stem) and a suffix with the sound changes of §20.1–20.4.
fn join(stem: &str, suffix: &str) -> String {
    let s = chars(stem);
    let Some(&first) = suffix.chars().next().as_ref() else {
        return stem.to_string();
    };
    if is_vowel(first) || first == '\'' {
        return format!("{stem}{suffix}");
    }
    let n = s.len();
    let last = s[n - 1];
    // A root in -y: before t, ts, s, n the y stays and palatalizes them
    // (poy-tam → poytyam); y-yaj → yaj; otherwise the y moves after the
    // consonant (poy-pa → popya).
    if last == 'y' {
        let base: String = s[..n - 1].iter().collect();
        let (c, rest) = split_first(suffix).unwrap_or((suffix.to_string(), String::new()));
        return match c.as_str() {
            "t" | "ts" | "s" | "n" => format!("{stem}{c}y{rest}"),
            "y" => format!("{base}{suffix}"),
            _ => format!("{base}{c}y{rest}"),
        };
    }
    // A root in j + consonant: the j is lost before a different consonant
    // (tsäjk-pa → tsäkpa) and kept before the same one (kijp-pa → kijpa),
    // but jt + t → tt (pejt-tam → pettam). Alveolars keep it before -yaj-
    // (pejtyaju).
    if n >= 2 && s[n - 2] == 'j' && !is_vowel(last) {
        let c: String = suffix.chars().take(1).collect();
        let without_j: String = s[..n - 2].iter().chain(std::iter::once(&last)).collect();
        if first == 'y' {
            return if "tsn".contains(last) {
                format!("{stem}{suffix}")
            } else {
                format!("{without_j}{suffix}")
            };
        }
        if c == last.to_string() {
            return if last == 't' {
                format!("{without_j}{suffix}")
            } else {
                format!("{stem}{}", &suffix[c.len()..])
            };
        }
        return format!("{without_j}{suffix}");
    }
    format!("{stem}{suffix}")
}

/// The harmonic suffixes -A (orders, past negative, subjunctive) and -I
/// (present negative), §20.5.4–20.5.6: -A is a after e, o and ä otherwise;
/// -I is e after e, a, o and i otherwise. A final jC loses its j, nh
/// becomes w, and a final j becomes y (w after u).
fn harmonic(stem: &str, a: bool) -> String {
    // 3rd person plural: -yaj- + -I → -yay; -yaj- + -A → -ya.
    if let Some(base) = stem.strip_suffix("yaj") {
        return if a {
            format!("{base}ya")
        } else {
            format!("{base}yay")
        };
    }
    let mut s = chars(stem);
    let v = s
        .iter()
        .rev()
        .copied()
        .find(|&c| is_vowel(c))
        .unwrap_or('a');
    let n = s.len();
    if n >= 2 && s[n - 2] == 'j' && !is_vowel(s[n - 1]) {
        s.remove(n - 2);
    } else if stem.ends_with("nh") {
        s.truncate(n - 2);
        s.push('w');
    } else if s[n - 1] == 'j' {
        s[n - 1] = if v == 'u' { 'w' } else { 'y' };
    }
    let suffix = match (a, v) {
        (true, 'e' | 'o') => 'a',
        (true, _) => 'ä',
        (false, 'i' | 'ä' | 'u') => 'i',
        (false, _) => 'e',
    };
    let mut out: String = s.into_iter().collect();
    out.push(suffix);
    out
}

/// An enclitic after the verb: the glottal stop of -'äj / -'äs only
/// appears after a vowel (poyu'äj, but ja'näj).
fn enclitic(word: &str, e: &str) -> String {
    match e.strip_prefix('\'') {
        Some(rest) if !word.ends_with(is_vowel) => format!("{word}{rest}"),
        _ => format!("{word}{e}"),
    }
}

// ---------- persons ----------

/// The seven persons: yo, tú, él/ella, nosotros (excl.), nosotros (incl.),
/// ustedes, ellos.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    S1,
    S2,
    S3,
    P1x,
    P1i,
    P2,
    P3,
}

impl Slot {
    /// Rows: 1st, 2nd, 3rd person (singular / plural) and the inclusive
    /// «we» (plural only).
    fn of(person: usize, plural: bool) -> Option<Slot> {
        Some(match (person, plural) {
            (0, false) => Slot::S1,
            (1, false) => Slot::S2,
            (2, false) => Slot::S3,
            (0, true) => Slot::P1x,
            (1, true) => Slot::P2,
            (2, true) => Slot::P3,
            (3, true) => Slot::P1i,
            _ => return None,
        })
    }

    fn second(self) -> bool {
        matches!(self, Slot::S2 | Slot::P2)
    }

    fn third(self) -> bool {
        matches!(self, Slot::S3 | Slot::P3)
    }

    /// The person enclitic: intransitive -'äj, -mij, -täj, -tä; transitive
    /// -'äs, -mis, -täs, -tä.
    fn enclitic(self, class: Class) -> &'static str {
        let t = class == Class::T;
        match self {
            Slot::S1 => {
                if t {
                    "'äs"
                } else {
                    "'äj"
                }
            }
            Slot::S2 | Slot::P2 => {
                if t {
                    "mis"
                } else {
                    "mij"
                }
            }
            Slot::P1x => {
                if t {
                    "täs"
                } else {
                    "täj"
                }
            }
            Slot::P1i => "tä",
            Slot::S3 | Slot::P3 => "",
        }
    }

    fn plural_suffix(self) -> Option<&'static str> {
        match self {
            Slot::P1i | Slot::P2 => Some("tam"),
            Slot::P3 => Some("yaj"),
            _ => None,
        }
    }
}

struct Verb {
    root: String,
    class: Class,
}

impl Verb {
    /// Root + plural: poytyam, pejtyaj.
    fn stem(&self, slot: Slot) -> String {
        match slot.plural_suffix() {
            Some(p) => join(&self.root, p),
            None => self.root.clone(),
        }
    }

    /// Prefix of an independent verb: Ny- for «tú/ustedes» of intransitive
    /// verbs; N- for the 1st and 2nd persons and y- for the 3rd of
    /// transitive verbs (cuadros 6 y 7).
    fn independent(&self, slot: Slot, word: &str) -> String {
        match (self.class, slot) {
            (Class::I, s) if s.second() => ny_prefix(word),
            (Class::I, _) => word.to_string(),
            (Class::T, s) if s.third() => y_prefix(word),
            (Class::T, _) => n_prefix(word),
        }
    }

    /// Prefix of the main verb after an auxiliary (cuadros 8 y 9).
    fn main(&self, slot: Slot, word: &str) -> String {
        match (self.class, slot) {
            (_, s) if s.third() => y_prefix(word),
            (Class::I, s) if s.second() => ny_prefix(word),
            _ => n_prefix(word),
        }
    }
}

fn build(verb: &Verb, form: Form, slot: Slot) -> Option<String> {
    let encl = slot.enclitic(verb.class);
    let stem = verb.stem(slot);
    let aux = |a: &str, ending: &str| {
        let main = verb.main(slot, &format!("{stem}{ending}"));
        format!("{} {main}", enclitic(a, encl))
    };
    Some(match form {
        Form::Present | Form::Imperfect => {
            let w = enclitic(&verb.independent(slot, &join(&stem, "pa")), encl);
            if form == Form::Imperfect {
                format!("{w} inäk")
            } else {
                w
            }
        }
        Form::Past => enclitic(&verb.independent(slot, &format!("{stem}u")), encl),
        Form::Progressive => aux("nä", "u"),
        Form::PastProgressive => {
            let main = verb.main(slot, &format!("{stem}u"));
            format!("{} inäk {main}", enclitic("nä", encl))
        }
        Form::Future => aux("manhpa", "u"),
        Form::Come => aux("minpa", "u"),
        Form::Want => aux("sunpa", "u"),
        Form::Can => aux("muspa", "u"),
        Form::Subjunctive => {
            let main = verb.main(slot, &harmonic(&stem, true));
            format!("{} {main}", enclitic("nyempe", encl))
        }
        Form::NegPresent => {
            let main = verb.main(slot, &harmonic(&stem, false));
            format!("{} {main}", enclitic("ja'n", encl))
        }
        // ja'näs manhu ntsäjku «no lo voy a hacer»: the pronoun goes on ja'n.
        Form::NegFuture => {
            let main = verb.main(slot, &format!("{stem}u"));
            format!("{} manhu {main}", enclitic("ja'n", encl))
        }
        // jay- + (N-/Ny-) + the root with y slipped in + -A: japyoya'äj,
        // jampyoyamij, jantsyäkä'äs.
        Form::NegPast => {
            let body = y_prefix(&harmonic(&stem, true));
            let nasal = match verb.class {
                Class::I => slot.second(),
                Class::T => !slot.third(),
            };
            let body = if nasal { n_prefix(&body) } else { body };
            enclitic(&format!("ja{body}"), encl)
        }
        Form::Imperative => match slot {
            Slot::S2 => harmonic(&verb.root, true),
            Slot::P2 => harmonic(&join(&verb.root, "tam"), true),
            // «¡corramos!»: -tam- + -'i- + tä(y): poytya'mitäy.
            Slot::P1i => {
                let s = join(&verb.root, "tam");
                format!("{}'mitäy", s.strip_suffix('m').unwrap_or(&s))
            }
            _ => return None,
        },
        Form::NegImperative => match slot {
            Slot::S2 => format!("uy {}", harmonic(&verb.root, false)),
            Slot::P2 => format!("uy {}", harmonic(&join(&verb.root, "tam"), false)),
            _ => return None,
        },
    })
}

// ---------- input ----------

fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb
        .trim()
        .trim_end_matches('-')
        .to_lowercase()
        .chars()
        .map(|c| match c {
            '’' | '‘' | 'ʼ' | '´' | '`' => '\'',
            c => c,
        })
        .collect::<String>()
        .replace("tz", "ts");
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
        let forced = match opts.kind() {
            Some("T") => Some(Class::T),
            Some("I") => Some(Class::I),
            _ => None,
        };
        // The citation form (poyu, tsyäjku) or the root (poy, tsäjk).
        let entry = self
            .lexicon
            .iter()
            .find(|e| (e.entry == word || e.root == word) && forced.is_none_or(|c| c == e.class));
        let verb = match entry {
            Some(e) => Verb {
                root: e.root.clone(),
                class: e.class,
            },
            None => {
                // A typed past form: drop -u (and the y- of a transitive).
                let class = forced.unwrap_or(Class::I);
                let root = word
                    .strip_suffix('u')
                    .filter(|r| r.chars().count() > 1)
                    .unwrap_or(&word);
                Verb {
                    root: root.to_string(),
                    class,
                }
            }
        };

        let es_meaning = opts
            .es()
            .map(str::to_string)
            .or_else(|| entry.and_then(|e| e.es.clone()));
        let es = |tense: &Tense, person: &Person, plural: bool| -> Option<String> {
            let meaning = es_meaning.as_deref()?;
            let p = person.es? + if plural { 3 } else { 0 };
            crate::gloss(opts, meaning, tense.es.as_ref()?, p, false)
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
                        let sg = Slot::of(i, false).and_then(|s| build(&verb, tense.form, s));
                        let pl = Slot::of(i, true).and_then(|s| build(&verb, tense.form, s));
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
            root: entry.map(|e| e.entry.clone()).unwrap_or(word),
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

    fn verb(root: &str, class: Class) -> Verb {
        Verb {
            root: root.into(),
            class,
        }
    }

    /// The seven forms in the dictionary's order: yo, tú, él, nosotros
    /// (excl.), nosotros (incl.), ustedes, ellos.
    fn seven(v: &Verb, form: Form) -> Vec<String> {
        [
            Slot::S1,
            Slot::S2,
            Slot::S3,
            Slot::P1x,
            Slot::P1i,
            Slot::P2,
            Slot::P3,
        ]
        .iter()
        .map(|&s| build(v, form, s).unwrap_or_default())
        .collect()
    }

    // Gramática §17, in INALI spelling (mb → mp, nd → nt, ŋg → nhk, ch → tsy,
    // ʌ → ä): the paradigms of poy- «correr» and tsäjk- «hacer».
    #[test]
    fn intransitive_poy() {
        let v = verb("poy", Class::I);
        assert_eq!(
            seven(&v, Form::Present),
            [
                "popya'äj",
                "mpyopyamij",
                "popya",
                "popyatäj",
                "poytyampatä",
                "mpyoytyampamij",
                "poyajpa"
            ]
        );
        assert_eq!(
            seven(&v, Form::Past),
            [
                "poyu'äj",
                "mpyoyumij",
                "poyu",
                "poyutäj",
                "poytyamutä",
                "mpyoytyamumij",
                "poyaju"
            ]
        );
        assert_eq!(
            seven(&v, Form::Progressive),
            [
                "nä'äj mpoyu",
                "nämij mpyoyu",
                "nä pyoyu",
                "nätäj mpoyu",
                "nätä mpoytyamu",
                "nämij mpyoytyamu",
                "nä pyoyaju"
            ]
        );
        assert_eq!(seven(&v, Form::Imperfect)[0], "popya'äj inäk");
        assert_eq!(seven(&v, Form::Future)[0], "manhpa'äj mpoyu");
        assert_eq!(
            seven(&v, Form::NegPresent),
            [
                "ja'näj mpoye",
                "ja'nmij mpyoye",
                "ja'n pyoye",
                "ja'ntäj mpoye",
                "ja'ntä mpoytyame",
                "ja'nmij mpyoytyame",
                "ja'n pyoyay"
            ]
        );
        assert_eq!(seven(&v, Form::NegFuture)[2], "ja'n manhu pyoyu");
        assert_eq!(seven(&v, Form::NegFuture)[0], "ja'näj manhu mpoyu");
        assert_eq!(
            seven(&v, Form::NegPast),
            [
                "japyoya'äj",
                "jampyoyamij",
                "japyoya",
                "japyoyatäj",
                "japyoytyamätä",
                "jampyoytyamämij",
                "japyoya"
            ]
        );
        assert_eq!(
            seven(&v, Form::Subjunctive),
            [
                "nyempe'äj mpoya",
                "nyempemij mpyoya",
                "nyempe pyoya",
                "nyempetäj mpoya",
                "nyempetä mpoytyamä",
                "nyempemij mpyoytyamä",
                "nyempe pyoya"
            ]
        );
        let imp = seven(&v, Form::Imperative);
        assert_eq!(
            (imp[1].as_str(), imp[5].as_str(), imp[4].as_str()),
            ("poya", "poytyamä", "poytya'mitäy")
        );
        assert_eq!(seven(&v, Form::NegImperative)[1], "uy poye");
        assert_eq!(seven(&v, Form::NegImperative)[5], "uy poytyame");
    }

    #[test]
    fn transitive_tsajk() {
        let v = verb("tsäjk", Class::T);
        assert_eq!(
            seven(&v, Form::Present),
            [
                "ntsäkpa'äs",
                "ntsäkpamis",
                "tsyäkpa",
                "ntsäkpatäs",
                "ntsäktampatä",
                "ntsäktampamis",
                "tsyäkyajpa"
            ]
        );
        assert_eq!(
            seven(&v, Form::Past),
            [
                "ntsäjku'äs",
                "ntsäjkumis",
                "tsyäjku",
                "ntsäjkutäs",
                "ntsäktamutä",
                "ntsäktamumis",
                "tsyäkyaju"
            ]
        );
        assert_eq!(seven(&v, Form::Progressive)[2], "nä tsyäjku");
        assert_eq!(seven(&v, Form::Want)[0], "sunpa'äs ntsäjku");
        assert_eq!(seven(&v, Form::NegFuture)[1], "ja'nmis manhu ntsäjku");
        assert_eq!(
            seven(&v, Form::NegPresent),
            [
                "ja'näs ntsäki",
                "ja'nmis ntsäki",
                "ja'n tsyäki",
                "ja'ntäs ntsäki",
                "ja'ntä ntsäktame",
                "ja'nmis ntsäktame",
                "ja'n tsyäkyay"
            ]
        );
        assert_eq!(
            seven(&v, Form::NegPast),
            [
                "jantsyäkä'äs",
                "jantsyäkämis",
                "jatsyäkä",
                "jantsyäkätäs",
                "jantsyäktamätä",
                "jantsyäktamämis",
                "jatsyäkya"
            ]
        );
        assert_eq!(seven(&v, Form::Subjunctive)[2], "nyempe tsyäkä");
        assert_eq!(seven(&v, Form::Imperative)[1], "tsäkä");
        assert_eq!(seven(&v, Form::Imperative)[4], "tsäkta'mitäy");
        assert_eq!(seven(&v, Form::NegImperative)[1], "uy tsäki");
    }

    #[test]
    fn sound_changes() {
        // §20.4: yojs-pa → yospa; pejt-tam-u → pettamu; pejt-yaj-u → pejtyaju.
        assert_eq!(join("yojs", "pa"), "yospa");
        assert_eq!(join("pejt", "tam"), "pettam");
        assert_eq!(join("pejt", "yaj"), "pejtyaj");
        assert_eq!(join("kijp", "pa"), "kijpa");
        // §20.2: y + p → py; juy-tam → juytyam (cómprenlo).
        assert_eq!(join("juy", "pa"), "jupya");
        assert_eq!(join("juy", "tam"), "juytyam");
        // §8: pa't- «encontrar»: mpa'tu'äs, pya'tu, mpa'ttamutä.
        let v = verb("pa't", Class::T);
        let past = seven(&v, Form::Past);
        assert_eq!(
            (past[0].as_str(), past[2].as_str(), past[4].as_str()),
            ("mpa'tu'äs", "pya'tu", "mpa'ttamutä")
        );
        // §20.5.5: manh- «ir»: ja'n myawe «no va», mawä «¡vete!».
        let v = verb("manh", Class::I);
        assert_eq!(seven(&v, Form::NegPresent)[2], "ja'n myawe");
        assert_eq!(seven(&v, Form::Imperative)[1], "mawä");
        // pejt- «barrer»: japyeta'äj, ja'n pyete.
        let v = verb("pejt", Class::I);
        assert_eq!(seven(&v, Form::NegPast)[0], "japyeta'äj");
        assert_eq!(seven(&v, Form::NegPresent)[2], "ja'n pyete");
        assert_eq!(seven(&v, Form::Past)[5], "mpyettamumij");
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/zoque.json")).unwrap();
        let c: Conjugation = serde_json::from_value(raw["conjugation"].clone()).unwrap();
        for e in &c.lexicon {
            let r = c.conjugate(&e.entry).unwrap();
            assert!(r.known, "{}", e.entry);
            let past = r.tables.iter().find(|t| t.id == "past").unwrap();
            assert_eq!(
                past.rows[2].singular, e.entry,
                "citation form of {}",
                e.root
            );
            for t in &r.tables {
                for row in &t.rows {
                    assert!(
                        row.singular.is_empty() || row.es_singular.is_some(),
                        "{} {}",
                        e.entry,
                        t.id
                    );
                }
            }
        }
    }
}
