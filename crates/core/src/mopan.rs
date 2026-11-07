//! Conjugation for Mopan Maya, written with the alphabet of the Academia
//! de Lenguas Mayas de Guatemala (ALMG): a ä e i o u, long vowels doubled,
//! b' d' ch ch' j k k' p' t' ts ts' x and the glottal stop ('). No tone.
//!
//! The morphology follows Charles Andrew Hofling, *Mopan Maya–Spanish–
//! English Dictionary* (University of Utah Press, 2011), §4.1–4.2:
//!
//! ```text
//! aspect  set A + verb + status   plural
//! tan     inw-em-el                       tan inwemel   «estoy bajando»
//! walak   u-jan-t-ik              -oo'    walak ujantikoo' «lo comen»
//! —       in-xok-aj                       inxokaj       «lo leí»
//! —       em-een (set B)                  emeen         «bajé»
//! ```
//!
//! Set A is written joined to the verb. The intransitive completive marks
//! the subject with set B (emeen, taleen, b'ini); activity verbs make it
//! with the auxiliary uch «happen» (uchi injanal «comí»); positionals
//! with -l-aj (jitlajeen «me acosté»).

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    /// Root intransitive (riv): emel, okol, tal. Completive with set B.
    I,
    /// Activity verb (actv): janal, meyaj, ts'iib'. Completive with uch.
    A,
    /// Positional (pv) in -tal: jittal, noytal. Completive -l-aj.
    P,
    /// Transitive: ilik (root), jantik (-t-), emesik (causative).
    T,
}

impl Class {
    fn from_kind(kind: &str) -> Option<Class> {
        Some(match kind {
            "I" => Class::I,
            "A" => Class::A,
            "P" => Class::P,
            "T" => Class::T,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Form {
    /// walak: walak inwemel «bajo (de costumbre)».
    Habitual,
    /// tan: tan inwemel «estoy bajando».
    Durative,
    /// emeen, uchi injanal, jitlajeen, inxokaj.
    Completive,
    /// ka' + dependent status: ka' emekech «que bajes».
    Subjunctive,
    /// yan: yan inb'eel «tengo que ir».
    Obligative,
    /// paatal: paatal inb'etik «puedo hacerlo».
    Abilitative,
    /// suk: suken injante «acostumbro comerlo», suk ti b'eel «suele ir».
    Customary,
    /// kajal: kajeen ti b'eel «empecé a ir», kaji ujante «empezó a comerlo».
    Inceptive,
    /// emen «¡baja!», ila «¡míralo!» (2nd person only).
    Imperative,
    /// ma' ab'eel «¡no vayas!» (2nd person only).
    Prohibitive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tense {
    pub id: String,
    pub group: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub form: Form,
    /// ma' t-: ma' tub'eel «no va».
    #[serde(default)]
    pub negative: bool,
    #[serde(default)]
    pub es: Option<spanish::Es>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Citation form (incompletive): "emel", "janal", "jittal", "ilik".
    pub entry: String,
    pub class: Class,
    /// Completive stem when it isn't the root: b'eel → b'in.
    #[serde(default)]
    pub completive: Option<String>,
    /// Dependent stem when it isn't the root: b'eel → xi' (xi'ik).
    /// For transitive verbs, the whole dependent form: ilik → ila'.
    #[serde(default)]
    pub dependent: Option<String>,
    /// Imperative when it isn't the rule: b'eel → xen, ilik → ila.
    #[serde(default)]
    pub imperative: Option<String>,
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
    "aäeiou".contains(c)
}

/// How a transitive stem is built, which decides its dependent and
/// imperative endings (Hofling §4.2.3.3).
#[derive(Clone, Copy, PartialEq)]
enum Transitive {
    /// il-ik → inwila' / ila: a vowel echoing the root.
    Root,
    /// jan-t-ik → injante / jante.
    T,
    /// em-es-ik → inwemes / emes.
    Causative,
}

struct Verb {
    class: Class,
    /// Citation form of an intransitive; transitive stem (il, jant, emes).
    full: String,
    root: String,
    completive: String,
    dependent: String,
    imperative: Option<String>,
    transitive: Transitive,
}

/// emel → em: a final vowel + l repeating the root's last vowel.
fn strip_vl(word: &str) -> Option<&str> {
    let stem = word.strip_suffix('l')?;
    let v = stem.chars().last().filter(|&c| is_vowel(c))?;
    let root = &stem[..stem.len() - v.len_utf8()];
    let last = root.chars().rev().find(|&c| is_vowel(c))?;
    (last == v).then_some(root)
}

fn last_vowel(s: &str) -> char {
    s.chars().rev().find(|&c| is_vowel(c)).unwrap_or('a')
}

impl Verb {
    fn new(class: Class, word: &str) -> Verb {
        let root = match class {
            Class::I => strip_vl(word).unwrap_or(word).to_string(),
            Class::P => word.strip_suffix("tal").unwrap_or(word).to_string(),
            Class::A => word.to_string(),
            Class::T => word.strip_suffix("ik").unwrap_or(word).to_string(),
        };
        let transitive = if root.ends_with("es") && root.chars().count() > 3 {
            Transitive::Causative
        } else if root.ends_with('t') && !root.ends_with("'t") {
            Transitive::T
        } else {
            Transitive::Root
        };
        Verb {
            class,
            full: if class == Class::T {
                root.clone()
            } else {
                word.to_string()
            },
            completive: root.clone(),
            dependent: root.clone(),
            root,
            imperative: None,
            transitive,
        }
    }

    fn from_entry(e: &Entry) -> Verb {
        let mut v = Verb::new(e.class, &e.entry);
        if let Some(c) = &e.completive {
            v.completive = c.clone();
        }
        if let Some(d) = &e.dependent {
            v.dependent = d.clone();
        }
        v.imperative = e.imperative.clone();
        v
    }

    /// Dependent status of a transitive verb: inwila', injante, inwemes.
    /// The dependent status before a plural ending: -e(j) keeps its j,
    /// ujantejoo'.
    fn transitive_dependent_pl(&self, plural: bool, person: usize) -> String {
        let d = self.transitive_dependent();
        if plural && person > 0 && d.ends_with('e') {
            format!("{d}j")
        } else {
            d
        }
    }

    fn transitive_dependent(&self) -> String {
        if self.dependent != self.root {
            return self.dependent.clone();
        }
        match self.transitive {
            Transitive::Root => format!("{}{}'", self.root, last_vowel(&self.root)),
            Transitive::T => format!("{}e", self.root),
            Transitive::Causative => self.root.clone(),
        }
    }
}

fn guess_class(word: &str) -> Class {
    let vowels = word.chars().filter(|&c| is_vowel(c)).count();
    if word.ends_with("ik") && vowels >= 2 {
        Class::T
    } else if word.ends_with("tal") && vowels >= 2 {
        Class::P
    } else if strip_vl(word).is_some() {
        Class::I
    } else {
        Class::A
    }
}

// ---------- person markers ----------

/// Set A, joined to the verb: in(w)-, a(w)-, u(y)-; ti(w)- for «we».
const SET_A: [&str; 3] = ["in", "a", "u"];
/// Plural endings that go with set A: «you all» -e'ex, «they» -oo'.
const PLURAL_A: [&str; 3] = ["", "e'ex", "oo'"];
/// Set B in the completive: -een, -eech, -i, -o'on, -e'ex, -oo'.
const SET_B_COM: [&str; 6] = ["een", "eech", "i", "o'on", "e'ex", "oo'"];
/// Set B in the dependent status: -en, -ech, —, -o'on, -e'ex, -oo'.
const SET_B_DEP: [&str; 6] = ["en", "ech", "", "o'on", "e'ex", "oo'"];

/// Set A joined to the verb, with its plural ending: inwemel, tiwemel,
/// uyemeloo'.
fn set_a(person: usize, plural: bool, verb: &str) -> String {
    let prefix = if plural && person == 0 {
        "ti"
    } else {
        SET_A[person]
    };
    let glide = match prefix {
        "u" if verb.starts_with(is_vowel) => "y",
        _ if verb.starts_with(is_vowel) => "w",
        _ => "",
    };
    let ending = if plural { PLURAL_A[person] } else { "" };
    format!("{prefix}{glide}{verb}{ending}")
}

/// Set A without the plural ending, when the plural goes on an
/// auxiliary: kajoo' ujante «empezaron a comerlo».
fn set_a_bare(person: usize, plural: bool, verb: &str) -> String {
    set_a(person, false, verb).replacen(
        SET_A[person],
        if plural && person == 0 {
            "ti"
        } else {
            SET_A[person]
        },
        1,
    )
}

fn set_b(stem: &str, table: &[&str; 6], person: usize, plural: bool) -> String {
    format!("{stem}{}", table[person + if plural { 3 } else { 0 }])
}

/// ma' t- + the verb; t- + ti- («we») is just ti-: ma' tiwemel.
fn negative(word: &str) -> String {
    if word.starts_with("ti") {
        format!("ma' {word}")
    } else {
        format!("ma' t{word}")
    }
}

fn build(verb: &Verb, tense: &Tense, person: usize, plural: bool) -> Option<String> {
    // The 4th row is the inclusive «we» (with the listener): the «we» form
    // + -e'ex (Hofling §4.1.1): tiwemele'ex, talo'one'ex.
    if person == 3 {
        if !plural || matches!(tense.form, Form::Imperative | Form::Prohibitive) {
            return None;
        }
        return build(verb, tense, 0, true).map(|w| format!("{w}e'ex"));
    }
    build_one(verb, tense, person, plural)
}

fn build_one(verb: &Verb, tense: &Tense, person: usize, plural: bool) -> Option<String> {
    let t = verb.class == Class::T;
    let inc = if t {
        format!("{}ik", verb.root)
    } else {
        verb.full.clone()
    };
    let a = |v: &str| set_a(person, plural, v);
    let word = match tense.form {
        Form::Habitual if tense.negative => negative(&a(&inc)),
        Form::Habitual => format!("walak {}", a(&inc)),
        Form::Durative if tense.negative => negative(&a(&inc)),
        Form::Durative => format!("tan {}", a(&inc)),
        Form::Obligative => format!("yan {}", a(&inc)),
        Form::Abilitative => format!("paatal {}", a(&inc)),
        Form::Completive => match verb.class {
            Class::T => a(&format!("{}aj", verb.root)),
            Class::I => set_b(&verb.completive, &SET_B_COM, person, plural),
            Class::P => set_b(&format!("{}laj", verb.root), &SET_B_COM, person, plural),
            // uchi injanal «comí»: «happen» + set A + the verb.
            Class::A => format!("uchi {}", a(&verb.full)),
        },
        // The subject goes on the auxiliary (set B); then «ti» + the verb, or
        // the transitive verb in the dependent status.
        Form::Customary | Form::Inceptive => {
            let (aux, b) = if tense.form == Form::Customary {
                ("suk", &SET_B_DEP)
            } else {
                ("kaj", &SET_B_COM)
            };
            let rest = if t {
                set_a_bare(person, plural, &verb.transitive_dependent())
            } else {
                format!("ti {}", verb.full)
            };
            format!("{} {rest}", set_b(aux, b, person, plural))
        }
        Form::Subjunctive => {
            let dep = match verb.class {
                Class::T => a(&verb.transitive_dependent_pl(plural, person)),
                Class::I => {
                    // em-ek, tal-ak, wäyn-äk, xi'-ik
                    let v = last_vowel(&verb.dependent);
                    set_b(
                        &format!("{}{v}k", verb.dependent),
                        &SET_B_DEP,
                        person,
                        plural,
                    )
                }
                Class::P => set_b(&format!("{}lak", verb.root), &SET_B_DEP, person, plural),
                Class::A => format!("uchuk {}", a(&verb.full)),
            };
            format!("ka' {dep}")
        }
        Form::Imperative => {
            if person != 1 {
                return None;
            }
            let sg = match (&verb.imperative, verb.class) {
                (Some(i), _) => i.clone(),
                (None, Class::I) => format!("{}en", verb.completive),
                (None, Class::P) => format!("{}len", verb.root),
                (None, Class::A) => format!("uchuk a{}", verb.full),
                (None, Class::T) => match verb.transitive {
                    Transitive::Root => format!("{}{}", verb.root, last_vowel(&verb.root)),
                    Transitive::T => format!("{}e", verb.root),
                    Transitive::Causative => verb.root.clone(),
                },
            };
            return Some(if plural { format!("{sg}e'ex") } else { sg });
        }
        Form::Prohibitive => {
            if person != 1 {
                return None;
            }
            // ma' ab'eel «¡no vayas!»; transitive in the dependent: ma' ajante.
            let v = if t {
                verb.transitive_dependent_pl(plural, person)
            } else {
                inc
            };
            return Some(format!("ma' {}", a(&v)));
        }
    };
    Some(word)
}

fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            '’' | '‘' | 'ʼ' | '´' | '`' => '\'',
            c => c,
        })
        .collect();
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
        let forced = opts.kind().and_then(Class::from_kind);
        let entry = self
            .lexicon
            .iter()
            .find(|e| e.entry == word && forced.is_none_or(|c| c == e.class));
        let verb = match entry {
            Some(e) => Verb::from_entry(e),
            None => Verb::new(forced.unwrap_or_else(|| guess_class(&word)), &word),
        };

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
                    .map(|(i, person)| {
                        let sg = build(&verb, tense, i, false);
                        let pl = build(&verb, tense, i, true);
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

    fn data() -> Conjugation {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/mopan.json")).unwrap();
        serde_json::from_value(raw["conjugation"].clone()).unwrap()
    }

    fn six(verb: &str, tense: &str) -> Vec<String> {
        let r = data().conjugate(verb).unwrap();
        let t = r.tables.iter().find(|t| t.id == tense).unwrap();
        let sg = t.rows.iter().take(3).map(|r| r.singular.clone());
        let pl = t
            .rows
            .iter()
            .take(3)
            .map(|r| r.plural.clone().unwrap_or_default());
        sg.chain(pl).collect()
    }

    #[test]
    fn set_a_paradigms() {
        // Hofling §4.1.1.1: tan intal … tan utaloo'; tan inwemel, tiwemel.
        assert_eq!(
            six("tal", "durative"),
            [
                "tan intal",
                "tan atal",
                "tan utal",
                "tan tital",
                "tan atale'ex",
                "tan utaloo'"
            ]
        );
        assert_eq!(
            six("emel", "durative")[..4],
            ["tan inwemel", "tan awemel", "tan uyemel", "tan tiwemel"]
        );
        assert_eq!(six("emel", "habitual")[5], "walak uyemeloo'");
    }

    #[test]
    fn completive_by_class() {
        // §4.1.1.2: taleen … tali, taloo'; em-eech; b'ini; wäyneen.
        assert_eq!(
            six("tal", "completive"),
            ["taleen", "taleech", "tali", "talo'on", "tale'ex", "taloo'"]
        );
        assert_eq!(six("emel", "completive")[1], "emeech");
        assert_eq!(six("b'eel", "completive")[2], "b'ini");
        assert_eq!(six("wäyäl", "completive")[0], "wäyneen");
        assert_eq!(six("jittal", "completive")[2], "jitlaji");
        assert_eq!(six("janal", "completive")[0], "uchi injanal");
        // Transitive: inwilaj (ALMG), inxokaj.
        assert_eq!(six("ilik", "completive")[0], "inwilaj");
        assert_eq!(six("jantik", "completive")[5], "ujantajoo'");
    }

    #[test]
    fn dependent_and_imperative() {
        // §4.2.2: ka' em-ek-ech, em-en; ka' tal-ak-ech; xi'ik, xen; ila.
        assert_eq!(six("emel", "subjunctive")[1], "ka' emekech");
        assert_eq!(six("tal", "subjunctive")[1], "ka' talakech");
        assert_eq!(six("b'eel", "subjunctive")[2], "ka' xi'ik");
        assert_eq!(six("wäyäl", "subjunctive")[2], "ka' wäynäk");
        assert_eq!(six("jittal", "subjunctive")[2], "ka' jitlak");
        assert_eq!(six("jantik", "subjunctive")[2], "ka' ujante");
        assert_eq!(six("ilik", "subjunctive")[1], "ka' awila'");
        assert_eq!(six("emel", "imperative")[1], "emen");
        assert_eq!(six("wäyäl", "imperative")[1], "wäynen");
        assert_eq!(six("jittal", "imperative")[1], "jitlen");
        assert_eq!(six("b'eel", "imperative")[4], "xene'ex");
        assert_eq!(six("ilik", "imperative")[1], "ila");
        assert_eq!(six("jantik", "imperative")[1], "jante");
        assert_eq!(six("emel", "imperative")[0], "");
    }

    #[test]
    fn negation_and_auxiliaries() {
        // ma' t-u-b'eel; ma' a-b'eel; paatal im-b'et-ik; yan u-b'eel.
        assert_eq!(six("b'eel", "negative")[2], "ma' tub'eel");
        assert_eq!(six("b'eel", "prohibitive")[1], "ma' ab'eel");
        assert_eq!(six("jantik", "prohibitive")[1], "ma' ajante");
        assert_eq!(six("b'etik", "abilitative")[0], "paatal inb'etik");
        assert_eq!(six("b'eel", "obligative")[2], "yan ub'eel");
        assert_eq!(six("emel", "negative")[3], "ma' tiwemel");
        assert_eq!(six("jantik", "subjunctive")[5], "ka' ujantejoo'");
    }

    #[test]
    fn inclusive_customary_inceptive() {
        let r = data().conjugate("emel").unwrap();
        let t = r.tables.iter().find(|t| t.id == "durative").unwrap();
        assert_eq!(t.rows[3].singular, "");
        assert_eq!(t.rows[3].plural.as_deref(), Some("tan tiwemele'ex"));
        let r = data().conjugate("tal").unwrap();
        let t = r.tables.iter().find(|t| t.id == "completive").unwrap();
        assert_eq!(t.rows[3].plural.as_deref(), Some("talo'one'ex"));
        // Hofling: suk(-en) in-jan-t-e; kaj-i ti …; kaj-oo' u-t'ul-u'.
        assert_eq!(six("jantik", "customary")[0], "suken injante");
        assert_eq!(six("b'eel", "customary")[2], "suk ti b'eel");
        assert_eq!(six("b'eel", "inceptive")[0], "kajeen ti b'eel");
        assert_eq!(six("jantik", "inceptive")[5], "kajoo' ujante");
    }

    #[test]
    fn lexicon_round_trip() {
        let c = data();
        for e in &c.lexicon {
            let r = c.conjugate(&e.entry).unwrap();
            assert!(r.known, "{}", e.entry);
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
