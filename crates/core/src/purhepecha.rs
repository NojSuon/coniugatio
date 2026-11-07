//! Conjugation for P'urhépecha (purépecha), written with the current
//! alphabet: a e i ï o u; ch ch' j k k' m n nh p p' r rh s t t' ts ts' x.
//!
//! P'urhépecha has no prefixes: everything is added after the root, in this
//! order (Hernández, Fabián y Coló, *Curso de lengua p'urhépecha*, UNAM,
//! §3; Medina Pérez, *Guía lingüística p'urhépecha*):
//!
//! ```text
//! root    aspect  tense  mode      person (plural)
//! uanda   xa      —      ka        —         uandaxaka     «estoy hablando»
//! uanda   s       p      ti        ksï       uandasptiksï  «hablaron»
//! uanda   a              ka        jtsï      uandaakajtsï  «hablarán ustedes»
//! ```
//!
//! - Aspect: xa actual (now), sïn habitual, s perfective, xam inceptive
//!   ("about to"). In the past habitual is sïam, and inceptive xanam.
//! - Tense: p past, a future, pirin conditional; nothing for the present.
//! - Mode: ka for the 1st and 2nd persons, ti for the 3rd; ga / di after a
//!   nasal (sïnga, piringa). Questions have their own endings for every
//!   person: -xaki, -sïni, -ski, -spi…
//! - Plural: -ksï for «nosotros» and «ellos», -jtsï for «ustedes».
//! - A plural direct object adds -a- after the root: exe-a-xati «los está
//!   viendo»; with the future a it becomes -aua-: exe-aua-ka «los veré».

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Form {
    /// -xa: what is going on right now (piréxaka «estoy cantando»).
    Actual,
    /// -sïn: what one does, or knows how to do (pirésïnga «canto»).
    Habitual,
    /// -s: done (piréska «he cantado»; past piréspka «canté»).
    Perfective,
    /// -xam: about to happen (piréxamga «estoy a punto de cantar»).
    Inceptive,
    /// -a: piréaka «cantaré».
    Future,
    /// -pirin: pirépiringa «cantaría».
    Conditional,
    /// The bare root: piré «¡canta!», pirée «¡canten!».
    Imperative,
    /// ásï + root: ásï piré «¡no cantes!».
    Vetative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tense {
    pub id: String,
    pub group: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub form: Form,
    /// The past (-p-) of actual, habitual, perfective and inceptive.
    #[serde(default)]
    pub past: bool,
    /// The question form (¿piréxaki?).
    #[serde(default)]
    pub question: bool,
    /// no + form.
    #[serde(default)]
    pub negative: bool,
    #[serde(default)]
    pub es: Option<spanish::Es>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Infinitive, as in the dictionary: "piréni".
    pub entry: String,
    /// Root, the infinitive without -ni: "piré".
    pub root: String,
    /// Future stem of the irregular verbs jarhani, jurhani, nirani…:
    /// "jau" (jauaka «estaré»).
    #[serde(default)]
    pub future: Option<String>,
    /// Conditional stem of those verbs, their first syllable: "ja"
    /// (japiringa «estaría»).
    #[serde(default)]
    pub short: Option<String>,
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

/// The verb, ready to conjugate.
struct Verb {
    root: String,
    future: Option<String>,
    short: Option<String>,
    /// Takes the plural-object -a-.
    plural_object: bool,
}

/// -ksï «nosotros, ellos», -jtsï «ustedes».
const PLURAL: [&str; 3] = ["ksï", "jtsï", "ksï"];

fn is_nasal(c: char) -> bool {
    c == 'n' || c == 'm'
}

/// ka / ti, voiced to ga / di after a nasal: sïn + ka → sïnga.
fn mode(before: &str, person: usize) -> &'static str {
    let nasal = before.ends_with(is_nasal);
    match (person, nasal) {
        (2, false) => "ti",
        (2, true) => "di",
        (_, false) => "ka",
        (_, true) => "ga",
    }
}

fn build(verb: &Verb, tense: &Tense, person: usize, plural: bool) -> Option<String> {
    let object = if verb.plural_object { "a" } else { "" };
    let stem = format!("{}{object}", verb.root);
    let word = match tense.form {
        Form::Imperative | Form::Vetative => {
            if person != 1 {
                return None;
            }
            let mut w = stem;
            if plural {
                // exe-a + e → exeaue «¡véanlos!» (the a takes a u before e).
                if verb.plural_object {
                    w.push('u');
                }
                w.push('e');
            }
            return Some(if tense.form == Form::Vetative {
                format!("ásï {w}")
            } else {
                w
            });
        }
        Form::Future => {
            let future = match (&verb.future, verb.plural_object) {
                (Some(f), false) => format!("{f}a"),
                // a (plural object) + a (future) → aua.
                (_, true) => format!("{}aua", verb.root),
                (None, false) => format!("{stem}a"),
            };
            if tense.question {
                future
            } else {
                format!("{future}{}", mode(&future, person))
            }
        }
        Form::Conditional => {
            let base = match (&verb.short, verb.plural_object) {
                (Some(s), false) => s.clone(),
                _ => stem,
            };
            if tense.question {
                format!("{base}pirini")
            } else {
                let pirin = format!("{base}pirin");
                format!("{pirin}{}", mode(&pirin, person))
            }
        }
        aspect => {
            let (marker, question) = match (aspect, tense.past) {
                (Form::Actual, false) => ("xa", "xaki"),
                (Form::Habitual, false) => ("sïn", "sïni"),
                (Form::Perfective, false) => ("s", "ski"),
                (Form::Inceptive, false) => ("xam", "xamki"),
                (Form::Actual, true) => ("xap", "xepi"),
                (Form::Habitual, true) => ("sïamp", "sïambi"),
                (Form::Perfective, true) => ("sp", "spi"),
                _ => ("xanamp", "xanambi"),
            };
            if tense.question {
                format!("{stem}{question}")
            } else {
                let base = format!("{stem}{marker}");
                format!("{base}{}", mode(&base, person))
            }
        }
    };
    let word = if plural {
        format!("{word}{}", PLURAL[person])
    } else {
        word
    };
    Some(if tense.negative {
        format!("no {word}")
    } else {
        word
    })
}

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

fn plain_word(s: &str) -> String {
    s.chars().map(plain).collect()
}

fn is_vowel(c: char) -> bool {
    "aeiïouáéíóú".contains(c)
}

/// Lower-cases and turns typographic apostrophes into '.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            '’' | '‘' | '`' | '´' => '\'',
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

/// The root of a typed verb: the infinitive without -ni (piréni → piré).
fn root_of(word: &str) -> String {
    match word.strip_suffix("ni") {
        Some(root) if root.chars().any(is_vowel) => root.to_string(),
        _ => word.to_string(),
    }
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
        let key = plain_word(&word);
        // An infinitive (piréni) or a bare root (piré), accents optional.
        let entry = self
            .lexicon
            .iter()
            .find(|e| plain_word(&e.entry) == key || plain_word(&e.root) == key);
        let plural_object = opts.kind() == Some("pl");
        let verb = match entry {
            Some(e) => Verb {
                // Accents are optional: if the person wrote them (piréni),
                // keep them in every form.
                root: if word != key {
                    root_of(&word)
                } else {
                    e.root.clone()
                },
                future: e.future.clone(),
                short: e.short.clone(),
                plural_object,
            },
            None => Verb {
                root: root_of(&word),
                future: None,
                short: None,
                plural_object,
            },
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
            serde_json::from_str(include_str!("../../../data/languages/purhepecha.json")).unwrap();
        serde_json::from_value(raw["conjugation"].clone()).unwrap()
    }

    /// The three persons of one tense: (singular, plural) each.
    fn forms(verb: &str, tense: &str, kind: Option<&str>) -> Vec<(String, Option<String>)> {
        let c = data();
        let opts: Options = match kind {
            Some(k) => serde_json::from_value(serde_json::json!({ "kind": k })).unwrap(),
            None => Options::default(),
        };
        let r = c.conjugate_with(verb, &opts).unwrap();
        let t = r.tables.iter().find(|t| t.id == tense).unwrap();
        t.rows
            .iter()
            .map(|r| (r.singular.clone(), r.plural.clone()))
            .collect()
    }

    fn sg(verb: &str, tense: &str) -> Vec<String> {
        forms(verb, tense, None).into_iter().map(|f| f.0).collect()
    }

    #[test]
    fn spreadsheet_paradigm_of_uanda() {
        // Liber verborum, sheet "Purhepecha".
        let cases = [
            ("actual", ["uandaxaka", "uandaxaka", "uandaxati"]),
            ("habitual", ["uandasïnga", "uandasïnga", "uandasïndi"]),
            ("perfective", ["uandaska", "uandaska", "uandasti"]),
            ("inceptive", ["uandaxamga", "uandaxamga", "uandaxamdi"]),
            ("past-actual", ["uandaxapka", "uandaxapka", "uandaxapti"]),
            (
                "past-habitual",
                ["uandasïampka", "uandasïampka", "uandasïampti"],
            ),
            ("past-perfective", ["uandaspka", "uandaspka", "uandaspti"]),
            (
                "past-inceptive",
                ["uandaxanampka", "uandaxanampka", "uandaxanampti"],
            ),
            (
                "conditional",
                ["uandapiringa", "uandapiringa", "uandapirindi"],
            ),
            ("future", ["uandaaka", "uandaaka", "uandaati"]),
        ];
        for (tense, want) in cases {
            assert_eq!(sg("uandani", tense), want, "{tense}");
        }
        let q: Vec<String> = [
            "q-actual",
            "q-habitual",
            "q-perfective",
            "q-past-actual",
            "q-past-habitual",
            "q-past-perfective",
            "q-conditional",
            "q-future",
        ]
        .iter()
        .map(|t| sg("uanda", t)[2].clone())
        .collect();
        assert_eq!(
            q,
            [
                "uandaxaki",
                "uandasïni",
                "uandaski",
                "uandaxepi",
                "uandasïambi",
                "uandaspi",
                "uandapirini",
                "uandaa"
            ]
        );
    }

    #[test]
    fn plural_endings() {
        // Medina Pérez: pirésïngaksï, pireskajtsï, piréstiksï.
        let f = forms("piréni", "habitual", None);
        assert_eq!(f[0].1.as_deref(), Some("pirésïngaksï"));
        assert_eq!(f[1].1.as_deref(), Some("pirésïngajtsï"));
        assert_eq!(f[2].1.as_deref(), Some("pirésïndiksï"));
        let f = forms("pireni", "q-habitual", None);
        assert_eq!(f[0].1.as_deref(), Some("piresïniksï"));
    }

    #[test]
    fn orders() {
        let f = forms("uandani", "imperative", None);
        assert_eq!(f[1], ("uanda".into(), Some("uandae".into())));
        assert_eq!(f[0].0, "");
        let f = forms("t'ireni", "vetative", None);
        assert_eq!(f[1], ("ásï t'ire".into(), Some("ásï t'iree".into())));
    }

    #[test]
    fn negative() {
        assert_eq!(sg("xukani", "neg-past-perfective")[2], "no xukaspti");
    }

    #[test]
    fn plural_object() {
        // Curso, §5.2: exeaxati, exeauaka, exeaue.
        let f = forms("exeni", "actual", Some("pl"));
        assert_eq!(f[2].0, "exeaxati");
        assert_eq!(forms("exeni", "future", Some("pl"))[1].0, "exeauaka");
        assert_eq!(
            forms("exeni", "imperative", Some("pl"))[1].1.as_deref(),
            Some("exeaue")
        );
    }

    #[test]
    fn irregular_future_and_conditional() {
        // Medina Pérez, lecc. 33 y 37: jauaka, jauati; japiringa.
        assert_eq!(sg("jarhani", "future"), ["jauaka", "jauaka", "jauati"]);
        assert_eq!(sg("jarhani", "q-future")[0], "jaua");
        assert_eq!(sg("jarhani", "conditional")[0], "japiringa");
        assert_eq!(sg("nirani", "future")[0], "niuaka");
        // The other tenses are regular.
        assert_eq!(sg("nirani", "habitual")[0], "nirasïnga");
    }

    #[test]
    fn unknown_verb_uses_the_rules() {
        let c = data();
        let r = c.conjugate("tarheni").unwrap();
        assert!(!r.known || r.root == "tarheni");
        let f = forms("ánchikurhini", "habitual", None);
        assert_eq!(f[0].0, "ánchikurhisïnga");
        assert_eq!(sg("sïkua", "actual")[2], "sïkuaxati");
    }

    #[test]
    fn spanish_equivalents() {
        let c = data();
        let r = c.conjugate("piréni").unwrap();
        let es = |id: &str, row: usize| {
            let t = r.tables.iter().find(|t| t.id == id).unwrap();
            (
                t.rows[row].es_singular.clone(),
                t.rows[row].es_plural.clone(),
            )
        };
        assert_eq!(es("actual", 0).0.as_deref(), Some("yo estoy cantando"));
        assert_eq!(
            es("past-actual", 2).0.as_deref(),
            Some("él/ella estaba cantando")
        );
        assert_eq!(es("past-perfective", 0).0.as_deref(), Some("yo canté"));
        assert_eq!(
            es("q-actual", 2).0.as_deref(),
            Some("¿él/ella está cantando?")
        );
        assert_eq!(es("vetative", 1).0.as_deref(), Some("¡no cantes!"));
        assert_eq!(
            es("inceptive", 0).0.as_deref(),
            Some("yo estoy a punto de cantar")
        );
    }

    #[test]
    fn lexicon_round_trip() {
        let c = data();
        for e in &c.lexicon {
            let r = c.conjugate(&e.entry).unwrap();
            assert!(r.known, "{}", e.entry);
            assert_eq!(r.root, e.entry);
            assert_eq!(e.entry, format!("{}ni", e.root), "{}", e.entry);
            assert!(
                r.tables.iter().all(|t| t
                    .rows
                    .iter()
                    .all(|row| { row.singular.is_empty() || row.es_singular.is_some() })),
                "missing Spanish for {}",
                e.entry
            );
        }
    }
}
