//! Conjugation for Lacandon Maya (Southern Lacandon of Lacanjá Chansayab),
//! written with the ALMG alphabet as Hofling records it (initial glottal
//! stops, high tone on long vowels), with ts for tz: a ä e i o u (long aa,
//! high tone áa); b ch ch' j k k' m n p p' r s t t' ts ts' w x y '.
//!
//! The morphology follows Charles Andrew Hofling, *Lacandon Maya–Spanish–
//! English Dictionary* (University of Utah Press, 2014), §4.1–4.2:
//!
//! ```text
//! aspect  set A + verb + status       plural
//! k-      in-bin                                kinbin       «voy»
//! táan    'uy-aj-är                              táan 'uyajär  «se despierta»
//! t-      in-kíins-aj                            tinkíinsaj    «lo maté»
//! —       ween (set B)                -o'b      weeno'b      «durmieron»
//! ```
//!
//! Set A goes joined to the verb; the intransitive status is -Vr (-Vn
//! after a nasal: weenan), the dependent -Vk, the imperative -en. Activity
//! verbs take -n- outside the incompletive (ts'íibnäj «escribió»), and
//! positionals -r- (kurräj «se sentó»).

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    /// Root intransitive: weenan, 'ookar, bin.
    I,
    /// Activity verb (actv): ts'íib, k'aay, che'ej.
    A,
    /// Positional: kurtaar.
    P,
    /// Transitive: 'irik, beetik, jaantik.
    T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// k-: kinbin «voy».
    Habitual,
    /// táan: táan 'inbin «estoy yendo».
    Durative,
    /// t- … -aj (transitive), set B (intransitive): binen, tinwiraj.
    Completive,
    /// taant: «acabo de…».
    Recent,
    /// ts'o'kar: «terminé de…».
    Terminative,
    /// taabar: «voy a…» (futuro inmediato).
    Immediate,
    /// chúunur: «empiezo a…».
    Inceptive,
    /// je'r … -e: je'r 'inbine «sí iré».
    Assurative,
    /// yaan: «tengo que…».
    Obligative,
    /// taak: «quiero…».
    Desiderative,
    /// suuk: «acostumbro…».
    Customary,
    /// ka' + dependent status: ka' xi'iken «que yo vaya».
    Subjunctive,
    /// -en, -ej: weenen «¡duerme!», 'irej «¡míralo!».
    Imperative,
    /// ma' t-: ma' tinbin «no voy».
    NegHabitual,
    /// ma' + completive: ma' tinkíinsaj «no lo maté».
    NegCompletive,
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
    /// Citation form (incompletive): "weenan", "bin", "ts'íib", "'irik".
    pub entry: String,
    /// Root: "ween", "bin", "ts'íib", "'ir".
    pub root: String,
    pub class: Class,
    /// Dependent stem when irregular: bin → xi' (xi'ik).
    #[serde(default)]
    pub dependent: Option<String>,
    /// Imperative when irregular: bin → xeen.
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
    "aäeiouáéíóú".contains(c)
}

/// The vowel of the harmonic -Vr / -Vk suffixes (§1.2.3.7–1.2.3.9): a, ä →
/// ä; e, o → a; i → i; u → u.
fn harmonic(root: &str) -> char {
    match root.chars().rev().find(|&c| is_vowel(c)).unwrap_or('a') {
        'i' | 'í' => 'i',
        'u' | 'ú' => 'u',
        'e' | 'é' | 'o' | 'ó' => 'a',
        _ => 'ä',
    }
}

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

    /// Set B: -en, -ech, -ij / Ø; -o'n, -o'ne'x (incl.), -eche'x, -o'b.
    fn set_b(self, completive: bool) -> &'static str {
        match self {
            Slot::S1 => "en",
            Slot::S2 => "ech",
            Slot::S3 if completive => "ij",
            Slot::S3 => "",
            Slot::P1x => "o'n",
            Slot::P1i => "o'ne'x",
            Slot::P2 => "eche'x",
            Slot::P3 => "o'b",
        }
    }
}

/// Set A joined to the verb, with its plural ending: inbin, inwajär
/// (before a vowel the glottal stop drops and w / y appears), ik'ajär,
/// abineche'x, ubino'b.
fn set_a(slot: Slot, stem: &str) -> String {
    let vowel = stem.starts_with('\'') && stem.chars().nth(1).is_some_and(is_vowel);
    let bare = stem.trim_start_matches('\'');
    let (prefix, glide) = match slot {
        Slot::S1 => ("in", "w"),
        Slot::S2 | Slot::P2 => ("a", "w"),
        Slot::S3 | Slot::P3 => ("u", "y"),
        Slot::P1x | Slot::P1i => ("ik", ""),
    };
    let body = match (vowel, glide) {
        (true, "") => stem.to_string(),
        (true, g) => format!("{g}{bare}"),
        (false, _) => stem.to_string(),
    };
    let ending = match slot {
        Slot::P1i => "e'x",
        Slot::P2 => "eche'x",
        Slot::P3 => "o'b",
        _ => "",
    };
    format!("{prefix}{body}{ending}")
}

struct Verb<'a> {
    e: &'a Entry,
}

impl Verb<'_> {
    /// The incompletive form that follows set A: the citation form.
    fn incompletive(&self) -> String {
        self.e.entry.clone()
    }

    fn completive(&self, slot: Slot) -> String {
        let r = &self.e.root;
        match self.e.class {
            Class::T => format!("t{}", set_a(slot, &format!("{r}aj"))),
            Class::I => format!("{r}{}", slot.set_b(true)),
            Class::A => format!("{r}näj{}", slot.set_b(false)),
            Class::P => format!("{r}räj{}", slot.set_b(false)),
        }
    }

    fn dependent(&self, slot: Slot) -> String {
        let r = &self.e.root;
        match self.e.class {
            Class::T => format!("'{}", set_a(slot, &format!("{r}ej"))),
            Class::I => {
                let stem = self.e.dependent.clone().unwrap_or_else(|| r.clone());
                format!("{stem}{}k{}", harmonic(&stem), slot.set_b(false))
            }
            Class::A => format!("{r}näk{}", slot.set_b(false)),
            Class::P => format!("{r}räk{}", slot.set_b(false)),
        }
    }

    fn imperative(&self, plural: bool) -> String {
        let r = &self.e.root;
        let sg = match (&self.e.imperative, self.e.class) {
            (Some(i), _) => i.clone(),
            (None, Class::I) => format!("{r}en"),
            (None, Class::A) => format!("{r}nen"),
            (None, Class::P) => format!("{r}ren"),
            (None, Class::T) => format!("{r}ej"),
        };
        if plural {
            format!("{sg}e'x")
        } else {
            sg
        }
    }
}

fn build(v: &Verb, form: Form, slot: Slot) -> Option<String> {
    let inc = v.incompletive();
    let word = |aux: &str| format!("{aux} '{}", set_a(slot, &inc));
    Some(match form {
        Form::Habitual => format!("k{}", set_a(slot, &inc)),
        Form::Durative => word("táan"),
        Form::Completive => v.completive(slot),
        Form::Recent => word("taant"),
        Form::Terminative => word("ts'o'kar"),
        Form::Immediate => word("taabar"),
        Form::Inceptive => word("chúunur"),
        Form::Assurative => format!("{}e", word("je'r")),
        Form::Obligative => word("yaan"),
        Form::Desiderative => word("taak"),
        Form::Customary => word("suuk"),
        Form::Subjunctive => format!("ka' {}", v.dependent(slot)),
        Form::Imperative => match slot {
            Slot::S2 => v.imperative(false),
            Slot::P2 => v.imperative(true),
            _ => return None,
        },
        Form::NegHabitual => format!("ma' t{}", set_a(slot, &inc)),
        Form::NegCompletive => format!("ma' {}", v.completive(slot)),
    })
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
        .collect::<String>()
        .replace("tz", "ts")
        .replace("b'", "b");
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_alphabetic() || c == '\'') || w.chars().count() > 40 || !w.chars().any(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

/// A verb that isn't in the list, from its shape: -ik transitive, -taar
/// positional, -Vr / -Vn root intransitive, otherwise an activity verb.
fn guess(word: &str, kind: Option<&str>) -> Entry {
    let class = match kind {
        Some("I") => Class::I,
        Some("A") => Class::A,
        Some("P") => Class::P,
        Some("T") => Class::T,
        _ if word.ends_with("ik") => Class::T,
        _ if word.ends_with("taar") => Class::P,
        _ if word.ends_with('r') || word.ends_with("an") => Class::I,
        _ => Class::A,
    };
    let root = match class {
        Class::T => word.strip_suffix("ik").unwrap_or(word).to_string(),
        Class::P => word.strip_suffix("taar").unwrap_or(word).to_string(),
        Class::I => {
            let c: Vec<char> = word.chars().collect();
            if c.len() > 3 && (c[c.len() - 1] == 'r' || c[c.len() - 1] == 'n') && is_vowel(c[c.len() - 2]) {
                c[..c.len() - 2].iter().collect()
            } else {
                word.to_string()
            }
        }
        Class::A => word.to_string(),
    };
    Entry {
        entry: word.to_string(),
        root,
        class,
        dependent: None,
        imperative: None,
        meaning: String::new(),
        es: None,
        note: None,
    }
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.lexicon.iter().find(|e| e.entry == word || e.root == word);
        let guessed;
        let e = match entry {
            Some(e) => e,
            None => {
                guessed = guess(&word, opts.kind());
                &guessed
            }
        };
        let v = Verb { e };

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
                    .enumerate()
                    .map(|(i, person)| {
                        let sg = Slot::of(i, false).and_then(|s| build(&v, tense.form, s));
                        let pl = Slot::of(i, true).and_then(|s| build(&v, tense.form, s));
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
            meaning: entry.map(|e| e.meaning.clone()).or_else(|| opts.es().map(str::to_string)),
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
            serde_json::from_str(include_str!("../../../data/languages/lacandon.json")).unwrap();
        serde_json::from_value(raw["conjugation"].clone()).unwrap()
    }

    fn seven(verb: &str, form: Form) -> Vec<String> {
        let c = data();
        let e = c.lexicon.iter().find(|e| e.entry == verb).unwrap();
        let v = Verb { e };
        [Slot::S1, Slot::S2, Slot::S3, Slot::P1x, Slot::P1i, Slot::P2, Slot::P3]
            .iter()
            .map(|&s| build(&v, form, s).unwrap_or_default())
            .collect()
    }

    // Hofling (2014) §4.1–4.2, written as words.
    #[test]
    fn paradigms() {
        // §4.1.1.1: k-in-bin … k-ik-bin-e'x, k-u-bin-o'b.
        assert_eq!(
            seven("bin", Form::Habitual),
            ["kinbin", "kabin", "kubin", "kikbin", "kikbine'x", "kabineche'x", "kubino'b"]
        );
        // k-inw-aj-är, k-ik-'aj-är, k-uy-aj-är-o'b.
        let w = seven("'ajär", Form::Habitual);
        assert_eq!((w[0].as_str(), w[3].as_str(), w[6].as_str()), ("kinwajär", "kik'ajär", "kuyajäro'b"));
        // táan 'uy-aj-är «se despierta».
        assert_eq!(seven("'ajär", Form::Durative)[2], "táan 'uyajär");
        // §4.1.1.2: bin-(e)en, bin(-ij), bin-o'b.
        let c = seven("bin", Form::Completive);
        assert_eq!((c[0].as_str(), c[2].as_str(), c[6].as_str()), ("binen", "binij", "bino'b"));
        // §4.2.2.2: k-a-ween-an, ween-(e)ech, ka' ween-ak-(e)ech, ween-en!
        assert_eq!(seven("weenan", Form::Habitual)[1], "kaweenan");
        assert_eq!(seven("weenan", Form::Completive)[1], "weenech");
        assert_eq!(seven("weenan", Form::Subjunctive)[1], "ka' weenakech");
        assert_eq!(seven("weenan", Form::Imperative)[1], "weenen");
        // xi'ik-ech «que te vayas»; xeen! «¡vete!».
        assert_eq!(seven("bin", Form::Subjunctive)[1], "ka' xi'ikech");
        assert_eq!(seven("bin", Form::Imperative)[1], "xeen");
        // §4.2.2.1 (ch'äk-): k-a-ch'äk-ik, t-a-ch'äk-aj, ka' 'a-ch'äk-ej, ch'äk-ej!
        assert_eq!(seven("ch'äkik", Form::Habitual)[1], "kach'äkik");
        assert_eq!(seven("ch'äkik", Form::Completive)[1], "tach'äkaj");
        assert_eq!(seven("ch'äkik", Form::Subjunctive)[1], "ka' 'ach'äkej");
        assert_eq!(seven("ch'äkik", Form::Imperative)[1], "ch'äkej");
        // k-uy-irik «lo ve», ma' t-in-kíin-s-aj «no lo maté».
        assert_eq!(seven("'irik", Form::Habitual)[2], "kuyirik");
        assert_eq!(seven("kíinsik", Form::NegCompletive)[0], "ma' tinkíinsaj");
        // §4.2.3.2.3: k-u-ts'íib, ts'íib-n-äj, ka' ts'íib-n-äk, ts'íib-n-en!
        assert_eq!(seven("ts'íib", Form::Habitual)[2], "kuts'íib");
        assert_eq!(seven("ts'íib", Form::Completive)[2], "ts'íibnäj");
        assert_eq!(seven("ts'íib", Form::Subjunctive)[2], "ka' ts'íibnäk");
        assert_eq!(seven("ts'íib", Form::Imperative)[1], "ts'íibnen");
        // §4.2.3.2.1: k-u-kur-taar, kur-r-äj, ka' kur-r-äk, kur-r-en!
        assert_eq!(seven("kurtaar", Form::Habitual)[2], "kukurtaar");
        assert_eq!(seven("kurtaar", Form::Completive)[2], "kurräj");
        assert_eq!(seven("kurtaar", Form::Imperative)[1], "kurren");
        // suuk 'u-beet-ik «tiene costumbre de hacerlo».
        assert_eq!(seven("beetik", Form::Customary)[2], "suuk 'ubeetik");
        // ma' t-u-… (negative incompletive).
        assert_eq!(seven("bin", Form::NegHabitual)[2], "ma' tubin");
    }

    #[test]
    fn lexicon_round_trip() {
        let c = data();
        for e in &c.lexicon {
            let r = c.conjugate(&e.entry).unwrap();
            assert!(r.known, "{}", e.entry);
            for t in &r.tables {
                for row in &t.rows {
                    assert!(row.singular.is_empty() || row.es_singular.is_some(), "{} {}", e.entry, t.id);
                }
            }
        }
    }
}
