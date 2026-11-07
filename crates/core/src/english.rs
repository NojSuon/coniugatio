//! A small English conjugator for the English pages: the column next to
//! each form says it in English ("dá pädi" → "I knew").
//!
//! It reads the same specs as the Spanish one (`spanish::Es`): a tense, or
//! a construction with an auxiliary. On the English pages those specs and
//! the lexicon meanings come in English from `data/languages/en/`:
//!
//! ```json
//! "present"                                            // I sleep
//! { "tense": "present", "aux": "need", "link": "to" }  // I need to sleep
//! { "tense": "progressive", "aux": "go", "link": "to" } // I am going to sleep
//! { "tense": "present", "aux": "be", "link": "about to" } // I am about to sleep
//! { "tense": "perfect", "adverb": "just" }             // I have just slept
//! { "tense": "present", "before": "when" }             // when I sleep
//! ```
//!
//! A meaning is an English verb, with anything after it kept as is:
//! "sleep", "do it", "drink water", "to see" (a leading "to" is dropped).

use crate::spanish::{Es, Tense};

/// I, you, he/she, we, you all, they — the order of `spanish::PRONOUNS`.
pub const PRONOUNS: [&str; 6] = ["I", "you", "he/she", "we", "you all", "they"];
/// Object pronouns, for "let me sing", "let them sing".
const OBJECTS: [&str; 6] = ["me", "you", "him/her", "us", "you all", "them"];

/// (base, 3rd person -s, past, past participle)
const IRREGULAR: &[(&str, &str, &str, &str)] = &[
    ("be", "is", "was", "been"),
    ("have", "has", "had", "had"),
    ("do", "does", "did", "done"),
    ("go", "goes", "went", "gone"),
    ("say", "says", "said", "said"),
    ("make", "makes", "made", "made"),
    ("see", "sees", "saw", "seen"),
    ("know", "knows", "knew", "known"),
    ("take", "takes", "took", "taken"),
    ("come", "comes", "came", "come"),
    ("become", "becomes", "became", "become"),
    ("give", "gives", "gave", "given"),
    ("get", "gets", "got", "gotten"),
    ("eat", "eats", "ate", "eaten"),
    ("drink", "drinks", "drank", "drunk"),
    ("sleep", "sleeps", "slept", "slept"),
    ("sing", "sings", "sang", "sung"),
    ("speak", "speaks", "spoke", "spoken"),
    ("write", "writes", "wrote", "written"),
    ("read", "reads", "read", "read"),
    ("run", "runs", "ran", "run"),
    ("sit", "sits", "sat", "sat"),
    ("fall", "falls", "fell", "fallen"),
    ("feel", "feels", "felt", "felt"),
    ("hear", "hears", "heard", "heard"),
    ("listen", "listens", "listened", "listened"),
    ("buy", "buys", "bought", "bought"),
    ("bring", "brings", "brought", "brought"),
    ("think", "thinks", "thought", "thought"),
    ("teach", "teaches", "taught", "taught"),
    ("learn", "learns", "learned", "learned"),
    ("die", "dies", "died", "died"),
    ("fly", "flies", "flew", "flown"),
    ("grow", "grows", "grew", "grown"),
    ("sell", "sells", "sold", "sold"),
    ("steal", "steals", "stole", "stolen"),
    ("weave", "weaves", "wove", "woven"),
    ("grind", "grinds", "ground", "ground"),
    ("hit", "hits", "hit", "hit"),
    ("put", "puts", "put", "put"),
    ("cut", "cuts", "cut", "cut"),
    ("swim", "swims", "swam", "swum"),
    ("begin", "begins", "began", "begun"),
    ("win", "wins", "won", "won"),
    ("forget", "forgets", "forgot", "forgotten"),
    ("understand", "understands", "understood", "understood"),
    ("stand", "stands", "stood", "stood"),
    ("leave", "leaves", "left", "left"),
    ("lose", "loses", "lost", "lost"),
    ("find", "finds", "found", "found"),
    ("tell", "tells", "told", "told"),
    ("wear", "wears", "wore", "worn"),
    ("catch", "catches", "caught", "caught"),
    ("fight", "fights", "fought", "fought"),
    ("draw", "draws", "drew", "drawn"),
    ("throw", "throws", "threw", "thrown"),
    ("ride", "rides", "rode", "ridden"),
    ("rise", "rises", "rose", "risen"),
    ("wake", "wakes", "woke", "woken"),
    ("choose", "chooses", "chose", "chosen"),
    ("mean", "means", "meant", "meant"),
    ("meet", "meets", "met", "met"),
    ("pay", "pays", "paid", "paid"),
    ("send", "sends", "sent", "sent"),
    ("spend", "spends", "spent", "spent"),
    ("build", "builds", "built", "built"),
    ("keep", "keeps", "kept", "kept"),
    ("hold", "holds", "held", "held"),
    ("lie", "lies", "lay", "lain"),
    ("lay", "lays", "laid", "laid"),
    ("can", "can", "could", "been able to"),
    ("bear", "bears", "bore", "born"),
    ("shine", "shines", "shone", "shone"),
    ("hang", "hangs", "hung", "hung"),
    ("bite", "bites", "bit", "bitten"),
    ("blow", "blows", "blew", "blown"),
    ("break", "breaks", "broke", "broken"),
    ("hide", "hides", "hid", "hidden"),
    ("shake", "shakes", "shook", "shaken"),
    ("sweep", "sweeps", "swept", "swept"),
    ("dig", "digs", "dug", "dug"),
    ("spin", "spins", "spun", "spun"),
    ("light", "lights", "lit", "lit"),
    ("dream", "dreams", "dreamed", "dreamed"),
    ("show", "shows", "showed", "shown"),
    ("sew", "sews", "sewed", "sewn"),
    ("sow", "sows", "sowed", "sown"),
    ("lead", "leads", "led", "led"),
    ("feed", "feeds", "fed", "fed"),
    ("bleed", "bleeds", "bled", "bled"),
    ("shoot", "shoots", "shot", "shot"),
    ("let", "lets", "let", "let"),
    ("set", "sets", "set", "set"),
    ("shut", "shuts", "shut", "shut"),
    ("quit", "quits", "quit", "quit"),
    ("cost", "costs", "cost", "cost"),
    ("hurt", "hurts", "hurt", "hurt"),
];

fn irregular(v: &str) -> Option<&'static (&'static str, &'static str, &'static str, &'static str)> {
    IRREGULAR.iter().find(|i| i.0 == v)
}

fn is_vowel(c: char) -> bool {
    "aeiou".contains(c)
}

/// One-syllable consonant–vowel–consonant verbs double their last letter:
/// stop → stopped, stopping; plan → planned.
fn doubles(v: &str) -> bool {
    let c: Vec<char> = v.chars().collect();
    let n = c.len();
    if n < 3 {
        return false;
    }
    let (a, b, z) = (c[n - 3], c[n - 2], c[n - 1]);
    let syllables = c
        .windows(2)
        .filter(|w| !is_vowel(w[0]) && is_vowel(w[1]))
        .count()
        + usize::from(is_vowel(c[0]));
    syllables == 1 && !is_vowel(a) && is_vowel(b) && !is_vowel(z) && !"wxy".contains(z)
}

fn third(v: &str) -> String {
    if let Some(i) = irregular(v) {
        return i.1.to_string();
    }
    let ends = |s: &str| v.ends_with(s);
    if ends("s") || ends("x") || ends("z") || ends("ch") || ends("sh") || ends("o") {
        format!("{v}es")
    } else if ends("y") && !v.chars().rev().nth(1).is_some_and(is_vowel) {
        format!("{}ies", &v[..v.len() - 1])
    } else {
        format!("{v}s")
    }
}

fn past(v: &str) -> String {
    if let Some(i) = irregular(v) {
        return i.2.to_string();
    }
    if v.ends_with('e') {
        format!("{v}d")
    } else if v.ends_with('y') && !v.chars().rev().nth(1).is_some_and(is_vowel) {
        format!("{}ied", &v[..v.len() - 1])
    } else if doubles(v) {
        format!("{v}{}ed", v.chars().last().unwrap())
    } else {
        format!("{v}ed")
    }
}

fn participle(v: &str) -> String {
    irregular(v)
        .map(|i| i.3.to_string())
        .unwrap_or_else(|| past(v))
}

fn gerund(v: &str) -> String {
    if v == "be" {
        return "being".into();
    }
    if let Some(stem) = v.strip_suffix("ie") {
        return format!("{stem}ying");
    }
    if v.ends_with('e')
        && !v.ends_with("ee")
        && !v.ends_with("ye")
        && !v.ends_with("oe")
        && v.len() > 2
    {
        return format!("{}ing", &v[..v.len() - 1]);
    }
    if doubles(v) {
        return format!("{v}{}ing", v.chars().last().unwrap());
    }
    format!("{v}ing")
}

fn be_present(p: usize) -> &'static str {
    ["am", "are", "is", "are", "are", "are"][p]
}

fn be_past(p: usize) -> &'static str {
    ["was", "were", "was", "were", "were", "were"][p]
}

/// A verb phrase: the finite part first (it moves in questions and takes
/// "not"), then the rest. `support` marks a phrase that needs do/does/did
/// in questions and negatives (simple present and past of most verbs).
struct Phrase {
    finite: String,
    rest: Vec<String>,
    /// do-support form ("do"/"does"/"did") and the bare verb.
    support: Option<(&'static str, String)>,
}

/// Modal verbs: no -s, no do-support (he can sleep, can he sleep?).
const MODALS: [&str; 5] = ["can", "might", "must", "should", "may"];

fn phrase(v: &str, tense: Tense, p: usize) -> Phrase {
    let simple = |finite: String, rest: Vec<String>| Phrase {
        finite,
        rest,
        support: None,
    };
    if MODALS.contains(&v) {
        let finite = match (v, tense) {
            ("can", Tense::Preterite | Tense::Conditional) => "could",
            _ => v,
        };
        return simple(finite.to_string(), vec![]);
    }
    let with = |aux: &str, rest: String| simple(aux.to_string(), vec![rest]);
    match tense {
        Tense::Present | Tense::Subjunctive if v == "be" && tense == Tense::Present => {
            simple(be_present(p).into(), vec![])
        }
        Tense::Present => Phrase {
            finite: if p == 2 { third(v) } else { v.to_string() },
            rest: vec![],
            support: Some((if p == 2 { "does" } else { "do" }, v.to_string())),
        },
        Tense::Preterite if v == "be" => simple(be_past(p).into(), vec![]),
        Tense::Preterite => Phrase {
            finite: past(v),
            rest: vec![],
            support: Some(("did", v.to_string())),
        },
        Tense::Imperfect => Phrase {
            finite: "used".into(),
            rest: vec!["to".into(), v.to_string()],
            support: Some(("did", format!("use to {v}"))),
        },
        Tense::Future => with("will", v.to_string()),
        Tense::Conditional => with("would", v.to_string()),
        Tense::Progressive => with(be_present(p), gerund(v)),
        Tense::PastProgressive => with(be_past(p), gerund(v)),
        Tense::Perfect => with(if p == 2 { "has" } else { "have" }, participle(v)),
        // Subjunctive, imperative and optative are built in `render`.
        Tense::Subjunctive | Tense::Imperative | Tense::Optative => simple(v.to_string(), vec![]),
    }
}

/// Says a meaning ("sleep", "do it") in English for one person (0..6, in
/// the order of [`PRONOUNS`]). `None` when the form doesn't exist (an
/// order for "I") or there is no meaning.
pub fn render(meaning: &str, es: &Es, person: usize, negative: bool) -> Option<String> {
    let spec = match es {
        Es::Simple(t) => crate::spanish::Spec {
            tense: *t,
            aux: None,
            link: None,
            before: None,
            adverb: None,
            pronoun: true,
            negative: false,
            question: false,
        },
        Es::Spec(s) => s.clone(),
    };
    let negative = negative || spec.negative;
    let mut words = meaning.split_whitespace().map(str::to_lowercase);
    let mut verb = words.next()?;
    if verb == "to" {
        verb = words.next()?;
    }
    if !verb.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let tail: Vec<String> = words.collect();
    let second = person == 1 || person == 4;

    // The main verb as it follows an auxiliary: "need to sleep".
    let infinitive = || {
        let mut v = vec![];
        match spec.link.as_deref() {
            // "-ing": the verb as a gerund (I finished sleeping).
            Some("-ing") => return vec![gerund(&verb)],
            Some(l) => v.push(l.to_string()),
            None => {}
        }
        v.push(verb.clone());
        v
    };

    let mut tense = spec.tense;
    if tense == Tense::Optative {
        tense = if second {
            Tense::Imperative
        } else {
            Tense::Optative
        };
    }
    let finish = |mut parts: Vec<String>, end: &str| {
        parts.extend(tail.iter().cloned());
        let text = parts.join(" ");
        Some(match end {
            "!" => format!("{text}!"),
            _ if spec.question => format!("{text}?"),
            _ => text,
        })
    };
    let mut before: Vec<String> = spec.before.iter().cloned().collect();

    match tense {
        Tense::Imperative => {
            if !second {
                return None;
            }
            let mut parts = vec![];
            if negative {
                parts.push("don't".to_string());
            }
            if let Some(aux) = &spec.aux {
                parts.push(aux.clone());
                parts.extend(infinitive());
            } else {
                parts.push(verb.clone());
            }
            return finish(parts, "!");
        }
        // let me sing, let him/her not sing
        Tense::Optative => {
            before.extend(["let".to_string(), OBJECTS[person].to_string()]);
            if negative {
                before.push("not".into());
            }
            before.push(verb.clone());
            return finish(before, "");
        }
        // that I sing, that he/she not sing
        Tense::Subjunctive => {
            if spec.pronoun {
                before.push(PRONOUNS[person].into());
            }
            if negative {
                before.push("not".into());
            }
            before.push(if verb == "be" {
                "be".into()
            } else {
                verb.clone()
            });
            return finish(before, "");
        }
        _ => {}
    }

    // An auxiliary takes the tense; the meaning follows: I need to sleep.
    let (main, extra) = match &spec.aux {
        Some(aux) => (aux.clone(), infinitive()),
        None => (verb.clone(), vec![]),
    };
    let ph = phrase(&main, tense, person);
    let subject: Vec<String> = if spec.pronoun {
        vec![PRONOUNS[person].to_string()]
    } else {
        vec![]
    };
    let adverb: Vec<String> = spec.adverb.iter().cloned().collect();

    let mut parts = before;
    match (&ph.support, negative || spec.question) {
        // do-support: did he sing? / he didn't sing / I almost slept
        (Some((aux, bare)), true) => {
            let aux = if negative {
                format!("{aux}n't")
            } else {
                aux.to_string()
            };
            if spec.question {
                parts.push(aux);
                parts.extend(subject);
            } else {
                parts.extend(subject);
                parts.push(aux);
            }
            parts.extend(adverb);
            parts.push(bare.clone());
        }
        (Some(_), false) => {
            parts.extend(subject);
            parts.extend(adverb);
            parts.push(ph.finite);
            parts.extend(ph.rest);
        }
        // An auxiliary or "be": is he singing? / he isn't singing
        (None, _) => {
            let finite = if negative {
                negate(&ph.finite)
            } else {
                ph.finite
            };
            if spec.question {
                parts.push(finite);
                parts.extend(subject);
            } else {
                parts.extend(subject);
                parts.push(finite);
            }
            parts.extend(adverb);
            parts.extend(ph.rest);
        }
    }
    parts.extend(extra);
    finish(parts, "")
}

fn negate(aux: &str) -> String {
    match aux {
        "will" => "won't".into(),
        "am" => "am not".into(),
        "can" => "can't".into(),
        "might" => "might not".into(),
        "may" => "may not".into(),
        a => format!("{a}n't"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(m: &str, es: &str, p: usize, neg: bool) -> String {
        let es: Es = serde_json::from_str(es).unwrap();
        render(m, &es, p, neg).unwrap()
    }

    #[test]
    fn tenses() {
        assert_eq!(r("sing", r#""present""#, 0, false), "I sing");
        assert_eq!(r("sing", r#""present""#, 2, false), "he/she sings");
        assert_eq!(r("go", r#""present""#, 2, false), "he/she goes");
        assert_eq!(r("cry", r#""present""#, 2, false), "he/she cries");
        assert_eq!(r("sing", r#""preterite""#, 0, false), "I sang");
        assert_eq!(r("stop", r#""preterite""#, 0, false), "I stopped");
        assert_eq!(r("plough", r#""preterite""#, 0, false), "I ploughed");
        assert_eq!(r("sing", r#""future""#, 3, false), "we will sing");
        assert_eq!(r("sing", r#""imperfect""#, 0, false), "I used to sing");
        assert_eq!(r("sing", r#""progressive""#, 0, false), "I am singing");
        assert_eq!(
            r("write", r#""progressive""#, 2, false),
            "he/she is writing"
        );
        assert_eq!(r("die", r#""progressive""#, 2, false), "he/she is dying");
        assert_eq!(
            r("sing", r#""past_progressive""#, 3, false),
            "we were singing"
        );
        assert_eq!(r("write", r#""perfect""#, 2, false), "he/she has written");
        assert_eq!(r("sing", r#""conditional""#, 5, false), "they would sing");
        assert_eq!(r("be", r#""present""#, 0, false), "I am");
        assert_eq!(r("be", r#""preterite""#, 5, false), "they were");
    }

    #[test]
    fn negatives_and_questions() {
        assert_eq!(r("sing", r#""present""#, 2, true), "he/she doesn't sing");
        assert_eq!(r("sing", r#""preterite""#, 0, true), "I didn't sing");
        assert_eq!(r("sing", r#""future""#, 0, true), "I won't sing");
        assert_eq!(r("sing", r#""progressive""#, 0, true), "I am not singing");
        assert_eq!(r("sing", r#""perfect""#, 1, true), "you haven't sung");
        let q = |t: &str| format!(r#"{{"tense":"{t}","question":true}}"#);
        assert_eq!(r("sing", &q("present"), 2, false), "does he/she sing?");
        assert_eq!(r("sing", &q("preterite"), 1, false), "did you sing?");
        assert_eq!(r("sing", &q("progressive"), 2, false), "is he/she singing?");
        assert_eq!(r("sing", &q("future"), 3, false), "will we sing?");
        assert_eq!(r("sing", &q("imperfect"), 0, false), "did I use to sing?");
    }

    #[test]
    fn orders_and_wishes() {
        assert_eq!(r("do it", r#""imperative""#, 1, false), "do it!");
        assert_eq!(r("sing", r#""imperative""#, 4, true), "don't sing!");
        assert!(render("sing", &Es::Simple(Tense::Imperative), 0, false).is_none());
        assert_eq!(r("speak", r#""optative""#, 0, false), "let me speak");
        assert_eq!(r("speak", r#""optative""#, 1, false), "speak!");
        assert_eq!(
            r("speak", r#""optative""#, 2, true),
            "let him/her not speak"
        );
        assert_eq!(
            r(
                "sleep",
                r#"{"tense":"subjunctive","before":"when"}"#,
                2,
                false
            ),
            "when he/she sleep"
        );
    }

    #[test]
    fn constructions() {
        let s = |j: &str, p: usize| r("sleep", j, p, false);
        assert_eq!(
            s(r#"{"tense":"present","aux":"need","link":"to"}"#, 0),
            "I need to sleep"
        );
        assert_eq!(
            s(r#"{"tense":"present","aux":"need","link":"to"}"#, 2),
            "he/she needs to sleep"
        );
        assert_eq!(
            s(r#"{"tense":"progressive","aux":"go","link":"to"}"#, 0),
            "I am going to sleep"
        );
        assert_eq!(
            s(r#"{"tense":"present","aux":"be","link":"about to"}"#, 2),
            "he/she is about to sleep"
        );
        assert_eq!(
            s(r#"{"tense":"perfect","adverb":"just"}"#, 0),
            "I have just slept"
        );
        assert_eq!(
            s(r#"{"tense":"preterite","adverb":"almost"}"#, 0),
            "I almost slept"
        );
        assert_eq!(
            s(r#"{"tense":"present","before":"when"}"#, 3),
            "when we sleep"
        );
        assert_eq!(
            r(
                "sleep",
                r#"{"tense":"present","aux":"need","link":"to"}"#,
                0,
                true
            ),
            "I don't need to sleep"
        );
        assert_eq!(
            r("drink water", r#""preterite""#, 2, false),
            "he/she drank water"
        );
        assert_eq!(
            s(r#"{"tense":"present","aux":"can"}"#, 2),
            "he/she can sleep"
        );
        assert_eq!(
            s(r#"{"tense":"preterite","aux":"finish","link":"-ing"}"#, 0),
            "I finished sleeping"
        );
        assert_eq!(
            r(
                "fall",
                r#"{"tense":"present","aux":"might","before":"careful!"}"#,
                1,
                false
            ),
            "careful! you might fall"
        );
        assert_eq!(
            s(r#"{"tense":"present","before":"let's","pronoun":false}"#, 3),
            "let's sleep"
        );
        assert_eq!(
            s(r#"{"tense":"imperative","aux":"come","link":"and"}"#, 1),
            "come and sleep!"
        );
        assert_eq!(r("to see", r#""present""#, 0, false), "I see");
    }
}
