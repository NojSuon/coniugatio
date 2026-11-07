//! Conjugation for Central Nahuatl (maseuallâjtolli), written in the SEP
//! orthography Tlâjkuilolpamitl: a e i o; ch j k l m n p s t tl ts u x y,
//! with j for the saltillo, u for /w/, no long vowels, and a circumflex on
//! the vowel that absorbs another when two meet (tla + ijtoa → tlâjtoa).
//!
//! The morphology follows the classical grammars: Thelma D. Sullivan,
//! *Compendio de la gramática náhuatl* (UNAM), and David Charles Wright
//! Carr, *Lectura del náhuatl* (INALI, 2016), §5.1–5.5:
//!
//! ```text
//! (o-) subject   object      verb    tense   plural
//!  o   ni        k           chiu    —              onikchiu   «lo hice»
//!      ti        k           chiua   s       kej    tikchiuaskej «lo haremos»
//! ```
//!
//! Past stems depend on the verb class (§5.2): 1 adds -k (pano → opanok),
//! 2 drops the last vowel (chiua → onikchiu), 3 (-ia, -oa) drops the final a
//! and adds -j (tlâjtoa → otlâjtoj), 4 adds -j (kua → otlakuaj).

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

/// The object (or reflexive) prefix a verb takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Object {
    /// Intransitive: nikochi.
    #[default]
    None,
    /// Specific 3rd person "lo/la": nikchiua.
    Ki,
    /// Indefinite "algo": nitlakua.
    Tla,
    /// Indefinite "alguien": nitemachtia.
    Te,
    /// Reflexive no-/mo-/to-: ninopaktia.
    Mo,
}

impl Object {
    fn from_kind(kind: &str) -> Option<Object> {
        Some(match kind {
            "none" => Object::None,
            "ki" => Object::Ki,
            "tla" => Object::Tla,
            "te" => Object::Te,
            "mo" => Object::Mo,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Form {
    /// nikchiua, tikchiuaj
    Present,
    /// nikchiuaya, tikchiuayaj (copretérito)
    Imperfect,
    /// onikchiu, otikchiukej
    Preterite,
    /// nikchiuas, tikchiuaskej
    Future,
    /// nikchiuani: what one habitually does
    Habitual,
    /// nikchiuatika: going on now
    Progressive,
    /// nikchiuaskia
    Conditional,
    /// ma nikchiua, xikchiua, xikchiuakan (orders and wishes)
    Optative,
    /// ma nikchiu, ma tikchiutin (warning: "que no…")
    Admonitive,
    /// nikchiuatiu: "voy a hacerlo"
    Going,
    /// onikchiuato: "fui a hacerlo"
    Went,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tense {
    pub id: String,
    pub group: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub form: Form,
    /// amo + form.
    #[serde(default)]
    pub negative: bool,
    #[serde(default)]
    pub es: Option<spanish::Es>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Citation form with its object, as shown to people: "kichiua",
    /// "tlâjtoa", "kochi".
    pub entry: String,
    /// Bare present stem: "chiua", "ijtoa", "kochi".
    pub root: String,
    #[serde(default)]
    pub object: Object,
    /// Past-tense class 1–4; inferred from the stem when missing.
    #[serde(default)]
    pub class: Option<u8>,
    /// The stem starts with a "weak" i that merges with a prefix vowel
    /// (ijtoa, itta).
    #[serde(default)]
    pub weak: bool,
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

// ---------- spelling helpers (SEP orthography) ----------

/// In the SEP spelling, u is the consonant /w/; the vowels are a e i o.
fn is_vowel(c: char) -> bool {
    "aeioâêîô".contains(c)
}

fn plain(c: char) -> char {
    match c {
        'â' => 'a',
        'ê' => 'e',
        'î' => 'i',
        'ô' => 'o',
        c => c,
    }
}

fn hat(c: char) -> char {
    match c {
        'a' => 'â',
        'e' => 'ê',
        'i' => 'î',
        'o' => 'ô',
        c => c,
    }
}

/// Consonants at the end of a stem, before its last vowel, counting the
/// digraphs ch, tl, ts as one: chiua → 1 (u), uetska → 2 (ts, k).
fn consonants_before_last_vowel(stem: &str) -> usize {
    let s: Vec<char> = stem.chars().collect();
    let Some(last) = s.iter().rposition(|&c| is_vowel(c)) else {
        return 0;
    };
    let mut i = last;
    let mut n = 0;
    while i > 0 && !is_vowel(s[i - 1]) {
        let digraph =
            i >= 2 && matches!((s[i - 2], s[i - 1]), ('c', 'h') | ('t', 'l') | ('t', 's'));
        i -= if digraph { 2 } else { 1 };
        n += 1;
    }
    n
}

fn vowel_count(stem: &str) -> usize {
    stem.chars().filter(|&c| is_vowel(c)).count()
}

/// Past-tense class from the shape of the stem (§5.2): -ia/-oa → 3; two
/// consonants before the last vowel, or one syllable → 1; otherwise 2,
/// the most common class. Class 4 and the class-1 verbs with CV endings
/// (pano, choka) come from the lexicon.
fn infer_class(root: &str) -> u8 {
    if root.ends_with("ia") || root.ends_with("oa") {
        3
    } else if consonants_before_last_vowel(root) >= 2 || vowel_count(root) <= 1 {
        1
    } else {
        2
    }
}

/// The verb, ready to conjugate.
struct Verb {
    root: String,
    object: Object,
    class: u8,
    weak: bool,
}

impl Verb {
    /// Class-2 past stem: chiua → chiu, nemi → nen, chiya → chix.
    fn dropped(&self) -> String {
        let mut s: String = self
            .root
            .chars()
            .take(self.root.chars().count() - 1)
            .collect();
        if s.ends_with('y') {
            s.pop();
            s.push('x');
        } else if s.ends_with('m') {
            s.pop();
            s.push('n');
        }
        s
    }

    /// Stem without the final a of class 3 (tlâjtoa → tlâjto).
    fn short(&self) -> String {
        if self.class == 3 {
            self.root
                .chars()
                .take(self.root.chars().count() - 1)
                .collect()
        } else {
            self.root.clone()
        }
    }

    /// Past stem and the suffixes for singular and plural.
    fn preterite(&self) -> (String, &'static str, &'static str) {
        match self.class {
            1 => (self.root.clone(), "k", "kej"),
            2 => (self.dropped(), "", "kej"),
            3 => (self.short(), "j", "jkej"),
            _ => (self.root.clone(), "j", "jkej"),
        }
    }

    /// Admonitive stem: like the past, with -j instead of -k (§5.5.3).
    fn admonitive(&self) -> String {
        match self.class {
            1 | 4 => format!("{}j", self.root),
            2 => self.dropped(),
            _ => format!("{}j", self.short()),
        }
    }
}

// ---------- building a word from its pieces ----------

#[derive(Clone, Copy, PartialEq)]
enum Piece {
    Antecessive,
    Subject,
    Object,
    Verb,
    Suffix,
}

/// Joins morphemes, applying the vowel rules at each boundary:
/// - a weak i at the start of the verb is absorbed by a preceding vowel,
///   which takes a circumflex unless it is also i (tla + ijtoa → tlâjtoa,
///   to + itta → tôtta, ki + itta → kitta);
/// - the i of ni-/ti-/xi- drops before a vowel, which takes the circumflex
///   (ni + ajsi → nâjsi);
/// - the vowel of tla-/te-/no-/mo-/to- drops before a vowel, likewise.
fn assemble(pieces: &[(Piece, String)], weak: bool) -> String {
    let mut out: Vec<char> = Vec::new();
    let mut prev: Option<Piece> = None;
    for (kind, text) in pieces {
        let mut next: Vec<char> = text.chars().collect();
        if next.is_empty() {
            continue;
        }
        let meet = out.last().is_some_and(|&c| is_vowel(c)) && is_vowel(next[0]);
        if meet && prev != Some(Piece::Antecessive) {
            let before = plain(*out.last().unwrap());
            if *kind == Piece::Verb && weak && next[0] == 'i' {
                next.remove(0);
                if before != 'i' {
                    let last = out.len() - 1;
                    out[last] = hat(before);
                }
            } else {
                out.pop();
                if plain(next[0]) != before {
                    next[0] = hat(next[0]);
                }
            }
        }
        out.extend(next);
        prev = Some(*kind);
    }
    out.into_iter().collect()
}

const SUBJECT: [[&str; 3]; 2] = [["ni", "ti", ""], ["ti", "an", ""]];
const REFLEXIVE: [[&str; 3]; 2] = [["no", "mo", "mo"], ["to", "mo", "mo"]];

fn build(verb: &Verb, form: Form, person: usize, plural: bool) -> Option<String> {
    let second = person == 1;
    let n = usize::from(plural);
    let (stem, sg, pl): (String, &str, &str) = match form {
        Form::Present => (verb.root.clone(), "", "j"),
        Form::Imperfect => (verb.root.clone(), "ya", "yaj"),
        Form::Preterite => verb.preterite(),
        Form::Future => (verb.short(), "s", "skej"),
        Form::Habitual => (verb.root.clone(), "ni", "nij"),
        Form::Progressive => (verb.root.clone(), "tika", "tikaj"),
        Form::Conditional => (verb.short(), "skia", "skiaj"),
        Form::Optative => (verb.short(), "", "kan"),
        Form::Admonitive => (verb.admonitive(), "", "tin"),
        Form::Going => (verb.root.clone(), "tiu", "tin"),
        Form::Went => (verb.root.clone(), "to", "ton"),
    };

    // Subject: the optative uses xi- for "tú" and "ustedes" (§5.5.2).
    let mut subject = if form == Form::Optative && second {
        "xi"
    } else {
        SUBJECT[n][person]
    }
    .to_string();
    let first_of_rest = match verb.object {
        Object::None => stem.chars().next(),
        _ => Some('k'),
    };
    if subject == "an" && first_of_rest.is_some_and(|c| is_vowel(c) || c == 'm' || c == 'p') {
        subject = "am".into();
    }

    // ki- is k- after a subject vowel or before a vowel (§5.1.3).
    let object = match verb.object {
        Object::None => String::new(),
        Object::Ki => {
            let after_vowel = subject.ends_with(|c: char| is_vowel(c));
            let before_vowel = stem.starts_with(|c: char| is_vowel(c));
            if after_vowel || before_vowel {
                "k".into()
            } else {
                "ki".into()
            }
        }
        Object::Tla => "tla".into(),
        Object::Te => "te".into(),
        Object::Mo => REFLEXIVE[n][person].into(),
    };

    let mut pieces = Vec::new();
    if matches!(form, Form::Preterite | Form::Went) {
        pieces.push((Piece::Antecessive, "o".to_string()));
    }
    pieces.push((Piece::Subject, subject));
    pieces.push((Piece::Object, object));
    pieces.push((Piece::Verb, stem));
    pieces.push((Piece::Suffix, if plural { pl } else { sg }.to_string()));
    let word = assemble(&pieces, verb.weak);

    Some(match form {
        Form::Optative if !second => format!("ma {word}"),
        Form::Admonitive => format!("ma {word}"),
        _ => word,
    })
}

/// Lower-cases, and accepts the circumflex whether or not it is typed.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb.trim().to_lowercase().chars().map(plain).collect();
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_alphabetic()) || w.chars().count() > 40 || !w.chars().any(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

fn plain_word(s: &str) -> String {
    s.chars().map(plain).collect()
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
        let forced = opts.kind().and_then(Object::from_kind);
        // A citation form (kichiua, tlâjtoa) or a bare root (chiua).
        let entry = self.lexicon.iter().find(|e| {
            forced.is_none_or(|o| o == e.object)
                && (plain_word(&e.entry) == word || plain_word(&e.root) == word)
        });
        // A lexicon verb typed with ki- instead of its usual object
        // (kichiua, kijtoa, kitta): same root, conjugated with «lo».
        let with_ki = match (entry, forced) {
            (None, None | Some(Object::Ki)) => ["ki", "k"].iter().find_map(|p| {
                let rest = word.strip_prefix(p)?;
                self.lexicon.iter().find(|e| {
                    let root = plain_word(&e.root);
                    root == rest || (e.weak && root.strip_prefix('i') == Some(rest))
                })
            }),
            _ => None,
        };
        let verb = match (entry, with_ki) {
            (None, Some(e)) => Verb {
                root: e.root.clone(),
                object: Object::Ki,
                class: e.class.unwrap_or_else(|| infer_class(&e.root)),
                weak: e.weak,
            },
            (e, _) => match e {
                Some(e) => Verb {
                    root: e.root.clone(),
                    object: e.object,
                    class: e.class.unwrap_or_else(|| infer_class(&e.root)),
                    weak: e.weak,
                },
                None => Verb {
                    class: infer_class(&word),
                    // A typed verb starting with i is treated as having a weak i
                    // when it takes a prefix with a vowel (ijkuiloa, itta).
                    weak: word.starts_with('i'),
                    object: forced.unwrap_or_default(),
                    root: word.clone(),
                },
            },
        };

        let es_meaning = opts
            .es()
            .map(str::to_string)
            .or_else(|| match (entry, with_ki) {
                (Some(e), _) => e.es.clone(),
                (None, Some(e)) => e.es.as_ref().map(|m| {
                    if opts.english() { format!("{m} it") } else { format!("{m}lo") }
                }),
                _ => None,
            });
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
                        // amo negates; after ma it follows it: ma amo nitlâjto
                        // «que no hable yo» (Wright Carr §5.5.2: macamo).
                        let neg = |f: Option<String>| {
                            f.map(|f| match (tense.negative, f.strip_prefix("ma ")) {
                                (false, _) => f,
                                (true, Some(rest)) => format!("ma amo {rest}"),
                                (true, None) => format!("amo {f}"),
                            })
                        };
                        let sg = neg(build(&verb, tense.form, i, false));
                        let pl = neg(build(&verb, tense.form, i, true));
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

    fn nahuatl() -> Conjugation {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/nahuatl.json")).unwrap();
        serde_json::from_value(raw["conjugation"].clone()).unwrap()
    }

    fn verb(root: &str, object: Object, class: Option<u8>, weak: bool) -> Verb {
        Verb {
            root: root.into(),
            object,
            class: class.unwrap_or_else(|| infer_class(root)),
            weak,
        }
    }

    fn all(v: &Verb, form: Form) -> Vec<String> {
        (0..3)
            .flat_map(|p| {
                [
                    build(v, form, p, false).unwrap(),
                    build(v, form, p, true).unwrap(),
                ]
            })
            .collect()
    }

    #[test]
    fn classes() {
        assert_eq!(infer_class("chiua"), 2);
        assert_eq!(infer_class("kochi"), 2);
        assert_eq!(infer_class("uetska"), 1);
        assert_eq!(infer_class("tsajtsi"), 1);
        assert_eq!(infer_class("itta"), 1);
        assert_eq!(infer_class("ijtoa"), 3);
        assert_eq!(infer_class("machtia"), 3);
    }

    // Wright Carr §5.1.1–5.1.3; Sun's course: tikchiua
    #[test]
    fn present() {
        let chiua = verb("chiua", Object::Ki, None, false);
        assert_eq!(
            all(&chiua, Form::Present),
            [
                "nikchiua",
                "tikchiuaj",
                "tikchiua",
                "ankichiuaj",
                "kichiua",
                "kichiuaj"
            ]
        );
        let paki = verb("paki", Object::None, None, false);
        assert_eq!(
            all(&paki, Form::Present),
            ["nipaki", "tipakij", "tipaki", "ampakij", "paki", "pakij"]
        );
    }

    // §5.2: opanok, onikchiu, otlâjtoj, otlakuaj
    #[test]
    fn preterite_by_class() {
        let pano = verb("pano", Object::None, Some(1), false);
        assert_eq!(build(&pano, Form::Preterite, 2, false).unwrap(), "opanok");
        assert_eq!(build(&pano, Form::Preterite, 2, true).unwrap(), "opanokej");
        let chiua = verb("chiua", Object::Ki, None, false);
        assert_eq!(
            build(&chiua, Form::Preterite, 0, false).unwrap(),
            "onikchiu"
        );
        assert_eq!(
            build(&chiua, Form::Preterite, 0, true).unwrap(),
            "otikchiukej"
        );
        assert_eq!(build(&chiua, Form::Preterite, 2, false).unwrap(), "okichiu");
        let nemi = verb("nemi", Object::None, None, false);
        assert_eq!(build(&nemi, Form::Preterite, 0, false).unwrap(), "oninen");
        let tlajtoa = verb("ijtoa", Object::Tla, None, true);
        assert_eq!(
            build(&tlajtoa, Form::Preterite, 2, false).unwrap(),
            "otlâjtoj"
        );
        assert_eq!(
            build(&tlajtoa, Form::Preterite, 2, true).unwrap(),
            "otlâjtojkej"
        );
        let kua = verb("kua", Object::Tla, Some(4), false);
        assert_eq!(
            build(&kua, Form::Preterite, 0, false).unwrap(),
            "onitlakuaj"
        );
    }

    // The circumflex marks a merged vowel (Sun's course: tlâjtoa, tôttaskej)
    #[test]
    fn vowel_merging() {
        let tlajtoa = verb("ijtoa", Object::Tla, None, true);
        assert_eq!(
            build(&tlajtoa, Form::Present, 0, false).unwrap(),
            "nitlâjtoa"
        );
        assert_eq!(build(&tlajtoa, Form::Present, 2, false).unwrap(), "tlâjtoa");
        let itta = verb("itta", Object::Mo, None, true);
        assert_eq!(build(&itta, Form::Future, 0, true).unwrap(), "titôttaskej");
        assert_eq!(build(&itta, Form::Present, 0, false).unwrap(), "ninôtta");
        let see = verb("itta", Object::Ki, None, true);
        assert_eq!(build(&see, Form::Present, 2, false).unwrap(), "kitta");
        assert_eq!(build(&see, Form::Present, 0, false).unwrap(), "nikitta");
        assert_eq!(build(&see, Form::Preterite, 0, false).unwrap(), "onikittak");
    }

    #[test]
    fn other_tenses() {
        let chiua = verb("chiua", Object::Ki, None, false);
        assert_eq!(build(&chiua, Form::Future, 0, false).unwrap(), "nikchiuas");
        assert_eq!(
            build(&chiua, Form::Imperfect, 0, true).unwrap(),
            "tikchiuayaj"
        );
        assert_eq!(
            build(&chiua, Form::Progressive, 0, false).unwrap(),
            "nikchiuatika"
        );
        assert_eq!(
            build(&chiua, Form::Conditional, 0, false).unwrap(),
            "nikchiuaskia"
        );
        assert_eq!(
            build(&chiua, Form::Habitual, 2, false).unwrap(),
            "kichiuani"
        );
        assert_eq!(build(&chiua, Form::Going, 0, false).unwrap(), "nikchiuatiu");
        assert_eq!(build(&chiua, Form::Going, 0, true).unwrap(), "tikchiuatin");
        assert_eq!(build(&chiua, Form::Went, 0, true).unwrap(), "otikchiuaton");
        assert_eq!(build(&chiua, Form::Went, 0, false).unwrap(), "onikchiuato");
        let ixmati = verb("ixmati", Object::Ki, None, false);
        assert_eq!(
            build(&ixmati, Form::Future, 0, false).unwrap(),
            "nikixmatis"
        );
    }

    // §5.5.2: xitlato, xitlatocan; ma niyauh…
    #[test]
    fn optative_and_admonitive() {
        let chiua = verb("chiua", Object::Ki, None, false);
        assert_eq!(build(&chiua, Form::Optative, 1, false).unwrap(), "xikchiua");
        assert_eq!(
            build(&chiua, Form::Optative, 1, true).unwrap(),
            "xikchiuakan"
        );
        assert_eq!(
            build(&chiua, Form::Optative, 0, false).unwrap(),
            "ma nikchiua"
        );
        let tlajtoa = verb("ijtoa", Object::Tla, None, true);
        assert_eq!(
            build(&tlajtoa, Form::Optative, 1, false).unwrap(),
            "xitlâjto"
        );
        assert_eq!(
            build(&tlajtoa, Form::Optative, 1, true).unwrap(),
            "xitlâjtokan"
        );
        // §5.5.3: ma nitzahtzih, ma tzahtzihtin, ma nitlapouh
        let tsajtsi = verb("tsajtsi", Object::None, None, false);
        assert_eq!(
            build(&tsajtsi, Form::Admonitive, 0, false).unwrap(),
            "ma nitsajtsij"
        );
        assert_eq!(
            build(&tsajtsi, Form::Admonitive, 2, true).unwrap(),
            "ma tsajtsijtin"
        );
        let poua = verb("poua", Object::Tla, None, false);
        assert_eq!(
            build(&poua, Form::Admonitive, 0, false).unwrap(),
            "ma nitlapou"
        );
    }

    #[test]
    fn from_the_data_with_spanish() {
        let c = nahuatl();
        let r = c.conjugate("kichiua").unwrap();
        let t = |id: &str| r.tables.iter().find(|t| t.id == id).unwrap();
        assert_eq!(t("present").rows[0].singular, "nikchiua");
        assert_eq!(
            t("present").rows[0].es_singular.as_deref(),
            Some("yo lo hago")
        );
        assert_eq!(
            t("preterite").rows[2].es_singular.as_deref(),
            Some("él/ella lo hizo")
        );
        assert_eq!(
            t("optative").rows[1].es_singular.as_deref(),
            Some("¡hazlo!")
        );
        assert_eq!(t("neg-present").rows[0].singular, "amo nikchiua");
        assert_eq!(t("neg-optative").rows[1].singular, "amo xikchiua");
        assert_eq!(t("neg-optative").rows[0].singular, "ma amo nikchiua");
        // The circumflex can be typed or not.
        assert_eq!(c.conjugate("tlajtoa").unwrap().root, "tlâjtoa");
        // A bare root finds the entry too.
        assert!(c.conjugate("chiua").unwrap().known);
    }

    #[test]
    fn your_own_verb() {
        let c = nahuatl();
        let opts = Options {
            es: Some("trabajar".into()),
            kind: Some("none".into()),
            ..Default::default()
        };
        let r = c.conjugate_with("tekiti", &opts).unwrap();
        assert_eq!(r.tables[0].rows[0].singular, "nitekiti");
        assert_eq!(
            r.tables[0].rows[0].es_singular.as_deref(),
            Some("yo trabajo")
        );
        let pret = r.tables.iter().find(|t| t.id == "preterite").unwrap();
        assert_eq!(pret.rows[0].singular, "onitekit");
        let opts = Options {
            es: Some("escribirlo".into()),
            kind: Some("ki".into()),
            ..Default::default()
        };
        let r = c.conjugate_with("ijkuiloa", &opts).unwrap();
        assert_eq!(r.tables[0].rows[0].singular, "nikijkuiloa");
    }

    #[test]
    fn site_lexicon_uses_tla_and_accepts_ki() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/nahuatl.json")).unwrap();
        let c: Conjugation = serde_json::from_value(data["conjugation"].clone()).unwrap();
        let present = |verb: &str| {
            let r = c.conjugate(verb).unwrap();
            let rows = &r.tables[0].rows;
            (
                rows[0].singular.clone(),
                rows[2].singular.clone(),
                rows[0].es_singular.clone(),
            )
        };
        for e in &c.lexicon {
            assert!(
                !e.entry.starts_with('k') || e.object == Object::None,
                "{}",
                e.entry
            );
            // The citation form is what the engine gives for the 3rd person.
            assert_eq!(present(&e.entry).1, e.entry, "{}", e.entry);
        }
        assert_eq!(present("tlachiua").0, "nitlachiua");
        // Typed with ki-, the same verb takes «lo».
        let (ni, ye, es) = present("kichiua");
        assert_eq!((ni.as_str(), ye.as_str()), ("nikchiua", "kichiua"));
        assert_eq!(es.as_deref(), Some("yo lo hago"));
        assert_eq!(present("kijtoa").1, "kijtoa");
        assert_eq!(present("kitta").0, "nikitta");
    }
}
