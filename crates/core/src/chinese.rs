//! Chinese equivalents for the Chinese pages (/zh/).
//!
//! Chinese verbs don't change with the person: the tense is said with
//! aspect words (了, 在, 会, 已经) and modal verbs (得, 想, 能). The pages
//! keep each tense's Spanish spec (`spanish::Es`), and this module reads
//! its auxiliaries and adverbs ("tener que", "ya") as their Chinese
//! counterparts, so the data only has to give each verb's meaning in
//! Chinese: "睡觉", "去", "看它" (a final 它 or 这件事 is the object, so 了 goes
//! before it: 我看了它).

use crate::spanish::{Es, Spec, Tense};

pub const PRONOUNS: [&str; 6] = ["我", "你", "他/她", "我们", "你们", "他们/她们"];

/// A Spanish auxiliary (with its link) as a Chinese word before the verb,
/// and its negative form.
fn modal(aux: &str, link: Option<&str>) -> Option<(&'static str, &'static str)> {
    Some(match (aux, link) {
        ("tener", Some("que")) => ("得", "不用"),
        ("necesitar", _) => ("需要", "不需要"),
        ("querer", _) => ("想", "不想"),
        ("poder", _) => ("能", "不能"),
        ("soler", _) => ("常常", "不常"),
        ("empezar" | "comenzar", _) => ("开始", "没开始"),
        ("acabar", Some("de")) => ("刚", "不是刚"),
        ("ir", Some("a")) => ("要", "不要"),
        ("ir", Some("-ndo")) => ("渐渐地", "没有渐渐地"),
        ("venir", Some("a")) => ("来", "不来"),
        ("estar", Some("a punto de")) => ("快要", "还不会"),
        // terminar de: 睡完 (handled as a result complement below).
        ("terminar", Some("de")) => ("", ""),
        _ => return None,
    })
}

fn adverb(word: &str) -> Option<&'static str> {
    Some(match word {
        "ya" => "已经",
        "casi" => "差点",
        "sí" => "一定",
        _ => return None,
    })
}

/// Words before the subject: (before the sentence, after it).
fn frame(word: &str) -> Option<(&'static str, &'static str)> {
    Some(match word {
        "que" => ("让", ""),
        "cuando" => ("", "的时候"),
        "aunque" => ("即使", ""),
        "si" => ("如果", ""),
        "hace rato que" => ("刚才", ""),
        "hace mucho tiempo que" => ("很久以前", ""),
        "algún día" => ("有一天", ""),
        "¡cuidado!" => ("小心！", ""),
        _ => return None,
    })
}

fn after(word: &str) -> Option<&'static str> {
    Some(match word {
        "allá" => "在那边",
        _ => return None,
    })
}

fn is_chinese(meaning: &str) -> bool {
    !meaning.is_empty()
        && meaning
            .chars()
            .all(|c| matches!(c as u32, 0x3400..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x2FFFF))
}

pub fn render(meaning: &str, es: &Es, person: usize, negative: bool) -> Option<String> {
    let spec = match es {
        Es::Simple(t) => Spec {
            tense: *t,
            aux: None,
            link: None,
            before: None,
            adverb: None,
            pronoun: true,
            negative: false,
            question: false,
            after: None,
        },
        Es::Spec(s) => s.clone(),
    };
    let negative = negative || spec.negative;
    let meaning = meaning.trim();
    if !is_chinese(meaning) || person >= PRONOUNS.len() {
        return None;
    }
    // The object, so 了 goes before it: 看了它, 做了这件事.
    let (verb, object) = ["这件事", "它"]
        .iter()
        .find_map(|o| meaning.strip_suffix(o).filter(|v| !v.is_empty()).map(|v| (v, *o)))
        .unwrap_or((meaning, ""));
    let second = person == 1 || person == 4;
    let place = match spec.after.as_deref() {
        Some(a) => after(a)?,
        None => "",
    };
    let (open, close) = match spec.before.as_deref() {
        Some(b) => frame(b)?,
        None => ("", ""),
    };
    let adv = match spec.adverb.as_deref() {
        Some(a) => adverb(a)?,
        None => "",
    };
    let finish_word = spec.aux.as_deref() == Some("terminar");
    let modal = match spec.aux.as_deref() {
        Some(a) => Some(modal(a, spec.link.as_deref())?),
        None => None,
    };
    let subject = if spec.pronoun { PRONOUNS[person] } else { "" };
    // 睡完, 看完它
    let v = if finish_word { format!("{verb}完") } else { verb.to_string() };
    // The verb with 了 (and its object): 去了, 看了它.
    let done = format!("{v}了{object}");
    let plain = format!("{v}{object}");

    let mut tense = spec.tense;
    // «fui a comer» is 我去吃了, not 我要吃了.
    let modal = match (modal, tense) {
        (Some(("要", _)), Tense::Preterite) => Some(("去", "没去")),
        (m, _) => m,
    };
    if tense == Tense::Optative && second {
        tense = Tense::Imperative;
    }

    let body: String = match tense {
        Tense::Imperative => {
            if !second {
                return None;
            }
            let who = if person == 4 { "你们" } else { "" };
            let m = modal.map(|(m, _)| m).unwrap_or("");
            let neg = if negative { "别" } else { "" };
            return Some(format!("{open}{who}{neg}{place}{m}{plain}！"));
        }
        Tense::Optative => {
            let neg = if negative { "不要" } else { "" };
            format!("让{subject}{neg}{place}{plain}吧")
        }
        _ => {
            // The words between the subject and the verb, and the verb.
            let (pre, main): (String, String) = match (modal, tense) {
                // 可能会: the conditional of poder («podría»).
                (Some(("能", _)), Tense::Conditional) => {
                    (if negative { "可能不会".into() } else { "可能会".into() }, plain.clone())
                }
                (Some((m, n)), _) => {
                    let m = if negative { n } else { m };
                    let main = match tense {
                        Tense::Preterite | Tense::Perfect | Tense::Pluperfect if !negative && !m.is_empty() => {
                            done.clone()
                        }
                        // 刚去了, 快要去了
                        _ if !negative && (m == "刚" || m == "快要") => done.clone(),
                        _ if finish_word && !negative && tense == Tense::Preterite => done.clone(),
                        _ if finish_word && negative => plain.clone(),
                        _ => plain.clone(),
                    };
                    let m = if finish_word {
                        if negative && tense == Tense::Preterite { "没" } else if negative { "不" } else { "" }
                    } else {
                        m
                    };
                    let time = match tense {
                        Tense::Future => "会",
                        Tense::Imperfect | Tense::PastProgressive => "那时",
                        Tense::Conditional => "就会",
                        _ => "",
                    };
                    (format!("{time}{m}"), main)
                }
                (None, _) => {
                    let (pre, main) = match (tense, negative) {
                        (Tense::Present | Tense::Subjunctive, false) => ("", plain.clone()),
                        (Tense::Present | Tense::Subjunctive, true) => ("不", plain.clone()),
                        (Tense::Preterite, false) => ("", done.clone()),
                        (Tense::Preterite, true) => ("没", plain.clone()),
                        (Tense::PastSubjunctive, false) => ("", plain.clone()),
                        (Tense::PastSubjunctive, true) => ("不", plain.clone()),
                        (Tense::Perfect, false) => ("已经", done.clone()),
                        (Tense::Perfect, true) => ("还没", plain.clone()),
                        (Tense::Pluperfect, false) => ("之前已经", done.clone()),
                        (Tense::Pluperfect, true) => ("之前还没", plain.clone()),
                        (Tense::Future, false) => ("会", plain.clone()),
                        (Tense::Future, true) => ("不会", plain.clone()),
                        (Tense::Conditional, false) => ("就会", plain.clone()),
                        (Tense::Conditional, true) => ("就不会", plain.clone()),
                        (Tense::Imperfect, false) => ("以前常", plain.clone()),
                        (Tense::Imperfect, true) => ("以前不", plain.clone()),
                        (Tense::Progressive, false) => ("正在", plain.clone()),
                        (Tense::Progressive, true) => ("没在", plain.clone()),
                        (Tense::PastProgressive, false) => ("那时正在", plain.clone()),
                        (Tense::PastProgressive, true) => ("那时没在", plain.clone()),
                        _ => return None,
                    };
                    (pre.to_string(), main)
                }
            };
            // 已经 + 了, 差点 + 了, 一定会
            let (adv, main) = match adv {
                "一定" => ("一定", main),
                "" => ("", main),
                a if !negative && tense == Tense::Preterite => (a, done.clone()),
                a => (a, main),
            };
            let pre = if adv == "一定" && pre == "会" { "会".to_string() } else { pre };
            format!("{subject}{adv}{pre}{place}{main}")
        }
    };
    let mut text = format!("{open}{body}{close}");
    if spec.question {
        text.push_str("吗？");
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple(t: Tense) -> Es {
        Es::Simple(t)
    }

    fn spec(json: &str) -> Es {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn tenses() {
        assert_eq!(render("去", &simple(Tense::Present), 0, false).unwrap(), "我去");
        assert_eq!(render("去", &simple(Tense::Present), 2, true).unwrap(), "他/她不去");
        assert_eq!(render("去", &simple(Tense::Preterite), 3, false).unwrap(), "我们去了");
        assert_eq!(render("去", &simple(Tense::Preterite), 3, true).unwrap(), "我们没去");
        assert_eq!(render("看它", &simple(Tense::Preterite), 0, false).unwrap(), "我看了它");
        assert_eq!(render("做这件事", &simple(Tense::Preterite), 0, false).unwrap(), "我做了这件事");
        assert_eq!(render("睡觉", &simple(Tense::Progressive), 1, false).unwrap(), "你正在睡觉");
        assert_eq!(render("睡觉", &simple(Tense::Future), 5, false).unwrap(), "他们/她们会睡觉");
        assert_eq!(render("睡觉", &simple(Tense::Imperative), 1, false).unwrap(), "睡觉！");
        assert_eq!(render("睡觉", &simple(Tense::Imperative), 4, true).unwrap(), "你们别睡觉！");
        assert_eq!(render("睡觉", &simple(Tense::Imperative), 0, false), None);
        assert_eq!(render("睡觉", &simple(Tense::Optative), 0, false).unwrap(), "让我睡觉吧");
        assert_eq!(render("sleep", &simple(Tense::Present), 0, false), None);
    }

    #[test]
    fn specs() {
        let s = |j: &str, p| render("吃它", &spec(j), p, false).unwrap();
        assert_eq!(s(r#"{"tense":"present","aux":"tener","link":"que"}"#, 0), "我得吃它");
        assert_eq!(s(r#"{"tense":"present","aux":"acabar","link":"de"}"#, 0), "我刚吃了它");
        assert_eq!(s(r#"{"tense":"present","aux":"querer"}"#, 2), "他/她想吃它");
        assert_eq!(s(r#"{"tense":"preterite","adverb":"ya"}"#, 0), "我已经吃了它");
        assert_eq!(s(r#"{"tense":"future","adverb":"sí"}"#, 0), "我一定会吃它");
        assert_eq!(s(r#"{"tense":"subjunctive","before":"que"}"#, 1), "让你吃它");
        assert_eq!(s(r#"{"tense":"preterite","aux":"terminar","link":"de"}"#, 0), "我吃完了它");
        assert_eq!(s(r#"{"tense":"present","question":true}"#, 1), "你吃它吗？");
        assert_eq!(s(r#"{"tense":"preterite","aux":"ir","link":"a"}"#, 0), "我去吃了它");
        assert_eq!(s(r#"{"tense":"present","after":"allá"}"#, 0), "我在那边吃它");
        assert_eq!(s(r#"{"tense":"present","before":"cuando"}"#, 0), "我吃它的时候");
        assert_eq!(
            render("吃它", &spec(r#"{"tense":"present","aux":"tener","link":"que"}"#), 0, true).unwrap(),
            "我不用吃它"
        );
    }
}
