//! Conjugation for Wixárika (huichol), written with the orthography of
//! Iturrioz et al. (1999) used by Ramos Bierge (2016): a e i ɨ u (long
//! vowels doubled); h k kw m n p r t ts w x y ' (x is the retroflex [ʐ],
//! ' the glottal stop).
//!
//! The verb follows the template of Ramos Bierge, *Clause Types and
//! Transitivity in Wixárika* (2016, fig. 3.1) and Iturrioz, Gómez &
//! Ramírez de la Cruz, «La morfología verbal», in *Lenguas y literaturas
//! indígenas de Jalisco*:
//!
//! ```text
//! subject  assertive  distributive  prefixes  stem      tense/aspect
//! ne-      pɨ-                      ti-       'uximaya            nepɨti'uximaya «trabajo»
//! me-      pɨ-        te-                     nei       -tɨwe      mepɨteneitɨwe  «están bailando»
//! —        p-                       u-        nua                 punua          «llegó»
//! ```
//!
//! Each verb brings its own prefix chain (ta- / ti- for singular or plural
//! action, u- «visible», e- «lejos», ka- «abajo», au- «todo»…) and its stems:
//! several verbs change root with the number of the subject or the aspect
//! (mie / hu / tɨa / kɨ «ir»).

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// pɨ- + imperfective stem: nepɨti'uximaya «trabajo».
    Present,
    /// + -kai: nepɨti'uximayakai «trabajaba».
    Imperfect,
    /// perfective stem, or + -xɨ: mepɨti'ɨwaxɨ «se bañaron».
    Perfective,
    /// + -ni: nepauküni «dormiré».
    Future,
    /// + -ne (sg.) / -tɨwe (pl.): «estoy bailando».
    Progressive,
    /// + -ne / -tɨwe + -kai: «estaba llorando».
    PastProgressive,
    /// ka- instead of pɨ-: nekati'uximaya «no trabajo».
    NegPresent,
    /// ka- + perfective: «no trabajé».
    NegPerfective,
    /// ka- + -ni: «no trabajaré».
    NegFuture,
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

/// The pieces of one verb, as attested in the sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The verb as shown in the list: "'uximaya".
    pub entry: String,
    /// Prefixes with a singular subject: "ti", "ta", "u", "au" (may be
    /// empty).
    #[serde(default)]
    pub prefix: String,
    /// Prefixes with a plural subject, when different: "ti", "ka".
    #[serde(default)]
    pub prefix_pl: Option<String>,
    /// Imperfective stem with a singular subject: "mie".
    pub stem: String,
    /// Imperfective stem with a plural subject: "hu".
    #[serde(default)]
    pub stem_pl: Option<String>,
    /// Perfective stem (no -xɨ): "tɨa". Missing: stem + -xɨ.
    #[serde(default)]
    pub perfective: Option<String>,
    #[serde(default)]
    pub perfective_pl: Option<String>,
    /// Prefixes in the perfective, when different: "e" (petɨa «se fue»).
    #[serde(default)]
    pub perfective_prefix: Option<String>,
    #[serde(default)]
    pub perfective_prefix_pl: Option<String>,
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
    "aeiuɨáéíúɨ́".contains(c)
}

/// Subject prefixes: ne- pe- Ø, te- xe- me-.
const SUBJECT: [&str; 3] = ["ne", "pe", ""];
const SUBJECT_PL: [&str; 3] = ["te", "xe", "me"];

/// The assertive pɨ- loses its vowel before a vowel: pɨ + u-nua → punua.
fn assertive(next: &str) -> String {
    if next.starts_with(is_vowel) {
        format!("p{next}")
    } else {
        format!("pɨ{next}")
    }
}

impl Entry {
    fn stem(&self, plural: bool, perfective: bool) -> (String, bool) {
        // (stem, needs -xɨ)
        let pfv = if plural {
            self.perfective_pl
                .as_ref()
                .or(self.perfective.as_ref().filter(|_| self.stem_pl.is_none()))
        } else {
            self.perfective.as_ref()
        };
        match (perfective, pfv) {
            (true, Some(p)) => (p.clone(), false),
            (true, None) => (self.imperfective(plural), true),
            (false, _) => (self.imperfective(plural), false),
        }
    }

    fn imperfective(&self, plural: bool) -> String {
        match (plural, &self.stem_pl) {
            (true, Some(s)) => s.clone(),
            _ => self.stem.clone(),
        }
    }

    fn prefix(&self, plural: bool, perfective: bool) -> String {
        let pfv = if plural {
            self.perfective_prefix_pl
                .as_ref()
                .or(self.perfective_prefix.as_ref())
        } else {
            self.perfective_prefix.as_ref()
        };
        match (perfective, pfv) {
            (true, Some(p)) => p.clone(),
            _ if plural => self
                .prefix_pl
                .clone()
                .unwrap_or_else(|| self.prefix.clone()),
            _ => self.prefix.clone(),
        }
    }
}

fn build(verb: &Entry, form: Form, person: usize, plural: bool) -> String {
    let perfective = matches!(form, Form::Perfective | Form::NegPerfective);
    let negative = matches!(
        form,
        Form::NegPresent | Form::NegPerfective | Form::NegFuture
    );
    let (stem, xi) = verb.stem(plural, perfective);
    let mut prefix = verb.prefix(plural, perfective).replace('-', "");
    // A plural animate subject takes the distributive te- when the verb has
    // no other prefix: tetenei «bailamos», mete'uximaya «trabajan».
    if plural && prefix.is_empty() && person != 1 {
        prefix = "te".into();
    }
    let suffix = match form {
        Form::Present | Form::NegPresent => "",
        Form::Imperfect => "kai",
        Form::Perfective | Form::NegPerfective => {
            if xi {
                "xɨ"
            } else {
                ""
            }
        }
        Form::Future | Form::NegFuture => "ni",
        Form::Progressive => {
            if plural {
                "tɨwe"
            } else {
                "ne"
            }
        }
        Form::PastProgressive => {
            if plural {
                "tɨwekai"
            } else {
                "nekai"
            }
        }
    };
    let body = format!("{prefix}{stem}{suffix}");
    let subject = if plural {
        SUBJECT_PL[person]
    } else {
        SUBJECT[person]
    };
    // Negation ka- takes the place of the assertive: nekati'uximaya.
    let modal = if negative {
        format!("ka{body}")
    } else {
        assertive(&body)
    };
    format!("{subject}{modal}")
}

fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            '’' | '‘' | 'ʼ' | '´' | '`' | 'ʔ' => '\'',
            '+' | 'ï' => 'ɨ',
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
        let entry = self
            .lexicon
            .iter()
            .find(|e| e.entry == word || e.stem == word);
        // A verb that isn't in the list: the stem as typed, with the
        // prefix chosen in «Conjuga tu verbo» (ti- by default).
        let verb = match entry {
            Some(e) => e.clone(),
            None => Entry {
                entry: word.clone(),
                prefix: opts.kind().unwrap_or("ti").to_string(),
                prefix_pl: None,
                stem: word.clone(),
                stem_pl: None,
                perfective: None,
                perfective_pl: None,
                perfective_prefix: None,
                perfective_prefix_pl: None,
                meaning: String::new(),
                es: None,
                note: None,
            },
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
                    .map(|(i, person)| Row {
                        person: person.label.clone(),
                        clitic: String::new(),
                        es_singular: es(tense, person, false),
                        es_plural: es(tense, person, true),
                        singular: build(&verb, tense.form, i, false),
                        plural: Some(build(&verb, tense.form, i, true)),
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
            serde_json::from_str(include_str!("../../../data/languages/wixarika.json")).unwrap();
        serde_json::from_value(raw["conjugation"].clone()).unwrap()
    }

    fn form(verb: &str, f: Form, person: usize, plural: bool) -> String {
        let c = data();
        let e = c.lexicon.iter().find(|e| e.entry == verb).unwrap();
        build(e, f, person, plural)
    }

    // Forms attested in Ramos Bierge (2016) and Iturrioz et al., written
    // as one word.
    #[test]
    fn attested_forms() {
        // (209a) ne-p-ti-'uximaya «yo trabajo» (here with the full pɨ-).
        assert_eq!(form("'uximaya", Form::Present, 0, false), "nepɨti'uximaya");
        // (209b) te-te-'uximaya «nosotros trabajamos».
        assert_eq!(form("'uximaya", Form::Present, 0, true), "tepɨte'uximaya");
        // (195) ne-pu-nua «llegué»; (26) p-u-kwai «se comió».
        assert_eq!(form("nua", Form::Perfective, 0, false), "nepunua");
        assert_eq!(form("nua", Form::Perfective, 2, false), "punua");
        // (211) me-pu-ti-'ɨwa-xɨ «ellos se bañaron».
        assert_eq!(form("'ɨwa", Form::Perfective, 2, true), "mepɨti'ɨwaxɨ");
        // (220a) ne-p-au-ku-ni «dormiré».
        assert_eq!(form("ku", Form::Future, 0, false), "nepaukuni");
        // (220b) me-mu-ka-kukutsu-kai «ellos estaban durmiendo» (with pɨ-).
        assert_eq!(form("ku", Form::Imperfect, 2, true), "mepɨkakukutsukai");
        // (208a) te-te-nei-tɨwe «estamos bailando».
        assert_eq!(form("nei", Form::Progressive, 0, true), "tepɨteneitɨwe");
        // (218a) waika-ne-kai «estaba jugando».
        assert_eq!(
            form("waika", Form::PastProgressive, 2, false),
            "pɨtiwaikanekai"
        );
        // (168)–(171): mie / hu (ir), tɨa / kɨ (fue, fueron).
        assert_eq!(form("mie", Form::Present, 0, false), "nepɨmie");
        assert_eq!(form("mie", Form::Present, 2, true), "mepɨtihu");
        assert_eq!(form("mie", Form::Perfective, 2, false), "petɨa");
        assert_eq!(form("mie", Form::Perfective, 0, true), "tepeyekɨ");
        // (196) ne-u-ka-ye-mie «no voy»: negation ka- (here without u-).
        assert_eq!(
            form("'uximaya", Form::NegPresent, 0, false),
            "nekati'uximaya"
        );
        // (207b) ta-kwika «canta» (sg.), (210) ti-kwika (pl.).
        assert_eq!(form("kwika", Form::Present, 2, false), "pɨtakwika");
        assert_eq!(form("kwika", Form::Present, 2, true), "mepɨtikwika");
    }

    #[test]
    fn lexicon_round_trip() {
        let c = data();
        for e in &c.lexicon {
            let r = c.conjugate(&e.entry).unwrap();
            assert!(r.known, "{}", e.entry);
            for t in &r.tables {
                for row in &t.rows {
                    assert!(row.es_singular.is_some(), "{} {}", e.entry, t.id);
                }
            }
        }
    }
}
