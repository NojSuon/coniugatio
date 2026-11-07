//! Morphology rules for Mezquital otomí (hñähñu).
//!
//! Source: Hernández Cruz, Victoria Torquemada & Sinclair Crawford,
//! *Diccionario del hñähñu (otomí) del Valle del Mezquital*, ILV, 2010 —
//! "Notas gramaticales" by Doris Bartholomew, §1 (secondary root forms) and
//! §12 (plural -hu̱).

/// Secondary form B (§1.1): the root's first consonant becomes voiced.
/// Used for the 3rd person of the secondary tenses (preterite, future).
pub fn form_b(root: &str) -> String {
    // Glottalised stops do not change; check them before their plain forms.
    const KEEP: [&str; 4] = ["ts'", "t'", "k'", "ch"];
    if KEEP.iter().any(|p| root.starts_with(p)) {
        return root.to_string();
    }
    const MAP: [(&str, &str); 8] = [
        ("th", "d"),
        ("ts", "z"),
        ("'b", "'m"),
        ("t", "d"),
        ("p", "b"),
        ("f", "m"),
        ("k", "g"),
        ("j", "g"),
    ];
    for (from, to) in MAP {
        if let Some(rest) = root.strip_prefix(from) {
            return format!("{to}{rest}");
        }
    }
    root.to_string()
}

fn starts_with_vowel(s: &str) -> bool {
    s.chars().next().is_some_and(|c| "aäeëiïoöuü".contains(c))
}

/// Secondary form Y (§1.2): roots that start with saltillo or *h* plus a vowel
/// insert *y* (oral vowel) or *ñ* (nasal vowel). Returns `None` if the rule
/// does not apply. Only *ä* is written as nasal, so other nasal vowels need a
/// lexicon entry.
pub fn form_y(root: &str) -> Option<String> {
    let (lead, rest) = if let Some(r) = root.strip_prefix('\'') {
        ("'", r)
    } else if let Some(r) = root.strip_prefix('h') {
        ("h", r)
    } else {
        ("'", root) // a written initial vowel carries an unwritten saltillo
    };
    if !starts_with_vowel(rest) {
        return None;
    }
    // "hu" + vowel is the semivowel [w], not a vowel-initial root.
    if lead == "h" && rest.starts_with('u') && rest.chars().nth(1).is_some_and(|c| "aäeio".contains(c)) {
        return None;
    }
    let glide = if rest.starts_with(['ä', 'ë', 'ï', 'ü']) { "ñ" } else { "y" };
    Some(format!("{lead}{glide}{rest}"))
}

/// Secondary form N (§1.3): an *n-* (or *m-* before a labial) on every person.
pub fn form_n(root: &str) -> String {
    let labial = root.starts_with(['p', 'b', 'f', 'm']) || root.starts_with("'b");
    format!("{}{root}", if labial { "m" } else { "n" })
}

/// Number of vowel groups — a rough syllable count.
fn syllables(s: &str) -> usize {
    let mut count = 0;
    let mut prev_vowel = false;
    for c in s.chars() {
        let v = "aäeëiïoöuü".contains(c);
        if v && !prev_vowel {
            count += 1;
        }
        prev_vowel = v;
    }
    count
}

/// Plural with -hu̱ in everyday speech (§12).
pub fn plural(stem: &str) -> String {
    const HU: &str = "hu\u{0331}";
    // Strip a trailing combining mark so endings like "-te̱" still match.
    let base = stem.trim_end_matches('\u{0331}');
    let drop_suffix = |n: usize| base[..base.len() - n].to_string();
    // Thematic -i (from *-mi) after a root vowel becomes ñ: tai → tañhu̱.
    if ["ai", "äi", "oi", "ei"].iter().any(|e| base.ends_with(e)) {
        return format!("{}ñ{HU}", drop_suffix(1));
    }
    if syllables(base) <= 1 {
        return format!("{stem}{HU}");
    }
    for ending in ["ndi", "nde"] {
        if base.ends_with(ending) {
            return format!("{}nt{HU}", drop_suffix(ending.len()));
        }
    }
    for ending in ["di", "de", "gi", "ge", "fi", "fe", "hi", "he"] {
        if base.ends_with(ending) {
            return format!("{}{HU}", drop_suffix(ending.len()));
        }
    }
    for ending in ["ts'i", "ts'e"] {
        if base.ends_with(ending) {
            return format!("{}x{HU}", drop_suffix(ending.len()));
        }
    }
    // -ti, -t'i, -'ti, -'tsi and -pi lose their vowel: the̱ti → the̱thu̱.
    for ending in ["ti", "te", "pi", "pe", "tsi", "tse"] {
        if base.ends_with(ending) {
            return format!("{}{HU}", drop_suffix(1));
        }
    }
    // -ni and -i (from *-mi) become ñ: beni → beñhu̱, tai → tañhu̱.
    if base.ends_with("ni") {
        return format!("{}ñ{HU}", drop_suffix(2));
    }
    format!("{stem}{HU}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b_form_examples_from_dictionary() {
        assert_eq!(form_b("pädi"), "bädi");
        assert_eq!(form_b("tai"), "dai");
        assert_eq!(form_b("thädi"), "dädi");
        assert_eq!(form_b("tsa"), "za");
        assert_eq!(form_b("ku̱ni"), "gu̱ni");
        assert_eq!(form_b("ju̱ki"), "gu̱ki");
        assert_eq!(form_b("juadi"), "guadi");
        assert_eq!(form_b("fu̱di"), "mu̱di");
        assert_eq!(form_b("'bai"), "'mai");
        assert_eq!(form_b("nei"), "nei");
        assert_eq!(form_b("xofo"), "xofo");
        assert_eq!(form_b("k'eñä"), "k'eñä");
    }

    #[test]
    fn y_form_examples_from_dictionary() {
        assert_eq!(form_y("honi").as_deref(), Some("hyoni"));
        assert_eq!(form_y("häts'i").as_deref(), Some("hñäts'i"));
        assert_eq!(form_y("'o̱de").as_deref(), Some("'yo̱de"));
        assert_eq!(form_y("'ähä").as_deref(), Some("'ñähä"));
        assert_eq!(form_y("handi").as_deref(), Some("hyandi"));
        assert_eq!(form_y("tai"), None);
        assert_eq!(form_y("huahni"), None);
    }

    #[test]
    fn n_form() {
        assert_eq!(form_n("sagi"), "nsagi");
        assert_eq!(form_n("peni"), "mpeni");
    }

    #[test]
    fn plural_examples_from_dictionary() {
        assert_eq!(plural("tsi"), "tsihu̱");
        assert_eq!(plural("tege"), "tehu̱");
        assert_eq!(plural("pe̱fi"), "pe̱hu̱");
        assert_eq!(plural("kohi"), "kohu̱");
        assert_eq!(plural("the̱ti"), "the̱thu̱");
        assert_eq!(plural("handi"), "hanthu̱");
        assert_eq!(plural("japi"), "japhu̱");
        assert_eq!(plural("beni"), "beñhu̱");
        assert_eq!(plural("tai"), "tañhu̱");
        assert_eq!(plural("'o̱de"), "'o̱hu̱");
    }
}
