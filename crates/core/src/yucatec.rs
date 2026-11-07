//! Conjugation for Yucatec Maya (maayat'aan), written with the INALI
//! *Normas de escritura para la lengua maya* (2014): long vowels doubled
//! (aa), high tone with an acute (áa), rearticulated vowels (a'a) and the
//! glottal stop ('). The morphology follows Fidencio Briceño Chel, *Los
//! verbos del maya yucateco actual* (INALI, 2021):
//!
//! ```text
//! aspect   set A    verb    status   plural
//! k-       in       xok     -ik               kin xokik       «lo leo»
//! táan     a        wenel            -e'ex    táan a wenele'ex «están durmiendo»
//! t-       u        xok     -aj      -o'ob    tu xokajo'ob    «lo leyeron»
//! ```
//!
//! Two aspects (completive and the "bíin" future) mark the subject with
//! set B at the end instead, and how depends on the kind of intransitive:
//! jweenen «dormí» (root), janalnajen «comí» (activity, -naj), kullajen
//! «me senté» (positional, -laj); bíin weneken, bíin janalnaken, bíin
//! kullaken «algún día…».

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

/// The kind of verb, which decides the completive, the future and the
/// imperative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    /// Root intransitive in -Vl: wenel, okol, kíimil (also bin, taal).
    /// Completive j- + root + set B: jweenen.
    I,
    /// Activity intransitive: janal, meyaj, báaxal. Completive -naj-.
    A,
    /// Positional in -tal: kultal, wa'altal. Completive -laj-.
    P,
    /// Transitive: il-ik, xok-ik, jaant-ik. Status -ik / -aj / -ej.
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
    /// k-: kin wenel «duermo», kin xokik «lo leo».
    Habitual,
    /// táan: táan in wenel «estoy durmiendo».
    Durative,
    /// ts'o'ok: ts'o'ok in wenel «ya dormí».
    Terminative,
    /// táant … -e': táant in wenele' «acabo de dormir».
    Recent,
    /// Completive: jweenen, janalnajen, tin xokaj «lo leí».
    Completive,
    /// úuch + future stem: úuch weneken «hace mucho que dormí».
    Remote,
    /// sáam + future stem: sáam weneken «hace rato que dormí».
    Earlier,
    /// je'el … -e': je'el in wenele' «sí dormiré».
    Assurative,
    /// yaan: yaan in wenel «tengo que dormir».
    Compulsive,
    /// bíin: bíin weneken «algún día dormiré».
    Future,
    /// taak: taak in wenel «quiero dormir».
    Desiderative,
    /// k'a'abet: k'a'abet in wenel «necesito dormir».
    Necessity,
    /// suuk: suuk in wenel «acostumbro dormir».
    Custom,
    /// jo'op': jo'op' in wenel «empecé a dormir».
    Incipient,
    /// weenen «¡duerme!», xokej «¡léelo!» (2nd person only).
    Imperative,
    /// ma' a wenel «¡no duermas!» (2nd person only).
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
    /// ma' … -i'.
    #[serde(default)]
    pub negative: bool,
    #[serde(default)]
    pub es: Option<spanish::Es>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// As in the dictionary: "wenel", "janal", "kultal", "xokik".
    pub entry: String,
    pub class: Class,
    /// Root intransitives: completive stem when it isn't the rule
    /// (k'uchul → k'uch; the rule lengthens a short vowel: wen → ween).
    #[serde(default)]
    pub completive: Option<String>,
    /// Stem of the "bíin" future when it isn't the root: bin → xi', taal →
    /// tal.
    #[serde(default)]
    pub future: Option<String>,
    /// Imperative when it isn't the rule: bin → xeen, taal → kóoten.
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

// ---------- spelling ----------

fn plain(c: char) -> char {
    match c {
        'á' => 'a',
        'é' => 'e',
        'í' => 'i',
        'ó' => 'o',
        'ú' => 'u',
        c => c,
    }
}

fn is_vowel(c: char) -> bool {
    "aeiouáéíóú".contains(c)
}

fn starts_with_vowel(s: &str) -> bool {
    s.chars().next().is_some_and(is_vowel)
}

/// The vowels of a root, without tone: "na'ak" → "aa", "wen" → "e".
fn vowels(s: &str) -> String {
    s.chars().filter(|&c| is_vowel(c)).map(plain).collect()
}

/// The verb, ready to conjugate.
struct Verb {
    class: Class,
    /// Dictionary form of an intransitive (wenel, janal, kultal), or the
    /// transitive stem (xok, jaant, ojéelt).
    full: String,
    /// Root: wen, kul; the same as `full` for activity and transitive verbs.
    root: String,
    completive: String,
    future: String,
    imperative: Option<String>,
}

impl Verb {
    fn new(class: Class, word: &str) -> Verb {
        let root = match class {
            // wenel → wen, kíimil → kíim, na'akal → na'ak
            Class::I => strip_vl(word).unwrap_or(word).to_string(),
            Class::P => word.strip_suffix("tal").unwrap_or(word).to_string(),
            Class::A => word.to_string(),
            // xokik → xok, jaantik → jaant
            Class::T => word.strip_suffix("ik").unwrap_or(word).to_string(),
        };
        // A short single vowel lengthens in the completive: wen → ween,
        // aj → aaj, ok → ook (Briceño §5.1.2).
        let completive = {
            let v = vowels(&root);
            let short = v.chars().count() == 1 && !root.contains(['\'', 'á', 'é', 'í', 'ó', 'ú']);
            if class == Class::I && short {
                let i = root.find(|c: char| is_vowel(c)).unwrap();
                let c = root[i..].chars().next().unwrap();
                format!("{}{c}{}", &root[..i], &root[i..])
            } else {
                root.clone()
            }
        };
        Verb {
            class,
            full: if class == Class::T {
                root.clone()
            } else {
                word.to_string()
            },
            future: root.clone(),
            root,
            completive,
            imperative: None,
        }
    }

    fn from_entry(e: &Entry) -> Verb {
        let mut v = Verb::new(e.class, &e.entry);
        if let Some(c) = &e.completive {
            v.completive = c.clone();
        }
        if let Some(f) = &e.future {
            v.future = f.clone();
        }
        v.imperative = e.imperative.clone();
        v
    }

    /// The form that follows bíin, úuch and sáam (with set B after it):
    /// wen-ek, kíim-ik, na'ak-ak, xi'-ik; janal-nak; kul-lak.
    fn future_stem(&self) -> String {
        match self.class {
            Class::I => {
                let v = vowels(&self.future).chars().next().unwrap_or('a');
                format!("{}{v}k", self.future)
            }
            Class::A => format!("{}nak", self.full),
            Class::P => format!("{}lak", self.root),
            Class::T => self.root.clone(),
        }
    }
}

/// wenel → wen: a final vowel + l, the vowel repeating the root's last one.
fn strip_vl(word: &str) -> Option<&str> {
    let stem = word.strip_suffix('l')?;
    let v = stem.chars().last().filter(|&c| is_vowel(c))?;
    let root = &stem[..stem.len() - v.len_utf8()];
    let last = root.chars().rev().find(|&c| is_vowel(c))?;
    (plain(last) == plain(v)).then_some(root)
}

/// The class of a verb that isn't in the lexicon, from its shape.
fn guess_class(word: &str) -> Class {
    if word.ends_with("ik") && vowels(word).chars().count() >= 2 {
        Class::T
    } else if word.ends_with("tal") && vowels(word).chars().count() >= 2 {
        Class::P
    } else if strip_vl(word).is_some() {
        Class::I
    } else {
        Class::A
    }
}

// ---------- person markers ----------

/// Set A, before the verb: in, a, u; k, a … -e'ex, u … -o'ob.
const SET_A: [&str; 3] = ["in", "a", "u"];
/// Set B, after the verb: -en, -ech, -ij; -o'on, -e'ex, -o'ob.
const SET_B: [&str; 3] = ["en", "ech", ""];
const SET_B_PL: [&str; 3] = ["o'on", "e'ex", "o'ob"];
/// The plural ending that goes with set A.
const PLURAL_A: [&str; 3] = ["", "e'ex", "o'ob"];

/// The aspect word, set A and the verb: "táan in wenel", "kin wenel",
/// "tu xokajo'ob". `fuse` is k- or t-, which joins the pronoun (kin, tin;
/// k + k → k).
fn with_set_a(
    aspect: Option<&str>,
    fuse: Option<char>,
    person: usize,
    plural: bool,
    verb: &str,
) -> String {
    let pronoun = if plural && person == 0 {
        "k"
    } else {
        SET_A[person]
    };
    // in w-okol, a w-okol, u y-okol; k okol
    let glide = match pronoun {
        "in" | "a" if starts_with_vowel(verb) => "w",
        "u" if starts_with_vowel(verb) => "y",
        _ => "",
    };
    let pronoun = match (fuse, pronoun) {
        (Some(f), "k") if f == 'k' => "k".to_string(),
        (Some(f), "k") => format!("{f} k"),
        (Some(f), p) => format!("{f}{p}"),
        (None, p) => p.to_string(),
    };
    let ending = if plural { PLURAL_A[person] } else { "" };
    let mut out = String::new();
    if let Some(a) = aspect {
        out.push_str(a);
        out.push(' ');
    }
    out.push_str(&format!("{pronoun} {glide}{verb}{ending}"));
    out
}

fn with_set_b(stem: &str, person: usize, plural: bool, completive: bool) -> String {
    let b = match (plural, person) {
        (true, p) => SET_B_PL[p],
        // -ij for the 3rd person of the completive: jkíimij, janalnajij.
        (false, 2) if completive => "ij",
        (false, p) => SET_B[p],
    };
    format!("{stem}{b}")
}

fn build(verb: &Verb, tense: &Tense, person: usize, plural: bool) -> Option<String> {
    let t = verb.class == Class::T;
    // Status of a transitive verb: -ik, -aj, or -ej (only at the end of a
    // singular form: bíin in xokej, bíin u xoko'ob).
    let ik = |s: &str| {
        if t {
            format!("{}{s}", verb.root)
        } else {
            verb.full.clone()
        }
    };
    let a = |aspect: Option<&str>, fuse: Option<char>, v: &str| {
        with_set_a(aspect, fuse, person, plural, v)
    };
    let word = match tense.form {
        Form::Habitual if tense.negative => a(Some("ma'"), Some('t'), &ik("ik")),
        Form::Habitual => a(None, Some('k'), &ik("ik")),
        Form::Durative => a(Some("táan"), None, &ik("ik")),
        Form::Terminative => a(Some("ts'o'ok"), None, &ik("ik")),
        Form::Recent => format!("{}e'", a(Some("táant"), None, &ik("ik"))),
        Form::Assurative => format!("{}e'", a(Some("je'el"), None, &ik("ik"))),
        Form::Compulsive => a(Some("yaan"), None, &ik("ik")),
        Form::Desiderative => a(Some("taak"), None, &ik("ik")),
        Form::Necessity => a(Some("k'a'abet"), None, &ik("ik")),
        Form::Custom => a(Some("suuk"), None, &ik("ik")),
        Form::Incipient => a(Some("jo'op'"), None, &ik("ik")),
        Form::Completive => {
            let w = match verb.class {
                Class::T => a(None, Some('t'), &format!("{}aj", verb.root)),
                Class::I => with_set_b(&format!("j{}", verb.completive), person, plural, true),
                Class::A => with_set_b(&format!("{}naj", verb.full), person, plural, true),
                Class::P => with_set_b(&format!("{}laj", verb.root), person, plural, true),
            };
            if tense.negative {
                format!("ma' {w}")
            } else {
                w
            }
        }
        Form::Future | Form::Remote | Form::Earlier => {
            let aux = match tense.form {
                Form::Future => "bíin",
                Form::Remote => "úuch",
                _ => "sáam",
            };
            if t {
                let ej = if plural { "" } else { "ej" };
                a(Some(aux), None, &format!("{}{ej}", verb.root))
            } else {
                format!(
                    "{aux} {}",
                    with_set_b(&verb.future_stem(), person, plural, false)
                )
            }
        }
        Form::Imperative => {
            if person != 1 {
                return None;
            }
            let sg = match (&verb.imperative, verb.class) {
                (Some(i), _) => i.clone(),
                (None, Class::I) => format!("{}en", verb.completive),
                (None, Class::A) => format!("{}nen", verb.full),
                (None, Class::P) => format!("{}en", verb.root),
                (None, Class::T) => format!("{}ej", verb.root),
            };
            return Some(match (plural, verb.class) {
                (false, _) => sg,
                // xokej → xoke'ex; weenen → weenene'ex
                (true, Class::T) if verb.imperative.is_none() => format!("{}e'ex", verb.root),
                (true, _) => format!("{sg}e'ex"),
            });
        }
        Form::Prohibitive => {
            if person != 1 {
                return None;
            }
            return Some(a(Some("ma'"), None, &ik("ik")));
        }
    };
    // ma' … -i' closes a negative: ma' tin weneli'.
    Some(if tense.negative {
        if word.starts_with("ma' ") {
            format!("{word}i'")
        } else {
            format!("ma' {word}i'")
        }
    } else {
        word
    })
}

// ---------- input ----------

/// Lower-cases and writes every glottal stop as a straight apostrophe.
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
            serde_json::from_str(include_str!("../../../data/languages/maya.json")).unwrap();
        serde_json::from_value(raw["conjugation"].clone()).unwrap()
    }

    /// The six forms of one tense: 1sg, 2sg, 3sg, 1pl, 2pl, 3pl.
    fn six(verb: &str, tense: &str) -> Vec<String> {
        six_as(verb, tense, None)
    }

    fn six_as(verb: &str, tense: &str, kind: Option<&str>) -> Vec<String> {
        let opts: Options = serde_json::from_value(serde_json::json!({ "kind": kind })).unwrap();
        let r = data().conjugate_with(verb, &opts).unwrap();
        let t = r.tables.iter().find(|t| t.id == tense).unwrap();
        let sg = t.rows.iter().map(|r| r.singular.clone());
        let pl = t.rows.iter().map(|r| r.plural.clone().unwrap_or_default());
        sg.chain(pl).collect()
    }

    #[test]
    fn habitual_with_set_a() {
        // Briceño §1.1: k-in janal … k-u janal-o'ob; k-in w-áalkab.
        assert_eq!(
            six("janal", "habitual"),
            [
                "kin janal",
                "ka janal",
                "ku janal",
                "k janal",
                "ka janale'ex",
                "ku janalo'ob"
            ]
        );
        assert_eq!(
            six("áalkab", "habitual")[..3],
            ["kin wáalkab", "ka wáalkab", "ku yáalkab"]
        );
        assert_eq!(six("áalkab", "habitual")[3], "k áalkab");
    }

    #[test]
    fn aspects_of_janal() {
        // Briceño §1.2.2.1.
        let first = |t: &str| six("janal", t)[0].clone();
        assert_eq!(first("durative"), "táan in janal");
        assert_eq!(first("terminative"), "ts'o'ok in janal");
        assert_eq!(first("recent"), "táant in janale'");
        assert_eq!(first("compulsive"), "yaan in janal");
        assert_eq!(first("assurative"), "je'el in janale'");
        assert_eq!(first("desiderative"), "taak in janal");
        assert_eq!(first("necessity"), "k'a'abet in janal");
        assert_eq!(first("incipient"), "jo'op' in janal");
    }

    #[test]
    fn completive_by_class() {
        // Briceño §5.1: janal-naj-en; j-ween-o'on; kul-laj-en; j-bin-en.
        assert_eq!(
            six("janal", "completive"),
            [
                "janalnajen",
                "janalnajech",
                "janalnajij",
                "janalnajo'on",
                "janalnaje'ex",
                "janalnajo'ob"
            ]
        );
        assert_eq!(six("wenel", "completive")[3], "jweeno'on");
        assert_eq!(six("ajal", "completive")[5], "jaajo'ob");
        assert_eq!(six("okol", "completive")[1], "jookech");
        assert_eq!(six("kíimil", "completive")[2], "jkíimij");
        assert_eq!(six("k'uchul", "completive")[5], "jk'ucho'ob");
        assert_eq!(six("bin", "completive")[0], "jbinen");
        assert_eq!(six("kultal", "completive")[0], "kullajen");
        // Transitive: t-in xok-aj.
        assert_eq!(six("xokik", "completive")[0], "tin xokaj");
        assert_eq!(six("xokik", "completive")[5], "tu xokajo'ob");
        assert_eq!(six("ilik", "completive")[2], "tu yilaj");
    }

    #[test]
    fn future_by_class() {
        // Briceño §5.2: bíin wen-ek-o'on, bíin xi'-ik-en, bíin tal-ak-en.
        assert_eq!(six("wenel", "future")[3], "bíin weneko'on");
        assert_eq!(six("bin", "future")[0], "bíin xi'iken");
        assert_eq!(six("taal", "future")[0], "bíin talaken");
        assert_eq!(six("na'akal", "future")[2], "bíin na'akak");
        assert_eq!(six("k'uchul", "future")[5], "bíin k'uchuko'ob");
        assert_eq!(six("báaxal", "future")[0], "bíin báaxalnaken");
        assert_eq!(six("kultal", "future")[3], "bíin kullako'on");
        // Transitive -ej only in the singular (Norma §3.4.2.7).
        assert_eq!(six("xokik", "future")[0], "bíin in xokej");
        assert_eq!(six("xokik", "future")[5], "bíin u xoko'ob");
        assert_eq!(six("báaxal", "remote")[0], "úuch báaxalnaken");
    }

    #[test]
    fn transitive_aspects() {
        assert_eq!(
            six("ilik", "habitual")[..4],
            ["kin wilik", "ka wilik", "ku yilik", "k ilik"]
        );
        assert_eq!(six("jaantik", "durative")[0], "táan in jaantik");
        assert_eq!(six("a'alik", "recent")[2], "táant u ya'alike'");
    }

    #[test]
    fn imperative_and_prohibitive() {
        // Norma §3.4.2.6: xooknen, aajen, éemen, xokej; plural -e'ex.
        assert_eq!(six("xook", "imperative")[1], "xooknen");
        assert_eq!(six("xook", "imperative")[4], "xooknene'ex");
        assert_eq!(six("ajal", "imperative")[1], "aajen");
        assert_eq!(six("éemel", "imperative")[1], "éemen");
        assert_eq!(six("xokik", "imperative")[1], "xokej");
        assert_eq!(six("xokik", "imperative")[4], "xoke'ex");
        assert_eq!(six("bin", "imperative")[1], "xeen");
        assert_eq!(six("taal", "imperative")[4], "kóotene'ex");
        assert_eq!(six("wenel", "imperative")[0], "");
        assert_eq!(six("wenel", "prohibitive")[1], "ma' a wenel");
    }

    #[test]
    fn negatives() {
        // ma' tin … -i' (Norma §2.10: ma' tin ch'a'aj in áanalte'i').
        assert_eq!(six("wenel", "neg-habitual")[0], "ma' tin weneli'");
        assert_eq!(six("ojéeltik", "neg-habitual")[0], "ma' tin wojéeltiki'");
        assert_eq!(six("xokik", "neg-completive")[0], "ma' tin xokaji'");
        assert_eq!(six("bin", "neg-completive")[0], "ma' jbineni'");
        assert_eq!(six("wenel", "neg-durative")[0], "ma' táan in weneli'");
    }

    #[test]
    fn unknown_verbs() {
        // Shape: -ik transitive, -tal positional, -Vl root intransitive.
        assert_eq!(six("kaxtik", "habitual")[0], "kin kaxtik");
        assert_eq!(six("wa'altal", "completive")[0], "wa'allajen");
        assert_eq!(six("lúubul", "completive")[0], "jlúuben");
        assert_eq!(six("meyaj", "completive")[0], "meyajnajen");
        // The type can be chosen: koonol as an activity verb.
        assert_eq!(
            six_as("koonol", "completive", Some("A"))[3],
            "koonolnajo'on"
        );
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
