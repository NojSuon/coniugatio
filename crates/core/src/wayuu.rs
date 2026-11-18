//! Conjugation for Wayuunaiki, the language of the Wayuu of the Guajira
//! Peninsula (Colombia and Venezuela), after José Álvarez, *Compendio de
//! la gramática de la lengua wayuu* (2017) and *Manual de la lengua
//! wayuu*.
//!
//! The subjective construction: the verb agrees with the subject in gender
//! and number, not in person (the person is said with a pronoun: taya,
//! pia, nia…).
//!
//! ```text
//! infinitive   stem        mutated   present-past (m, f, pl)
//! apütaa       apüta-      apütü-    apütüshi  apütüsü  apütüshii   1st conjugation
//! achajawaa    achajaa-    achajaa-  achajaashi achajaasü achajaashii 2nd (-waa)
//! ```
//!
//! The 1st conjugation cuts the final long vowel (apütaa → apüta) and most
//! tenses change a final a / o to ü / u (vowel mutation: apütü-, o'unu-).
//! The 2nd conjugation drops -waa and lengthens the vowel (achaja → achajaa).
//! Each tense takes one of several sets of gender-number endings: -shi /
//! -sü / -shii, -chi / -rü / -na, -chi / -tü / -na, -i / -lü / -lii.

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// apütüshi «deja, dejó».
    PresentPast,
    /// apütayüi «dejó hace un rato».
    RecentPast,
    /// apütayüiya «todavía deja».
    CurrentPast,
    /// apütüichipa «acaba de dejar».
    ImmediatePast,
    /// apütatüjüiya «ya antes había dejado».
    AnteriorPast,
    /// apütüshima'a «había dejado hace mucho».
    ExtendedPast,
    /// apütüshinka «había dejado».
    RemotePast,
    /// apütamuyuu «solía dejar».
    Frequentative,
    /// apütüichi «ya deja, empieza a dejar».
    ImminentFuture,
    /// apüteechi «dejará».
    Future,
    /// apütüinjachi «va a dejar».
    DeterminedFuture,
    /// apüteeshi «quiere dejar».
    Desiderative,
    /// nnojoishi apütüin «no deja».
    Negative,
    /// nnojoleechi apütüin «no dejará».
    NegativeFuture,
    /// püpüta «¡deja!», japüta «¡dejen!».
    Imperative,
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
    /// The infinitive: "apütaa", "achajawaa", "talataa".
    pub entry: String,
    pub meaning: String,
    #[serde(default)]
    pub es: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    /// Stative verbs (no prefix, no imperative with pü-): talataa, makataa.
    #[serde(default)]
    pub stative: bool,
    /// 1 or 2; detected from the ending (-waa is the 2nd).
    #[serde(default)]
    pub conjugation: Option<u8>,
    /// Overrides the vowel mutation (malaa → malasü, not *malüsü).
    #[serde(default)]
    pub mutation: Option<bool>,
    /// The imperative when the rule doesn't apply; "" for none.
    #[serde(default)]
    pub imperative: Option<[String; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conjugation {
    pub persons: Vec<Person>,
    pub tenses: Vec<Tense>,
    #[serde(default)]
    pub lexicon: Vec<Entry>,
}

fn is_vowel(c: char) -> bool {
    "aeiouü".contains(c)
}

/// The verb's stems.
struct Verb {
    /// apüta-, achajaa- (no mutation).
    plain: String,
    /// apütü-, achajaa- (with mutation).
    mutated: String,
    /// The 2nd conjugation (stem in a long vowel).
    second: bool,
    stative: bool,
    imperative: Option<[String; 2]>,
}

/// The mutated stem: a → ü (u after u), o → u.
fn mutate(stem: &str) -> String {
    let mut chars: Vec<char> = stem.chars().collect();
    let Some(&last) = chars.last() else { return stem.to_string() };
    let prev = chars[..chars.len() - 1].iter().rev().find(|c| is_vowel(**c)).copied();
    let new = match last {
        'a' if prev == Some('u') => 'u',
        'a' => 'ü',
        'o' => 'u',
        _ => return stem.to_string(),
    };
    *chars.last_mut().unwrap() = new;
    chars.into_iter().collect()
}

impl Verb {
    fn new(infinitive: &str, entry: Option<&Entry>, stative_hint: Option<bool>) -> Option<Verb> {
        let stative = entry.map(|e| e.stative).or(stative_hint).unwrap_or_else(|| !infinitive.starts_with(['a', 'e', 'o']));
        let second = match entry.and_then(|e| e.conjugation) {
            Some(n) => n == 2,
            None => infinitive.ends_with("waa") && infinitive.chars().count() > 4,
        };
        let plain = if second {
            let base = infinitive.strip_suffix("waa")?;
            let chars: Vec<char> = base.chars().collect();
            let n = chars.len();
            let last = *chars.last()?;
            if !is_vowel(last) || (n >= 2 && chars[n - 2] == last) {
                base.to_string()
            } else {
                format!("{base}{last}")
            }
        } else {
            let chars: Vec<char> = infinitive.chars().collect();
            let n = chars.len();
            // A long vowel at the end: apütaa, makataa, kamanee.
            if n < 3 || chars[n - 1] != chars[n - 2] || !is_vowel(chars[n - 1]) {
                return None;
            }
            chars[..n - 1].iter().collect()
        };
        let mutation = entry.and_then(|e| e.mutation).unwrap_or_else(|| {
            // Actives mutate; statives only with the thematic suffix -ta.
            !second && (!stative || plain.ends_with("ta") || plain.ends_with("to"))
        });
        let mutated = if mutation && !second { mutate(&plain) } else { plain.clone() };
        Some(Verb { plain, mutated, second, stative, imperative: entry.and_then(|e| e.imperative.clone()) })
    }

    /// The stem before -ee (future, desiderative): apüt-ee, achajaa-jee.
    fn before_ee(&self) -> String {
        if self.second {
            format!("{}j", self.plain)
        } else {
            let mut s = self.plain.clone();
            s.pop();
            s
        }
    }
}

/// Adds an ending to the mutated stem. Before -sh / -s, a final lü, rü,
/// lu, ru loses its vowel and l, r become t (emiralü-shi → emiratshi).
fn attach(stem: &str, ending: &str) -> String {
    if ending.starts_with('s') {
        for (tail, new) in [("lü", "t"), ("rü", "t"), ("lu", "t"), ("ru", "t")] {
            if let Some(root) = stem.strip_suffix(tail) {
                return format!("{root}{new}{ending}");
            }
        }
    }
    format!("{stem}{ending}")
}

/// Gender-number endings: masculine, feminine, plural.
const A: [&str; 3] = ["shi", "sü", "shii"];
const B: [&str; 3] = ["chi", "rü", "na"];
const C: [&str; 3] = ["chi", "tü", "na"];
const E: [&str; 3] = ["i", "lü", "lii"];

/// The 2nd-person prefixes for the imperative, by vowel harmony with the
/// root: pü- / ja-, pi- / je- (e, i), pu- (w), pa- / ja- (j, ' + a)…
fn imperative_prefixes(root: &str) -> (String, String) {
    let first_vowel = root.chars().find(|c| is_vowel(*c)).unwrap_or('a');
    if root.starts_with(['j', '\'']) && "aeo".contains(first_vowel) {
        return (format!("p{first_vowel}"), format!("j{first_vowel}"));
    }
    if root.starts_with('w') {
        return ("pu".into(), "ja".into());
    }
    if root.starts_with('y') {
        return ("pi".into(), "ja".into());
    }
    let coronal = ["t", "s", "n", "l", "r", "ch", "sh", "y"].iter().any(|c| root.starts_with(c));
    if !coronal && "ei".contains(first_vowel) {
        return ("pi".into(), "je".into());
    }
    ("pü".into(), "ja".into())
}

/// Gender-number slot: 0 masculine, 1 feminine, 2 plural.
fn build(v: &Verb, form: Form, g: usize) -> Option<String> {
    let s = &v.mutated;
    let n = &v.plain;
    Some(match form {
        Form::PresentPast => attach(s, A[g]),
        Form::RecentPast => format!("{n}yü{}", E[g]),
        Form::CurrentPast => format!("{n}yü{}ya", E[g]),
        Form::ImmediatePast => format!("{s}{}", ["ichipa", "itpa", "inapa"][g]),
        Form::AnteriorPast => format!("{n}tüjü{}ya", E[g]),
        Form::ExtendedPast => format!("{}ma'a", attach(s, A[g])),
        Form::RemotePast => attach(s, ["shinka", "süinka", "shiinka"][g]),
        Form::Frequentative => format!("{n}muyuu"),
        Form::ImminentFuture => format!("{s}i{}", B[g]),
        Form::Future => format!("{}ee{}", v.before_ee(), B[g]),
        Form::DeterminedFuture => format!("{s}inja{}", C[g]),
        Form::Desiderative if v.stative => return None,
        Form::Desiderative => format!("{}ee{}", v.before_ee(), A[g]),
        Form::Negative => format!("{} {s}in", ["nnojoishi", "nnojotsü", "nnojoishii"][g]),
        Form::NegativeFuture => format!("{} {s}in", ["nnojoleechi", "nnojoleerü", "nnojoleena"][g]),
        // Only one row: «tú» in the singular column, «ustedes» in the plural.
        Form::Imperative => {
            if let Some(imp) = &v.imperative {
                let f = imp[if g == 0 { 0 } else { 1 }].clone();
                return (!f.is_empty()).then_some(f);
            }
            if v.stative {
                return Some(format!("{n} {}", if g == 0 { "pia" } else { "jaya" }));
            }
            // The impersonal a- / e- / o- gives way to the person.
            let root: String = n.chars().skip(1).collect();
            let (sg, pl) = imperative_prefixes(&root);
            format!("{}{root}", if g == 0 { sg } else { pl })
        }
    })
}

fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb.trim().to_lowercase().replace(['’', 'ʼ', '´', '`', '‘'], "'");
    // ü typed as u + combining diaeresis.
    let w = w.replace("u\u{308}", "ü");
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_alphabetic() || c == '\'') || w.chars().count() > 40 || !w.chars().any(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

/// «él/ella» as «él» or «ella», by the row's gender.
fn gendered(text: String, feminine: bool) -> String {
    let pairs: [(&str, &str, &str); 3] = [("él/ella", "él", "ella"), ("he/she", "he", "she"), ("他/她", "他", "她")];
    pairs.iter().fold(text, |t, (both, m, f)| t.replace(both, if feminine { f } else { m }))
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.lexicon.iter().find(|e| e.entry == word);
        let hint = match opts.kind.as_deref() {
            Some("activo") => Some(false),
            Some("estativo") => Some(true),
            _ => None,
        };
        let verb = Verb::new(&word, entry, hint).ok_or(Error::InvalidVerb)?;

        let es_meaning = opts.es().map(str::to_string).or_else(|| entry.and_then(|e| e.es.clone()));
        let es = |tense: &Tense, p: usize, feminine: bool| -> Option<String> {
            let meaning = es_meaning.as_deref()?;
            crate::gloss(opts, meaning, tense.es.as_ref()?, p, tense.negative).map(|t| gendered(t, feminine))
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
                        let feminine = person.id == "f";
                        let imperative = tense.form == Form::Imperative;
                        let (sg, pl) = match (imperative, feminine) {
                            (true, true) => (None, None),
                            (true, false) => (build(&verb, tense.form, 0), build(&verb, tense.form, 2)),
                            (false, true) => (build(&verb, tense.form, 1), None),
                            (false, false) => (build(&verb, tense.form, 0), build(&verb, tense.form, 2)),
                        };
                        // Orders are for «tú» and «ustedes»; the rest for él / ella / ellos.
                        let (p_sg, p_pl) = if imperative { (1, 4) } else { (2, 5) };
                        Row {
                            person: person.label.clone(),
                            clitic: String::new(),
                            es_singular: sg.as_ref().and_then(|_| es(tense, p_sg, feminine)),
                            es_plural: pl.as_ref().and_then(|_| es(tense, p_pl, false)),
                            singular: sg.unwrap_or_default(),
                            plural: pl,
                        }
                    })
                    .collect(),
            })
            .collect();

        Ok(Conjugated {
            root: word.clone(),
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

    fn three(inf: &str, form: Form) -> Vec<String> {
        let v = Verb::new(inf, None, None).unwrap();
        (0..3).map(|g| build(&v, form, g).unwrap_or_default()).collect()
    }

    // Álvarez (2017), tables for apütaa «dejar» and achajawaa «buscar».
    #[test]
    fn dejar() {
        assert_eq!(three("apütaa", Form::PresentPast), ["apütüshi", "apütüsü", "apütüshii"]);
        assert_eq!(three("apütaa", Form::ImminentFuture), ["apütüichi", "apütüirü", "apütüina"]);
        assert_eq!(three("apütaa", Form::Future), ["apüteechi", "apüteerü", "apüteena"]);
        assert_eq!(three("apütaa", Form::DeterminedFuture), ["apütüinjachi", "apütüinjatü", "apütüinjana"]);
        assert_eq!(three("apütaa", Form::ImmediatePast), ["apütüichipa", "apütüitpa", "apütüinapa"]);
        assert_eq!(three("apütaa", Form::RecentPast), ["apütayüi", "apütayülü", "apütayülii"]);
        assert_eq!(three("apütaa", Form::CurrentPast), ["apütayüiya", "apütayülüya", "apütayüliiya"]);
        assert_eq!(three("apütaa", Form::AnteriorPast), ["apütatüjüiya", "apütatüjülüya", "apütatüjüliiya"]);
        assert_eq!(three("apütaa", Form::ExtendedPast)[..2], ["apütüshima'a", "apütüsüma'a"]);
        assert_eq!(three("apütaa", Form::RemotePast), ["apütüshinka", "apütüsüinka", "apütüshiinka"]);
        assert_eq!(three("apütaa", Form::Frequentative)[0], "apütamuyuu");
        assert_eq!(three("apütaa", Form::Desiderative)[0], "apüteeshi");
        assert_eq!(three("apütaa", Form::Imperative)[0], "püpüta");
        assert_eq!(three("apütaa", Form::Imperative)[2], "japüta");
    }

    #[test]
    fn buscar() {
        assert_eq!(three("achajawaa", Form::PresentPast), ["achajaashi", "achajaasü", "achajaashii"]);
        assert_eq!(three("achajawaa", Form::Future), ["achajaajeechi", "achajaajeerü", "achajaajeena"]);
        assert_eq!(three("achajawaa", Form::DeterminedFuture)[0], "achajaainjachi");
        assert_eq!(three("achajawaa", Form::RecentPast)[2], "achajaayülii");
        assert_eq!(three("achajawaa", Form::AnteriorPast)[0], "achajaatüjüiya");
        assert_eq!(three("achajawaa", Form::Desiderative)[0], "achajaajeeshi");
        assert_eq!(three("achajawaa", Form::Imperative)[0], "püchajaa");
        assert_eq!(three("achajawaa", Form::Imperative)[2], "jachajaa");
        assert_eq!(three("a'lakajawaa", Form::PresentPast)[0], "a'lakajaashi");
    }

    #[test]
    fn mutation_and_statives() {
        assert_eq!(three("ayonnajaa", Form::PresentPast)[0], "ayonnajüshi");
        // u after u: atunkushi, o'unushi.
        assert_eq!(three("atunkaa", Form::PresentPast)[0], "atunkushi");
        assert_eq!(three("o'unaa", Form::PresentPast)[1], "o'unusü");
        // l / r → t before -sh: emiratshi.
        assert_eq!(three("emiralaa", Form::PresentPast)[0], "emiratshi");
        // Statives: talatüshi, makatüshi; imperative with a pronoun.
        assert_eq!(three("talataa", Form::PresentPast)[0], "talatüshi");
        assert_eq!(three("jimataa", Form::Imperative)[0], "jimata pia");
        assert_eq!(three("kamanewaa", Form::Imperative)[2], "kamanee jaya");
        // Negation: nnojoishi a'yataain, nnojoleechi a'yataain.
        assert_eq!(three("a'yatawaa", Form::Negative)[0], "nnojoishi a'yataain");
        assert_eq!(three("a'yatawaa", Form::NegativeFuture)[0], "nnojoleechi a'yataain");
        assert_eq!(three("o'unaa", Form::Desiderative)[0], "o'uneeshi");
        assert_eq!(three("waraitaa", Form::Negative)[0], "nnojoishi waraitüin");
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/wayuu.json")).unwrap();
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
        let r = c.conjugate("talataa").unwrap();
        assert!(r.tables[0].rows[1].es_singular.as_deref().unwrap().starts_with("ella"));
    }
}
