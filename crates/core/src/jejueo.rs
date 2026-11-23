//! Conjugation for Jejueo (제주어), the language of Jeju Island, after
//! Changyong Yang, Sejung Yang and William O'Grady, *Jejueo: The Language of
//! Korea's Jeju Island* (University of Hawai'i Press, 2020), ch. 2, 5–7.
//!
//! Jejueo is written in Hangeul; forms here are in the book's Romanization,
//! which maps one to one onto Hangeul letters (aw = ㆍ, the «alae a»), with a
//! hyphen between morphemes, as the book writes them: meog-eoms-jeo.
//!
//! The verb doesn't change for person. After the root come up to four
//! time-related suffixes and a sentence ender, which says whether the
//! sentence is a statement or a question and how formal it is:
//!
//! ```text
//! root  aspect  aspect  modality  tense   (honorific) ender
//! meog  -eoms   -eos    -euk      -eun    -u          -da
//!       CONT    PFV     -(eu)neu  -eon    -eub        -jeo, -ga…
//!                       PROSP     -eul(geo)
//! ```
//!
//! So a table has one row per level of speech (informal, polite with the
//! addressee honorific -u, formal with -eub) and two columns, usually a
//! statement and a question.
//!
//! Suffixes come in two kinds (§2.4.4–2.4.7):
//!
//! * eo-suffixes (-eoms, -eos, -eon) harmonize: -ams, -as, -an after a root
//!   whose vowel is a, o or (with a consonant after it) aw; they shrink to
//!   -ms, -s, -n after a and eo, take y after e, ae and aw (haw-yeoms), and
//!   join an i into y (dekky-eoms).
//! * eu-suffixes (-eun, -euk, -eul, -eub, -eula) start with eu after most
//!   consonants, i after s, j and ch, u after b, p and m, and nothing after a
//!   vowel or l; a root-final l drops before n, b and the future l
//!   (menggeu-n-da, menggeu-lgeo).

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// meog-eun-da «come», meog-neu-n-ga «¿come?»; meog-eub-ne-da.
    Present,
    /// meog-eoms-jeo «está comiendo», meog-eom-ga, meog-eoms-u-da.
    Continuative,
    /// meog-eos-jeo «comió», meog-eos-ya, meog-eos-u-da, meog-eos-ib-de-da.
    Perfective,
    /// meog-eon «comió (lo vi)», meog-eon-ga.
    Past,
    /// meog-euk-yeo «voy a comer», meog-euk-u-da.
    Prospective,
    /// meog-eulgeo-yeo «comerá», meog-eulgeo-u-da.
    Future,
    /// meog-eula «¡come!», meog-ge «¡comamos!», meog-eub-seo «coma usted».
    Imperative,
    /// meog-ji mal-la «¡no comas!».
    Prohibitive,
    /// an meog-eos-jeo «no comió».
    NegativePerfective,
    /// an meog-euk-yeo «no voy a comer».
    NegativeProspective,
    /// mos meog-eos-jeo «no pudo comer».
    CannotPerfective,
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
    /// The equivalent of the first column (usually the statement).
    #[serde(default)]
    pub es: Option<spanish::Es>,
    /// The equivalent of the second column (usually the question).
    #[serde(default)]
    pub es_2: Option<spanish::Es>,
    /// Spanish person (0–5) for the first column's equivalent: él.
    #[serde(default = "third")]
    pub person: usize,
    /// Spanish person for the second column's equivalent: tú.
    #[serde(default = "second")]
    pub person_2: usize,
    /// The equivalent of the formal row's first column, when it differs
    /// (meog-eub-seo «coma», a request to an elder, said with 3rd person).
    #[serde(default)]
    pub es_formal: Option<spanish::Es>,
    /// Person for `es_formal`: usted (2) in Spanish, «you» (1) in English.
    #[serde(default = "third")]
    pub person_formal: usize,
}

fn third() -> usize {
    2
}

fn second() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The dictionary form, root + -da: "meog-da", "haw-da".
    pub entry: String,
    /// The root, when it isn't the entry without -da.
    #[serde(default)]
    pub root: Option<String>,
    pub meaning: String,
    #[serde(default)]
    pub es: Option<String>,
    /// "a" or "eo": the vowel of -eoms / -eos / -eon when the root doesn't
    /// follow the rule (menggeul-as, geuchy-as, deul-eos).
    #[serde(default)]
    pub harmony: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conjugation {
    /// The rows: "informal", "polite", "formal".
    pub persons: Vec<Person>,
    pub tenses: Vec<Tense>,
    #[serde(default)]
    pub lexicon: Vec<Entry>,
}

/// How a root ends, for choosing allomorphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    /// a, eo: -ms, -s, -n.
    Open,
    /// e, ae, aw: y before eo (gwe-yeoms, haw-yeoms).
    Y,
    /// i: joins into y (dekky-eoms).
    I,
    /// eu: drops before eo (singg-eoms).
    Eu,
    /// o, u: the eo-suffix is added whole (ju-eos).
    Round,
    /// l: menggeul-ams, menggeu-n-da.
    L,
    /// s, j, ch: i before eu-suffixes (jus-in-da).
    Sibilant,
    /// b, p, m: u before eu-suffixes (sim-un-da).
    Labial,
    /// Other consonants: g, k, d, t, n, ng, h (meog-eun-da).
    Consonant,
}

fn end(root: &str) -> End {
    if root.ends_with("aw") || root.ends_with("ae") || root.ends_with('e') && !root.ends_with("eu") {
        return End::Y;
    }
    if root.ends_with("eo") || root.ends_with('a') {
        return End::Open;
    }
    if root.ends_with("eu") {
        return End::Eu;
    }
    if root.ends_with("ch") {
        return End::Sibilant;
    }
    match root.chars().last() {
        Some('i') => End::I,
        Some('o' | 'u') => End::Round,
        Some('l') => End::L,
        Some('s' | 'j') => End::Sibilant,
        Some('b' | 'p' | 'm') => End::Labial,
        _ => End::Consonant,
    }
}

/// The vowel of an eo-suffix by the general rule (§4.2.4): a after a root
/// whose last vowel is a or o, or aw followed by a consonant; eo otherwise.
fn harmony(root: &str) -> &'static str {
    // A final eu drops (singg-eoms), so the vowel before it decides.
    let r = root.strip_suffix("eu").unwrap_or(root);
    let vowels = ["aw", "ae", "eo", "eu", "a", "e", "i", "o", "u"];
    let mut i = r.len();
    while i > 0 {
        let s = &r[..i];
        if let Some(v) = vowels.iter().find(|v| s.ends_with(*v)) {
            let consonant_after = i < r.len();
            return match *v {
                "a" | "o" => "a",
                "aw" if consonant_after => "a",
                _ => "eo",
            };
        }
        i -= 1;
    }
    "eo"
}

/// Root + an eo-suffix ("eoms", "eom", "eos", "eon"), with a hyphen:
/// meog-eoms, chawj-ams, ga-ms, haw-yeoms, dekky-eoms, singg-eoms.
fn eo(root: &str, vowel: &str, suffix: &str) -> String {
    let rest = suffix.strip_prefix("eo").unwrap_or(suffix);
    match end(root) {
        End::Open => format!("{root}-{rest}"),
        End::Y => format!("{root}-yeo{rest}"),
        End::I => format!("{}y-eo{rest}", &root[..root.len() - 1]),
        End::Eu => format!("{}-{vowel}{rest}", &root[..root.len() - 2]),
        _ => format!("{root}-{vowel}{rest}"),
    }
}

/// Root + an eu-suffix ("eun", "euk", "eulgeo", "eub", "eula"), with a
/// hyphen. `drop_l`: a final l falls before this suffix (menggeu-n-da), as
/// it does before n, b and the future l; it stays before -k and -la.
fn eu(root: &str, suffix: &str, drop_l: bool) -> String {
    let rest = suffix.strip_prefix("eu").unwrap_or(suffix);
    match end(root) {
        End::Sibilant => format!("{root}-i{rest}"),
        End::Labial => format!("{root}-u{rest}"),
        End::Consonant => format!("{root}-eu{rest}"),
        End::L if drop_l => format!("{}-{rest}", &root[..root.len() - 1]),
        _ => format!("{root}-{rest}"),
    }
}

/// Root + -(eu)neu-n, the indicative nonpast: meog-neu-n, menggeu-neu-n.
fn neu(root: &str) -> String {
    match end(root) {
        End::L => format!("{}-neu-n", &root[..root.len() - 1]),
        _ => format!("{root}-neu-n"),
    }
}

/// [informal, polite, formal] × [first column, second column].
type Grid = [[Option<String>; 2]; 3];

fn grid(rows: [[Option<String>; 2]; 3]) -> Grid {
    rows
}

fn s(text: String) -> Option<String> {
    Some(text)
}

fn build(root: &str, vowel: &str, form: Form) -> Grid {
    let cont = eo(root, vowel, "eoms");
    let pfv = eo(root, vowel, "eos");
    let prosp = eu(root, "euk", false);
    let not = |prefix: &str, g: Grid| -> Grid { g.map(|row| row.map(|c| c.map(|c| format!("{prefix} {c}")))) };
    match form {
        Form::Present => grid([
            [s(format!("{}-da", eu(root, "eun", true))), s(format!("{}-ga", neu(root)))],
            [None, None],
            [s(format!("{}-ne-da", eu(root, "eub", true))), s(format!("{}-ne-kka", eu(root, "eub", true)))],
        ]),
        Form::Continuative => grid([
            [s(format!("{cont}-jeo")), s(format!("{}-ga", eo(root, vowel, "eom")))],
            [s(format!("{cont}-u-da")), s(format!("{cont}-u-gwa"))],
            [s(format!("{cont}-ib-ne-da")), s(format!("{cont}-ib-ne-kka"))],
        ]),
        Form::Perfective => grid([
            [s(format!("{pfv}-jeo")), s(format!("{pfv}-ya"))],
            [s(format!("{pfv}-u-da")), s(format!("{pfv}-u-gwa"))],
            [s(format!("{pfv}-ib-de-da")), s(format!("{pfv}-ib-de-ga"))],
        ]),
        Form::Past => {
            let past = eo(root, vowel, "eon");
            grid([[s(past.clone()), s(format!("{past}-ga"))], [None, None], [None, None]])
        }
        Form::Prospective => grid([
            [s(format!("{prosp}-yeo")), s(format!("{prosp}-eul-a"))],
            [s(format!("{prosp}-u-da")), s(format!("{prosp}-u-gwa"))],
            [None, None],
        ]),
        Form::Future => {
            let fut = eu(root, "eulgeo", true);
            grid([
                [s(format!("{fut}-yeo")), s(format!("{fut}-ga"))],
                [s(format!("{fut}-u-da")), s(format!("{fut}-u-kkwa"))],
                [None, None],
            ])
        }
        Form::Imperative => grid([
            [s(eu(root, "eula", false)), s(format!("{root}-ge"))],
            [None, None],
            [s(format!("{}-seo", eu(root, "eub", true))), None],
        ]),
        Form::Prohibitive => grid([[s(format!("{root}-ji mal-la")), None], [None, None], [None, None]]),
        Form::NegativePerfective => not("an", build(root, vowel, Form::Perfective)),
        Form::NegativeProspective => not("an", build(root, vowel, Form::Prospective)),
        Form::CannotPerfective => not("mos", build(root, vowel, Form::Perfective)),
    }
}

/// Lowercase letters only; hyphens and the book's syllable dots go.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = verb.trim().to_lowercase().chars().filter(|c| !matches!(c, '-' | '.' | ' ')).collect();
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_ascii_lowercase()) || w.len() > 30 || !w.chars().any(|c| "aeiou".contains(c)) {
        return Err(Error::InvalidVerb);
    }
    Ok(w)
}

/// The root of a typed word: meogda → meog; a bare root stays as it is.
fn root_of(word: &str) -> std::result::Result<String, Error> {
    let root = match word.strip_suffix("da") {
        Some(r) if r.chars().any(|c| "aeiou".contains(c)) => r,
        _ => word,
    };
    if root.is_empty() {
        return Err(Error::InvalidVerb);
    }
    Ok(root.to_string())
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    fn entry_root(e: &Entry) -> Option<String> {
        match &e.root {
            Some(r) => Some(r.clone()),
            None => normalize(&e.entry).ok().and_then(|w| root_of(&w).ok()),
        }
    }

    /// The entry for a typed word: the dictionary form (meog-da, meogda) or
    /// the root (meog).
    fn find(&self, word: &str) -> Option<&Entry> {
        self.lexicon
            .iter()
            .find(|e| normalize(&e.entry).as_deref() == Ok(word))
            .or_else(|| self.lexicon.iter().find(|e| Self::entry_root(e).as_deref() == Some(word)))
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.find(&word);
        let root = match entry {
            Some(e) => Self::entry_root(e).ok_or(Error::InvalidVerb)?,
            None => root_of(&word)?,
        };
        let vowel = match entry.and_then(|e| e.harmony.as_deref()) {
            Some("a") => "a",
            Some("eo") => "eo",
            _ => harmony(&root),
        };

        let es_meaning = opts.es().map(str::to_string).or_else(|| entry.and_then(|e| e.es.clone()));
        let gloss = |es: Option<&spanish::Es>, person: usize, negative: bool| -> Option<String> {
            crate::gloss(opts, es_meaning.as_deref()?, es?, person, negative)
        };

        let tables = self
            .tenses
            .iter()
            .map(|tense| {
                let g = build(&root, vowel, tense.form);
                Table {
                    id: tense.id.clone(),
                    group: tense.group.clone(),
                    name: tense.name.clone(),
                    description: tense.description.clone(),
                    rows: self
                        .persons
                        .iter()
                        .map(|level| {
                            let i = match level.id.as_str() {
                                "informal" => 0,
                                "polite" => 1,
                                _ => 2,
                            };
                            let [first, second] = g[i].clone();
                            let (es, person) = match (&tense.es_formal, i) {
                                (Some(f), 2) => (Some(f), tense.person_formal),
                                _ => (tense.es.as_ref(), tense.person),
                            };
                            Row {
                                person: level.label.clone(),
                                clitic: String::new(),
                                es_singular: first.as_ref().and_then(|_| gloss(es, person, tense.negative)),
                                es_plural: second
                                    .as_ref()
                                    .and_then(|_| gloss(tense.es_2.as_ref(), tense.person_2, tense.negative)),
                                singular: first.unwrap_or_default(),
                                plural: second,
                            }
                        })
                        .collect(),
                }
            })
            .collect();

        Ok(Conjugated {
            root: entry.map(|e| e.entry.clone()).unwrap_or_else(|| format!("{root}-da")),
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

    fn cell(root: &str, form: Form, row: usize, col: usize) -> String {
        build(root, harmony(root), form)[row][col].clone().unwrap_or_default()
    }

    fn with(root: &str, vowel: &str, form: Form, row: usize, col: usize) -> String {
        build(root, vowel, form)[row][col].clone().unwrap_or_default()
    }

    #[test]
    fn harmony_rule() {
        // Table 2.8: meog-eos, sis-eos, guj-eos; gal-as, gob-as; §4.2.4:
        // yawg-as, dawl-as, chawj-as (aw + consonant).
        for r in ["meog", "sis", "guj", "geus", "sim", "jus", "ig"] {
            assert_eq!(harmony(r), "eo", "{r}");
        }
        for r in ["gal", "gob", "yawg", "dawl", "chawj", "gawm", "aj", "nol"] {
            assert_eq!(harmony(r), "a", "{r}");
        }
    }

    #[test]
    fn eo_suffixes() {
        // Ch. 6 §2.1–2.2 and the examples there.
        assert_eq!(eo("meog", "eo", "eoms"), "meog-eoms");
        assert_eq!(eo("chawj", "a", "eoms"), "chawj-ams");
        assert_eq!(eo("ka", "a", "eoms"), "ka-ms");
        assert_eq!(eo("pa", "a", "eos"), "pa-s");
        assert_eq!(eo("gwe", "eo", "eoms"), "gwe-yeoms");
        assert_eq!(eo("haw", "eo", "eos"), "haw-yeos");
        assert_eq!(eo("dekki", "eo", "eoms"), "dekky-eoms");
        assert_eq!(eo("sichi", "eo", "eos"), "sichy-eos");
        assert_eq!(eo("singgeu", "eo", "eos"), "singg-eos");
        assert_eq!(eo("ju", "eo", "eos"), "ju-eos");
        assert_eq!(eo("menggeul", "a", "eoms"), "menggeul-ams");
        assert_eq!(eo("sim", "eo", "eon"), "sim-eon");
        assert_eq!(eo("ga", "a", "eon"), "ga-n");
        assert_eq!(eo("nol", "a", "eon"), "nol-an");
    }

    #[test]
    fn eu_suffixes() {
        // -eun (§2.4.1): meog-eun-da, chawj-in-da, gawm-un-da, ga-n-da.
        assert_eq!(cell("meog", Form::Present, 0, 0), "meog-eun-da");
        assert_eq!(cell("chawj", Form::Present, 0, 0), "chawj-in-da");
        assert_eq!(cell("gawm", Form::Present, 0, 0), "gawm-un-da");
        assert_eq!(cell("ga", Form::Present, 0, 0), "ga-n-da");
        // -(eu)neu-n: meog-neu-n-ga, menggeu-neu-n-ga.
        assert_eq!(cell("meog", Form::Present, 0, 1), "meog-neu-n-ga");
        assert_eq!(cell("menggeul", Form::Present, 0, 1), "menggeu-neu-n-ga");
        // -euk (§2.3): dekki-k-yeo, sim-uk-yeo, jab-uk-yeo; menggeul-k.
        assert_eq!(cell("dekki", Form::Prospective, 0, 0), "dekki-k-yeo");
        assert_eq!(cell("sim", Form::Prospective, 0, 0), "sim-uk-yeo");
        assert_eq!(cell("jab", Form::Prospective, 0, 0), "jab-uk-yeo");
        assert_eq!(cell("ig", Form::Prospective, 0, 1), "ig-euk-eul-a");
        assert_eq!(cell("haw", Form::Prospective, 1, 0), "haw-k-u-da");
        assert_eq!(cell("dekki", Form::Prospective, 1, 1), "dekki-k-u-gwa");
        // -eulgeo (§2.4.3): chawj-ilgeo-yeo, jug-eulgeo-yeo, ga-lgeo-u-da,
        // chuli-lgeo-u-da.
        assert_eq!(cell("chawj", Form::Future, 0, 0), "chawj-ilgeo-yeo");
        assert_eq!(cell("jug", Form::Future, 0, 0), "jug-eulgeo-yeo");
        assert_eq!(cell("ga", Form::Future, 1, 0), "ga-lgeo-u-da");
        assert_eq!(cell("chuli", Form::Future, 1, 0), "chuli-lgeo-u-da");
    }

    #[test]
    fn sentence_enders() {
        // Ch. 7: meog-eoms-jeo, jus-eom-ga, menggeul-ams-u-da,
        // jus-eoms-ib-ne-da, gawl-ams-ib-ne-kka.
        assert_eq!(cell("meog", Form::Continuative, 0, 0), "meog-eoms-jeo");
        assert_eq!(cell("jus", Form::Continuative, 0, 1), "jus-eom-ga");
        assert_eq!(with("menggeul", "a", Form::Continuative, 1, 0), "menggeul-ams-u-da");
        assert_eq!(cell("jus", Form::Continuative, 2, 0), "jus-eoms-ib-ne-da");
        // sim-eos-ya, chawj-as-ya, sichy-eos-u-gwa, me-yeos-ib-di-ga.
        assert_eq!(cell("sim", Form::Perfective, 0, 1), "sim-eos-ya");
        assert_eq!(cell("chawj", Form::Perfective, 0, 1), "chawj-as-ya");
        assert_eq!(cell("sichi", Form::Perfective, 1, 1), "sichy-eos-u-gwa");
        assert_eq!(cell("pa", Form::Perfective, 0, 0), "pa-s-jeo");
        assert_eq!(cell("dekki", Form::Perfective, 0, 0), "dekky-eos-jeo");
        // jus-eon-ga, jug-eon, pa-n.
        assert_eq!(cell("jus", Form::Past, 0, 1), "jus-eon-ga");
        assert_eq!(cell("pa", Form::Past, 0, 0), "pa-n");
        // meog-eub-ne-da, aj-ib-ne-kka, sim-ub-ne-da.
        assert_eq!(cell("meog", Form::Present, 2, 0), "meog-eub-ne-da");
        assert_eq!(cell("aj", Form::Present, 2, 1), "aj-ib-ne-kka");
        assert_eq!(cell("sim", Form::Present, 2, 0), "sim-ub-ne-da");
        // Polite present doesn't exist for action verbs (§5.1).
        assert_eq!(build("meog", "eo", Form::Present)[1][0], None);
    }

    #[test]
    fn commands_and_negation() {
        // §3.4: meog-eula, aj-ila, sim-ula, seolleu-la; haw-ge;
        // §4.3: meog-eub-seo, chuli-b-seo.
        assert_eq!(cell("meog", Form::Imperative, 0, 0), "meog-eula");
        assert_eq!(cell("aj", Form::Imperative, 0, 0), "aj-ila");
        assert_eq!(cell("sim", Form::Imperative, 0, 0), "sim-ula");
        assert_eq!(cell("seolleu", Form::Imperative, 0, 0), "seolleu-la");
        assert_eq!(cell("haw", Form::Imperative, 0, 1), "haw-ge");
        assert_eq!(cell("meog", Form::Imperative, 2, 0), "meog-eub-seo");
        assert_eq!(cell("chuli", Form::Imperative, 2, 0), "chuli-b-seo");
        // Ch. 5 §5: an ga-k-yeo, ani haw-yeos-jeo, gob-ji-ji mal-la.
        assert_eq!(cell("ga", Form::NegativeProspective, 0, 0), "an ga-k-yeo");
        assert_eq!(cell("haw", Form::NegativePerfective, 0, 0), "an haw-yeos-jeo");
        assert_eq!(cell("meog", Form::CannotPerfective, 0, 0), "mos meog-eos-jeo");
        assert_eq!(cell("gobji", Form::Prohibitive, 0, 0), "gobji-ji mal-la");
    }

    #[test]
    fn input() {
        assert_eq!(normalize(" Meog-da ").unwrap(), "meogda");
        assert!(normalize("meog1").is_err());
        assert!(normalize("").is_err());
        assert_eq!(root_of("meogda").unwrap(), "meog");
        assert_eq!(root_of("haw").unwrap(), "haw");
        assert_eq!(root_of("da").unwrap(), "da");
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/jejueo.json")).unwrap();
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
        assert!(c.conjugate("meog").unwrap().known);
        assert!(c.conjugate("meogda").unwrap().known);
        let r = c.conjugate("menggeul-da").unwrap();
        assert_eq!(r.tables.iter().find(|t| t.id == "perfective").unwrap().rows[0].singular, "menggeul-as-jeo");
    }
}
