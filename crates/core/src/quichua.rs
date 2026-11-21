//! Conjugation for Santiago del Estero Quichua (quichua santiagueño), after
//! Domingo A. Bravo, *¿Quiere Ud. aprender quichua?* (La Banda: CEBIL, 5th
//! ed. 1990), and the vocabulary of *Yanasuspura, entre amigos* (Equipo
//! Quichua en Buenos Aires, FFyL-UBA, 2018). Spelling: Bravo's signografía
//! (ck, j, ll, sh, 'r, hu, qu), which both books use.
//!
//! A verb is a root, an optional tense suffix and a person ending:
//!
//! ```text
//! present   muna-ni  muna-nqui  muna-n  muna-nchis  muna-ycu  muna-nquichis  muna-ncu
//! past      muna-ra-ni  …  muna-ra  …
//! future    munásaj  munanqui  munancka  munasunchis  munasajcu  munanquichis  munanckancu
//! ```
//!
//! The persons are nocka, ckam, pay, nockanchis («we, with you»), nockaycu
//! («we, not you»), ckamcuna and paycuna.
//!
//! Roots in -i and -u (llójsiy «salir», púñuy «dormir») turn that vowel into
//! e and o in the past, in the 3rd person future and in the participle:
//! llojserani, llojsencka, llójsej; puñorani, puñoncka, púñoj (Bravo, «verbos
//! irregulares»). The present keeps it: llojsini, puñuni.
//!
//! Written accents follow Spanish rules, as Bravo writes them: a word stressed
//! on the second-to-last syllable that ends in -j takes an acute (munásaj,
//! múnaj).

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// munani «quiero».
    Present,
    /// munarani «quise, quería».
    Past,
    /// munásaj «querré».
    Future,
    /// munashcani «estoy queriendo».
    Progressive,
    /// munashcarani «estaba queriendo».
    PastProgressive,
    /// munasa cani «he querido».
    Perfect,
    /// munasa carani «había querido».
    Pluperfect,
    /// múnaj cani «suelo querer».
    Habitual,
    /// múnaj carani «solía querer».
    PastHabitual,
    /// munayman «querría».
    Conditional,
    /// munanayta «que yo quiera».
    Subjunctive,
    /// munaptiy «cuando yo quiera, si yo quiero».
    Switch,
    /// munay «¡quiere!», munaychis «¡quieran!».
    Imperative,
    /// munachun «¡que quiera!», munachuncu «¡que quieran!».
    Jussive,
    /// ama munaychu «¡no quieras!».
    Prohibitive,
    /// mana munanichu «no quiero».
    Negative,
    /// mana munaranichu «no quise».
    NegativePast,
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
    /// The infinitive, as Bravo writes it: "múnay", "llójsiy", "'riy".
    pub entry: String,
    pub meaning: String,
    #[serde(default)]
    pub es: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    /// Overrides the i → e, u → o change (true: it changes; false: it doesn't).
    #[serde(default)]
    pub shift: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conjugation {
    pub persons: Vec<Person>,
    pub tenses: Vec<Tense>,
    #[serde(default)]
    pub lexicon: Vec<Entry>,
}

fn is_vowel(c: char) -> bool {
    "aeiouáéíóúü".contains(c)
}

/// Drops written accents: múnay → munay.
fn plain(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' => 'u',
            other => other,
        })
        .collect()
}

/// The acute on the vowel of the second-to-last syllable: munasaj → munásaj.
fn accent_penult(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let vowels: Vec<usize> = chars.iter().enumerate().filter(|(_, c)| is_vowel(**c)).map(|(i, _)| i).collect();
    if vowels.len() < 2 {
        return word.to_string();
    }
    let i = vowels[vowels.len() - 2];
    let acute = match chars[i] {
        'a' => 'á',
        'e' => 'é',
        'i' => 'í',
        'o' => 'ó',
        'u' => 'ú',
        other => other,
    };
    chars[..i].iter().chain(std::iter::once(&acute)).chain(chars[i + 1..].iter()).collect()
}

/// Spanish spelling: a word ending in -j (stressed on the second-to-last
/// syllable, as Quichua words are) takes a written accent.
fn spell(word: String) -> String {
    if word.ends_with('j') { accent_penult(&word) } else { word }
}

/// Person endings after the root: nocka, ckam, pay, nockanchis, nockaycu,
/// ckamcuna, paycuna.
const PRESENT: [&str; 7] = ["ni", "nqui", "n", "nchis", "ycu", "nquichis", "ncu"];
/// After -ra- (past) the 3rd singular has no ending: munara.
const PAST: [&str; 7] = ["ni", "nqui", "", "nchis", "ycu", "nquichis", "ncu"];
/// The auxiliary cay «ser» in the present and the past.
const CA: [&str; 7] = ["cani", "canqui", "can", "canchis", "caycu", "canquichis", "cancu"];
const CARA: [&str; 7] = ["carani", "caranqui", "cara", "caranchis", "caraycu", "caranquichis", "carancu"];
/// Bravo: munásaj, munanqui, munancka, munasunchis, munasajcu, munanquichis, munanckancu.
const FUTURE: [&str; 7] = ["saj", "nqui", "ncka", "sunchis", "sajcu", "nquichis", "nckancu"];
const CONDITIONAL: [&str; 7] = ["yman", "nquiman", "nman", "nchisman", "ycuman", "nquichisman", "ncuman"];
const SUBJUNCTIVE: [&str; 7] = ["nayta", "nayquita", "nanta", "nanchista", "naycuta", "nayquichista", "nancuta"];
const SWITCH: [&str; 7] = ["ptiy", "ptiyqui", "ptin", "ptinchis", "ptiycu", "ptiyquichis", "ptincuna"];

struct Verb {
    /// The root, without the -y of the infinitive: muna, llojsi, 'ri.
    root: String,
    /// The root with i → e, u → o: llojse, puño.
    changed: String,
}

impl Verb {
    fn new(root: &str, shift: Option<bool>) -> Verb {
        let last = root.chars().last();
        let shifts = shift.unwrap_or(matches!(last, Some('i' | 'u')));
        let changed = match (shifts, last) {
            (true, Some('i')) => format!("{}e", &root[..root.len() - 1]),
            (true, Some('u')) => format!("{}o", &root[..root.len() - 1]),
            _ => root.to_string(),
        };
        Verb { root: root.to_string(), changed }
    }

    /// The active participle: múnaj, llójsej, púñoj.
    fn participle(&self) -> String {
        spell(format!("{}j", self.changed))
    }
}

fn build(v: &Verb, form: Form, p: usize) -> Option<String> {
    let r = &v.root;
    let c = &v.changed;
    Some(match form {
        Form::Present => format!("{r}{}", PRESENT[p]),
        Form::Past => format!("{c}ra{}", PAST[p]),
        Form::Future => {
            // The 3rd persons take the changed root: llojsencka, puñonckancu.
            let stem = if p == 2 || p == 6 { c } else { r };
            spell(format!("{stem}{}", FUTURE[p]))
        }
        Form::Progressive => format!("{r}shca{}", PRESENT[p]),
        Form::PastProgressive => format!("{r}shcara{}", PAST[p]),
        Form::Perfect => format!("{r}sa {}", CA[p]),
        Form::Pluperfect => format!("{r}sa {}", CARA[p]),
        Form::Habitual => format!("{} {}", v.participle(), CA[p]),
        Form::PastHabitual => format!("{} {}", v.participle(), CARA[p]),
        Form::Conditional => format!("{r}{}", CONDITIONAL[p]),
        Form::Subjunctive => format!("{r}{}", SUBJUNCTIVE[p]),
        Form::Switch => format!("{r}{}", SWITCH[p]),
        Form::Imperative => match p {
            1 => format!("{r}y"),
            5 => format!("{r}ychis"),
            _ => return None,
        },
        Form::Jussive => match p {
            2 => format!("{r}chun"),
            6 => format!("{r}chuncu"),
            _ => return None,
        },
        Form::Prohibitive => match p {
            1 => format!("ama {r}ychu"),
            5 => format!("ama {r}ychischu"),
            _ => return None,
        },
        Form::Negative => format!("mana {r}{}chu", PRESENT[p]),
        Form::NegativePast => format!("mana {c}ra{}chu", PAST[p]),
    })
}

/// Lowercase, one kind of apostrophe, no written accents.
fn normalize(verb: &str) -> std::result::Result<String, Error> {
    let w: String = plain(
        &verb
            .trim()
            .to_lowercase()
            .replace(['’', 'ʼ', '´', '`', '‘'], "'")
            .replace('\u{301}', ""),
    );
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

/// The root of a typed word: munay → muna; a bare root stays (muna).
fn root_of(word: &str) -> std::result::Result<String, Error> {
    let root = match word.strip_suffix('y') {
        Some(r) if r.chars().last().is_some_and(is_vowel) => r,
        _ => word,
    };
    if !root.chars().last().is_some_and(is_vowel) {
        return Err(Error::InvalidVerb);
    }
    Ok(root.to_string())
}

impl Conjugation {
    pub fn conjugate(&self, input: &str) -> std::result::Result<Conjugated, Error> {
        self.conjugate_with(input, &Options::default())
    }

    /// The entry for a typed word: the infinitive (munay, múnay) or the root
    /// (muna); "riy" also finds 'riy, written with Bravo's soft 'r.
    fn find(&self, word: &str) -> Option<&Entry> {
        let candidates = [word.to_string(), format!("{word}y"), format!("'{word}"), format!("'{word}y")];
        let n = if word.starts_with('r') { 4 } else { 2 };
        self.lexicon.iter().find(|e| {
            let key = plain(&e.entry.to_lowercase());
            candidates[..n].iter().any(|c| *c == key)
        })
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.find(&word);
        let root = match entry {
            Some(e) => root_of(&plain(&e.entry.to_lowercase()))?,
            None => root_of(&word)?,
        };
        let verb = Verb::new(&root, entry.and_then(|e| e.shift));

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
                    .map(|person| {
                        // Rows: 1 (nocka / nockanchis), 1x (— / nockaycu),
                        // 2 (ckam / ckamcuna), 3 (pay / paycuna).
                        let (sg, pl) = match person.id.as_str() {
                            "1" => (Some(0), Some(3)),
                            "1x" => (None, Some(4)),
                            "2" => (Some(1), Some(5)),
                            _ => (Some(2), Some(6)),
                        };
                        let sg = sg.and_then(|p| build(&verb, tense.form, p));
                        let pl = pl.and_then(|p| build(&verb, tense.form, p));
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
            root: entry.map(|e| e.entry.clone()).unwrap_or_else(|| format!("{root}y")),
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

    fn seven(root: &str, form: Form) -> Vec<String> {
        let v = Verb::new(root, None);
        (0..7).map(|p| build(&v, form, p).unwrap_or_default()).collect()
    }

    #[test]
    fn regular() {
        // Bravo: munani, munanqui, munan, munanchis, munaycu, munanquichis, munancu.
        assert_eq!(
            seven("muna", Form::Present),
            ["munani", "munanqui", "munan", "munanchis", "munaycu", "munanquichis", "munancu"]
        );
        assert_eq!(
            seven("muna", Form::Past),
            ["munarani", "munaranqui", "munara", "munaranchis", "munaraycu", "munaranquichis", "munarancu"]
        );
        assert_eq!(
            seven("muna", Form::Future),
            ["munásaj", "munanqui", "munancka", "munasunchis", "munasajcu", "munanquichis", "munanckancu"]
        );
        assert_eq!(seven("muna", Form::Progressive)[0], "munashcani");
        assert_eq!(seven("muna", Form::PastProgressive)[2], "munashcara");
        assert_eq!(seven("muna", Form::Pluperfect)[0], "munasa carani");
        assert_eq!(seven("muna", Form::Habitual)[0], "múnaj cani");
        assert_eq!(seven("muna", Form::Conditional)[1], "munanquiman");
        assert_eq!(seven("muna", Form::Switch)[6], "munaptincuna");
    }

    #[test]
    fn vowel_change() {
        // Bravo: llojserani, llojsencka, llojsenckancu, llójsej; the present keeps i.
        assert_eq!(seven("llojsi", Form::Present)[0], "llojsini");
        assert_eq!(seven("llojsi", Form::Past)[0], "llojserani");
        let fut = seven("llojsi", Form::Future);
        assert_eq!(fut[0], "llojsísaj");
        assert_eq!(fut[2], "llojsencka");
        assert_eq!(fut[6], "llojsenckancu");
        assert_eq!(seven("llojsi", Form::Habitual)[0], "llójsej cani");
        // púñuy: puñorani, puñoncka, púñoj.
        assert_eq!(seven("puñu", Form::Past)[0], "puñorani");
        assert_eq!(seven("puñu", Form::Future)[2], "puñoncka");
        assert_eq!(seven("puñu", Form::Future)[0], "puñúsaj");
        assert_eq!(seven("puñu", Form::Habitual)[0], "púñoj cani");
        // 'riy «ir»: 'rerani; amuy «venir»: amorancu.
        assert_eq!(seven("'ri", Form::Past)[0], "'rerani");
        assert_eq!(seven("amu", Form::Past)[6], "amorancu");
        // Regular -oy: ckoy «dar».
        assert_eq!(seven("cko", Form::Past)[0], "ckorani");
    }

    #[test]
    fn moods() {
        let v = Verb::new("muna", None);
        assert_eq!(build(&v, Form::Imperative, 1).unwrap(), "munay");
        assert_eq!(build(&v, Form::Imperative, 5).unwrap(), "munaychis");
        assert_eq!(build(&v, Form::Imperative, 0), None);
        assert_eq!(build(&v, Form::Jussive, 2).unwrap(), "munachun");
        assert_eq!(build(&v, Form::Jussive, 6).unwrap(), "munachuncu");
        // Bravo: ama pitaychu.
        assert_eq!(build(&Verb::new("pita", None), Form::Prohibitive, 1).unwrap(), "ama pitaychu");
        // Bravo: mana atinichu.
        assert_eq!(build(&Verb::new("ati", None), Form::Negative, 0).unwrap(), "mana atinichu");
        assert_eq!(build(&v, Form::Subjunctive, 0).unwrap(), "munanayta");
    }

    #[test]
    fn input() {
        assert_eq!(normalize(" Múnay ").unwrap(), "munay");
        assert_eq!(normalize("\u{2019}riy").unwrap(), "'riy");
        assert!(normalize("mun4y").is_err());
        assert_eq!(root_of("munay").unwrap(), "muna");
        assert_eq!(root_of("muna").unwrap(), "muna");
        assert_eq!(root_of("upiay").unwrap(), "upia");
        assert!(root_of("mun").is_err());
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/quichua.json")).unwrap();
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
        // Typed without the soft-r apostrophe or the accent, it is still found.
        assert!(c.conjugate("riy").unwrap().known);
        assert!(c.conjugate("munay").unwrap().known);
        let r = c.conjugate("llojsiy").unwrap();
        assert_eq!(r.tables[1].rows[0].singular, "llojserani");
    }
}
