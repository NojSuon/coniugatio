//! Conjugation for Alaskan Haida (X̱aad Kíl, Kaigani dialect of Northern
//! Haida), after Jordan Lachler, *Dictionary of Alaskan Haida* (Juneau:
//! Sealaska Heritage Institute, 2010), in its spelling: g̱ ḵ x̱ for the
//! sounds made at the back of the throat, an apostrophe for glottalized
//! consonants and an acute accent on the vowel with high pitch.
//!
//! A Haida verb doesn't change for person: who does it is shown by a pronoun
//! in front (Hl «I», dáng «you», hal «he, she», t'aláng «we», daláng «you
//! folks», tl' «they»). The verb takes a tense ending, and its stem changes
//! with the ending, in three patterns:
//!
//! ```text
//!              present      direct past  indirect past  future
//! táa  «eat»   táagang      táagan       táagaan        táasaang
//! isda «do»    isdáang      isdgán       isdáayaan      isdáasaang
//! t'asgad      t'asgíidang  t'asgíidan   t'asgadáan     t'asgíidsaang
//! ```
//!
//! The direct past tells what the speaker saw; the indirect past, what they
//! learned or inferred. Negation puts gám before the pronoun and -'ang on
//! the verb (gám Hl táa'anggang «I don't eat it»); commands use hlaa
//! (táa hlaa «eat it!») and, for several people, hl … -'uu.

use serde::{Deserialize, Serialize};

use crate::conjugate::{Error, Options, Person, Result as Conjugated, Row, Table};
use crate::spanish;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// Hl táagang «I am eating it» (SP).
    Present,
    /// Hl táagan «I ate it» (DP, seen).
    DirectPast,
    /// Hl táagaan «I ate it (it turns out)» (IP).
    IndirectPast,
    /// Hl táasaang «I will eat it».
    Future,
    /// táa hlaa «eat it!»; hl táa'uu «eat it, you folks!».
    Imperative,
    /// gám Hl táa'anggang «I don't eat it».
    Negative,
    /// gám Hl táa'anggan «I didn't eat it».
    NegativePast,
    /// gám Hl táa'angsaang «I won't eat it».
    NegativeFuture,
    /// gám hl táa'ang «don't eat it!».
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
    #[serde(default)]
    pub negative: bool,
    #[serde(default)]
    pub es: Option<spanish::Es>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The basic form, as the dictionary lists it: "táa", "isda", "t'asgad".
    pub entry: String,
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

/// Active subject pronouns: I, you, he/she, we, you folks, they.
const PRONOUNS: [&str; 6] = ["Hl", "dáng", "hal", "t'aláng", "daláng", "tl'"];

const ACCENTED: &str = "áéíóú";

fn is_vowel(c: char) -> bool {
    "aeiouáéíóú".contains(c)
}

fn has_accent(s: &str) -> bool {
    // Pitch is counted on the last word: dáa da'a → dáa da'áang.
    s.rsplit(' ').next().unwrap_or(s).chars().any(|c| ACCENTED.contains(c))
}

fn accent(c: char) -> char {
    match c {
        'a' => 'á',
        'e' => 'é',
        'i' => 'í',
        'o' => 'ó',
        'u' => 'ú',
        c => c,
    }
}

/// An ending, with high pitch when the stem has none: k'ah|gáng, aad|gán.
fn ending(stem: &str, plain: &str) -> String {
    // Two glottal stops are kept apart with a dot, as the dictionary does:
    // hl tlats'.'úu «plant (you folks)!».
    if stem.ends_with('\x27') && plain.starts_with('\x27') {
        return format!(".{}", ending(&stem[..stem.len() - 1], plain));
    }
    if has_accent(stem) {
        return plain.to_string();
    }
    let mut done = false;
    plain
        .chars()
        .map(|c| {
            if !done && is_vowel(c) {
                done = true;
                accent(c)
            } else {
                c
            }
        })
        .collect()
}

/// -gang, -gan, but -ang, -an after d and j: ḵ'íid|ang.
fn g_ending(stem: &str, plain: &str) -> String {
    if stem.ends_with('d') || stem.ends_with('j') {
        ending(stem, &plain[1..])
    } else {
        ending(stem, plain)
    }
}

/// The three stem patterns of the dictionary.
#[derive(Debug, PartialEq)]
enum Class {
    /// Ends in a short a after a consonant: isda, aada, da'a, tlaawhla.
    ShortA,
    /// Ends in short -ad or -as, which become -iid / -iij: t'asgad, ánt'as.
    Ablaut,
    /// Everything else: táa, ñíng, áalang, k'ah.
    Plain,
}

fn class(stem: &str) -> Class {
    let chars: Vec<char> = stem.chars().collect();
    let n = chars.len();
    if n >= 2 && chars[n - 1] == 'a' && !is_vowel(chars[n - 2]) {
        return Class::ShortA;
    }
    if n >= 3
        && (chars[n - 1] == 'd' || chars[n - 1] == 's')
        && chars[n - 2] == 'a'
        && !is_vowel(chars[n - 3])
    {
        return Class::Ablaut;
    }
    Class::Plain
}

/// The lengthened stem of a short-a verb: isda → isdáa, dáada → dáadaa.
fn long_a(stem: &str) -> String {
    let base = &stem[..stem.len() - 1];
    if has_accent(stem) {
        format!("{base}aa")
    } else {
        format!("{base}áa")
    }
}

/// The short stem of a short-a verb: isda → isd, da'a → daa, chiya → chii,
/// isdla → ishl.
fn short_a(stem: &str) -> String {
    let base = &stem[..stem.len() - 1];
    if let Some(b) = base.strip_suffix('\'') {
        if let Some(last) = b.chars().last().filter(|&c| is_vowel(c)) {
            return format!("{b}{last}");
        }
    }
    if let Some(b) = base.strip_suffix("iy") {
        return format!("{b}ii");
    }
    if let Some(b) = base.strip_suffix("uw") {
        return format!("{b}uu");
    }
    // dadg(a)gán: the a stays before the g of the ending.
    if base.ends_with('g') {
        return stem.to_string();
    }
    if let Some(b) = base.strip_suffix("sdl") {
        return format!("{b}shl");
    }
    base.to_string()
}

/// The present stem of a plain verb: níihl → níl, chaawéehl → chaawéel.
fn plain_stem(stem: &str) -> String {
    if let Some(b) = stem.strip_suffix("íihl") {
        return format!("{b}íl");
    }
    if let Some(b) = stem.strip_suffix("iihl") {
        return format!("{b}il");
    }
    if let Some(b) = stem.strip_suffix("hl") {
        if b.chars().last().is_some_and(is_vowel) {
            return format!("{b}l");
        }
    }
    stem.to_string()
}

/// The present stem of an ablaut verb: t'asgad → t'asgíid, ánt'as → ánt'iij.
fn ablaut_stem(stem: &str) -> String {
    let base = &stem[..stem.len() - 2];
    let end = if stem.ends_with('s') { "j" } else { "d" };
    let ii = if has_accent(base) { "ii" } else { "íi" };
    format!("{base}{ii}{end}")
}

/// The indirect past of a plain verb: táa → táagaan, tl'íi → tl'áayaan,
/// k'ajúu → k'ajáawaan, ñíng → ñéengaan, áalang → áalaangaan.
fn plain_indirect(stem: &str) -> String {
    for (from, to) in [("íi", "áay"), ("ii", "aay"), ("úu", "áaw"), ("uu", "aaw")] {
        if let Some(b) = stem.strip_suffix(from) {
            return format!("{b}{to}aan");
        }
    }
    let last = stem.chars().last().unwrap_or('a');
    if is_vowel(last) || last == 'h' {
        return format!("{stem}{}", ending(stem, "gaan"));
    }
    // Lengthen the last vowel if it is short: a → aa, i → ee.
    let chars: Vec<char> = stem.chars().collect();
    let Some(v) = chars.iter().rposition(|&c| is_vowel(c)) else {
        return format!("{stem}aan");
    };
    let long = v > 0 && is_vowel(chars[v - 1]) || chars.get(v + 1).is_some_and(|&c| is_vowel(c));
    let mut out: String = chars[..v].iter().collect();
    if long {
        out.push(chars[v]);
    } else {
        match chars[v] {
            'a' => out.push_str("aa"),
            'á' => out.push_str("áa"),
            'i' => out.push_str("ee"),
            'í' => out.push_str("ée"),
            'u' => out.push_str("uu"),
            'ú' => out.push_str("úu"),
            c => out.push(c),
        }
    }
    out.extend(&chars[v + 1..]);
    let suffix = ending(&out, "aan");
    format!("{out}{suffix}")
}

/// The verb alone, without pronoun or gám.
fn verb(stem: &str, form: Form, plural: bool) -> String {
    let class = class(stem);
    // The stem before -'ang and -'uu: isd'áng, táa'ang, t'asgíid'ang.
    let neg_base = match class {
        Class::ShortA => short_a(stem),
        Class::Ablaut => ablaut_stem(stem),
        Class::Plain => plain_stem(stem),
    };
    let neg = |rest: &str| format!("{neg_base}{}{rest}", ending(&neg_base, "'ang"));
    match form {
        Form::Present => match class {
            Class::ShortA => format!("{}ng", long_a(stem)),
            Class::Ablaut => format!("{}ang", ablaut_stem(stem)),
            Class::Plain => {
                let s = plain_stem(stem);
                let e = g_ending(&s, "gang");
                format!("{s}{e}")
            }
        },
        Form::DirectPast => match class {
            Class::ShortA => {
                let s = short_a(stem);
                let e = ending(&s, "gan");
                format!("{s}{e}")
            }
            Class::Ablaut => format!("{}an", ablaut_stem(stem)),
            Class::Plain => {
                let s = plain_stem(stem);
                let e = g_ending(&s, "gan");
                format!("{s}{e}")
            }
        },
        Form::IndirectPast => match class {
            Class::ShortA => format!("{}yaan", long_a(stem)),
            Class::Ablaut => {
                let base = if stem.ends_with('s') { format!("{}j", &stem[..stem.len() - 1]) } else { stem.to_string() };
                let e = ending(&base, "aan");
                format!("{base}{e}")
            }
            Class::Plain => plain_indirect(&plain_stem(stem)),
        },
        Form::Future => {
            let s = match class {
                Class::ShortA => long_a(stem),
                Class::Ablaut => ablaut_stem(stem),
                Class::Plain => plain_stem(stem),
            };
            let e = ending(&s, "saang");
            format!("{s}{e}")
        }
        Form::Imperative => {
            if plural {
                let e = ending(&neg_base, "'uu");
                format!("hl {neg_base}{e}")
            } else {
                let s = match class {
                    Class::ShortA => long_a(stem),
                    Class::Ablaut => stem.to_string(),
                    Class::Plain => plain_stem(stem),
                };
                format!("{s} hlaa")
            }
        }
        Form::Negative => neg("gang"),
        Form::NegativePast => neg("gan"),
        Form::NegativeFuture => neg("saang"),
        Form::Prohibitive => {
            if plural {
                format!("gám hl {}'uu", neg(""))
            } else {
                format!("gám hl {}", neg(""))
            }
        }
    }
}

/// The whole form for person `p` (0–5), or None where it doesn't exist.
fn build(stem: &str, form: Form, p: usize) -> Option<String> {
    let plural = p >= 3;
    match form {
        Form::Imperative | Form::Prohibitive => match p {
            1 | 4 => Some(verb(stem, form, plural)),
            _ => None,
        },
        Form::Negative | Form::NegativePast | Form::NegativeFuture => {
            Some(format!("gám {} {}", PRONOUNS[p], verb(stem, form, plural)))
        }
        _ => Some(format!("{} {}", PRONOUNS[p], verb(stem, form, plural))),
    }
}

/// Lowercase, one apostrophe, and the dictionary's underlined letters:
/// g̱ ḵ x̱ (also typed g_ k_ x_, or with U+0332).
fn normalize(input: &str) -> std::result::Result<String, Error> {
    let mut w = input.trim().to_lowercase().replace(['’', 'ʼ', '´', '`', '‘'], "'");
    for (from, to) in [
        ("\u{332}", "\u{331}"),
        ("k\u{331}", "ḵ"),
        ("g_", "g\u{331}"),
        ("k_", "ḵ"),
        ("x_", "x\u{331}"),
    ] {
        w = w.replace(from, to);
    }
    if w.is_empty() {
        return Err(Error::EmptyVerb);
    }
    if !w.chars().all(|c| c.is_alphabetic() || c == '\'' || c == ' ' || c == '\u{331}')
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

    /// The entry for a typed word, with or without its accent.
    fn find(&self, word: &str) -> Option<&Entry> {
        let bare = |s: &str| -> String {
            s.chars()
                .map(|c| match c {
                    'á' => 'a',
                    'é' => 'e',
                    'í' => 'i',
                    'ó' => 'o',
                    'ú' => 'u',
                    c => c,
                })
                .collect()
        };
        self.lexicon
            .iter()
            .find(|e| e.entry.to_lowercase() == word)
            .or_else(|| self.lexicon.iter().find(|e| bare(&e.entry.to_lowercase()) == bare(word)))
    }

    pub fn conjugate_with(&self, input: &str, opts: &Options) -> std::result::Result<Conjugated, Error> {
        let word = normalize(input)?;
        let entry = self.find(&word);
        let stem = entry.map(|e| e.entry.to_lowercase()).unwrap_or(word);

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
                        let p = person.es.unwrap_or(0) as usize;
                        let sg = build(&stem, tense.form, p);
                        let pl = build(&stem, tense.form, p + 3);
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
            root: entry.map(|e| e.entry.clone()).unwrap_or(stem),
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

    /// Present, direct past, indirect past, as in the dictionary.
    fn three(stem: &str) -> [String; 3] {
        [
            verb(stem, Form::Present, false),
            verb(stem, Form::DirectPast, false),
            verb(stem, Form::IndirectPast, false),
        ]
    }

    #[test]
    fn dictionary_forms() {
        // Lachler 2010: SP | DP | IP.
        assert_eq!(three("táa"), ["táagang", "táagan", "táagaan"]);
        assert_eq!(three("aada"), ["aadáang", "aadgán", "aadáayaan"]);
        assert_eq!(three("isda"), ["isdáang", "isdgán", "isdáayaan"]);
        assert_eq!(three("dáada"), ["dáadaang", "dáadgan", "dáadaayaan"]);
        assert_eq!(three("da'a"), ["da'áang", "daagán", "da'áayaan"]);
        assert_eq!(three("chiya"), ["chiyáang", "chiigán", "chiyáayaan"]);
        assert_eq!(three("da g̱asdla"), ["da g̱asdláang", "da g̱ashlgán", "da g̱asdláayaan"]);
        assert_eq!(three("k'ajúu"), ["k'ajúugang", "k'ajúugan", "k'ajáawaan"]);
        assert_eq!(three("tl'íi"), ["tl'íigang", "tl'íigan", "tl'áayaan"]);
        assert_eq!(three("ḵíng"), ["ḵínggang", "ḵínggan", "ḵéengaan"]);
        assert_eq!(three("áalang"), ["áalanggang", "áalanggan", "áalaangaan"]);
        assert_eq!(three("sdíihl"), ["sdílgang", "sdílgan", "sdéelaan"]);
        assert_eq!(three("k'ah"), ["k'ahgáng", "k'ahgán", "k'ahgáan"]);
        assert_eq!(three("t'asgad"), ["t'asgíidang", "t'asgíidan", "t'asgadáan"]);
        assert_eq!(three("ánt'as"), ["ánt'iijang", "ánt'iijan", "ánt'ajaan"]);
        assert_eq!(three("chaawéehl"), ["chaawéelgang", "chaawéelgan", "chaawéelaan"]);
    }

    #[test]
    fn other_forms() {
        // Examples in the dictionary: táasaang, isdáasaang, xiláadaasaang,
        // sdíihlsaang (here sdílsaang, the present stem).
        assert_eq!(verb("táa", Form::Future, false), "táasaang");
        assert_eq!(verb("isda", Form::Future, false), "isdáasaang");
        assert_eq!(verb("xiláada", Form::Future, false), "xiláadaasaang");
        // gatáa'anggan «didn't eat», isd'ángsaang «won't give».
        assert_eq!(verb("gatáa", Form::NegativePast, false), "gatáa'anggan");
        assert_eq!(verb("isda", Form::NegativeFuture, false), "isd'ángsaang");
        // K'ajúu hlaa! «Sing!», hl tlats'.'úu «plant (you folks)!»,
        // gám hl táa'ang «don't eat it», gám hl ḵíng'ang'uu.
        assert_eq!(verb("k'ajúu", Form::Imperative, false), "k'ajúu hlaa");
        assert_eq!(verb("tlats'a", Form::Imperative, true), "hl tlats'.'úu");
        assert_eq!(verb("táa", Form::Prohibitive, false), "gám hl táa'ang");
        assert_eq!(verb("ḵíng", Form::Prohibitive, true), "gám hl ḵíng'ang'uu");
        // With pronouns.
        assert_eq!(build("gatáa", Form::Present, 2).unwrap(), "hal gatáagang");
        assert_eq!(build("gatáa", Form::Future, 3).unwrap(), "t'aláng gatáasaang");
        assert_eq!(build("gatáa", Form::NegativePast, 0).unwrap(), "gám Hl gatáa'anggan");
        assert_eq!(build("gatáa", Form::Imperative, 0), None);
    }

    #[test]
    fn input() {
        assert_eq!(normalize(" K\u{332}íng ").unwrap(), "ḵíng");
        assert_eq!(normalize("x_aad").unwrap(), "x\u{331}aad");
        assert!(normalize("ta4").is_err());
    }

    #[test]
    fn lexicon_round_trip() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/languages/haida.json")).unwrap();
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
        // Typed without accents or with g_ / k_.
        assert!(c.conjugate("gataa").unwrap().known);
        assert_eq!(c.conjugate("k_ing").unwrap().tables[0].rows[0].singular, "Hl ḵínggang");
        assert_eq!(c.conjugate("isda").unwrap().tables[1].rows[2].plural.as_deref(), Some("tl' isdgán"));
    }

    /// Checks the rules against every verb in the dictionary, from a table
    /// made with pdftotext (head, gloss, SP, DP, IP): HAIDA_VERBS=path.
    #[test]
    #[ignore]
    fn whole_dictionary() {
        let Ok(path) = std::env::var("HAIDA_VERBS") else { return };
        let text = std::fs::read_to_string(path).unwrap();
        let fix = |s: &str| -> String {
            s.replace('ç', "g\u{331}").replace('ñ', "ḵ").replace('ý', "x\u{331}").replace(['|', '¬'], "").trim().to_string()
        };
        let (mut ok, mut bad) = (0, 0);
        for line in text.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 6 || f[0].is_empty() || f[0].starts_with('-') {
                continue;
            }
            let head = fix(f[0]);
            let want = [fix(f[3]), fix(f[4]), fix(f[5].split(" NOTE").next().unwrap_or(""))];
            let got = three(&head);
            if got[0] == want[0] && got[1] == want[1] && got[2] == want[2] {
                ok += 1;
                println!("OK\t{head}\t{}", f[1]);
            } else {
                bad += 1;
                println!("BAD\t{head}\t{:?}\t{:?}", got, want);
            }
        }
        println!("{ok} ok, {bad} differ");
    }
}
