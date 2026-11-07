//! A small Spanish conjugator, used to show the Spanish equivalent next to
//! each conjugated form ("dá pädi" → "yo supe").
//!
//! Regular -ar/-er/-ir verbs follow the rules; irregular verbs used in the
//! lexicons are listed in `irregular`. A meaning like "beber agua" conjugates
//! the first word and keeps the rest; "reírse" is conjugated as reflexive.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tense {
    Present,
    Preterite,
    Future,
    Imperfect,
    Progressive,
    /// he visto, has dormido…
    Perfect,
}

/// yo, tú, él/ella, nosotros, ustedes, ellos/ellas
pub const PRONOUNS: [&str; 6] = ["yo", "tú", "él/ella", "nosotros", "ustedes", "ellos/ellas"];
const REFLEXIVE: [&str; 6] = ["me", "te", "se", "nos", "se", "se"];
const ESTAR: [&str; 6] = ["estoy", "estás", "está", "estamos", "están", "están"];
const HABER: [&str; 6] = ["he", "has", "ha", "hemos", "han", "han"];

type Six = [&'static str; 6];

/// Full forms for irregular verbs. `None` means "use the regular rule".
struct Irregular {
    present: Option<Six>,
    preterite: Option<Six>,
    imperfect: Option<Six>,
    future_stem: Option<&'static str>,
    gerund: Option<&'static str>,
    participle: Option<&'static str>,
}

const NONE: Irregular =
    Irregular { present: None, preterite: None, imperfect: None, future_stem: None, gerund: None, participle: None };

fn irregular(inf: &str) -> Irregular {
    match inf {
        "saber" => Irregular {
            present: Some(["sé", "sabes", "sabe", "sabemos", "saben", "saben"]),
            preterite: Some(["supe", "supiste", "supo", "supimos", "supieron", "supieron"]),
            future_stem: Some("sabr"),
            ..NONE
        },
        "ver" => Irregular {
            present: Some(["veo", "ves", "ve", "vemos", "ven", "ven"]),
            preterite: Some(["vi", "viste", "vio", "vimos", "vieron", "vieron"]),
            imperfect: Some(["veía", "veías", "veía", "veíamos", "veían", "veían"]),
            participle: Some("visto"),
            ..NONE
        },
        "dormir" => Irregular {
            present: Some(["duermo", "duermes", "duerme", "dormimos", "duermen", "duermen"]),
            preterite: Some(["dormí", "dormiste", "durmió", "dormimos", "durmieron", "durmieron"]),
            gerund: Some("durmiendo"),
            ..NONE
        },
        "oír" => Irregular {
            present: Some(["oigo", "oyes", "oye", "oímos", "oyen", "oyen"]),
            preterite: Some(["oí", "oíste", "oyó", "oímos", "oyeron", "oyeron"]),
            imperfect: Some(["oía", "oías", "oía", "oíamos", "oían", "oían"]),
            future_stem: Some("oir"),
            gerund: Some("oyendo"),
            participle: Some("oído"),
        },
        "moler" => Irregular {
            present: Some(["muelo", "mueles", "muele", "molemos", "muelen", "muelen"]),
            ..NONE
        },
        "reír" => Irregular {
            present: Some(["río", "ríes", "ríe", "reímos", "ríen", "ríen"]),
            preterite: Some(["reí", "reíste", "rio", "reímos", "rieron", "rieron"]),
            imperfect: Some(["reía", "reías", "reía", "reíamos", "reían", "reían"]),
            future_stem: Some("reir"),
            gerund: Some("riendo"),
            participle: Some("reído"),
        },
        "ir" => Irregular {
            present: Some(["voy", "vas", "va", "vamos", "van", "van"]),
            preterite: Some(["fui", "fuiste", "fue", "fuimos", "fueron", "fueron"]),
            imperfect: Some(["iba", "ibas", "iba", "íbamos", "iban", "iban"]),
            gerund: Some("yendo"),
            ..NONE
        },
        "venir" => Irregular {
            present: Some(["vengo", "vienes", "viene", "venimos", "vienen", "vienen"]),
            preterite: Some(["vine", "viniste", "vino", "vinimos", "vinieron", "vinieron"]),
            future_stem: Some("vendr"),
            gerund: Some("viniendo"),
            ..NONE
        },
        "salir" => Irregular {
            present: Some(["salgo", "sales", "sale", "salimos", "salen", "salen"]),
            future_stem: Some("saldr"),
            ..NONE
        },
        "morir" => Irregular {
            present: Some(["muero", "mueres", "muere", "morimos", "mueren", "mueren"]),
            preterite: Some(["morí", "moriste", "murió", "morimos", "murieron", "murieron"]),
            gerund: Some("muriendo"),
            participle: Some("muerto"),
            ..NONE
        },
        "caer" => Irregular {
            present: Some(["caigo", "caes", "cae", "caemos", "caen", "caen"]),
            preterite: Some(["caí", "caíste", "cayó", "caímos", "cayeron", "cayeron"]),
            gerund: Some("cayendo"),
            participle: Some("caído"),
            ..NONE
        },
        _ => NONE,
    }
}

fn regular(inf: &str, tense: Tense, p: usize) -> String {
    let (stem, class) = inf.split_at(inf.len() - 2);
    let endings: Six = match (tense, class) {
        (Tense::Present, "ar") => ["o", "as", "a", "amos", "an", "an"],
        (Tense::Present, "er") => ["o", "es", "e", "emos", "en", "en"],
        (Tense::Present, _) => ["o", "es", "e", "imos", "en", "en"],
        (Tense::Preterite, "ar") => ["é", "aste", "ó", "amos", "aron", "aron"],
        (Tense::Preterite, _) => ["í", "iste", "ió", "imos", "ieron", "ieron"],
        (Tense::Imperfect, "ar") => ["aba", "abas", "aba", "ábamos", "aban", "aban"],
        (Tense::Imperfect, _) => ["ía", "ías", "ía", "íamos", "ían", "ían"],
        (Tense::Future, _) => return format!("{inf}{}", ["é", "ás", "á", "emos", "án", "án"][p]),
        (Tense::Progressive | Tense::Perfect, _) => unreachable!(),
    };
    // Spelling changes before -é: buscar → busqué, pagar → pagué, cazar → cacé.
    if tense == Tense::Preterite && p == 0 && class == "ar" {
        for (end, repl) in [("c", "qu"), ("g", "gu"), ("z", "c")] {
            if let Some(s) = stem.strip_suffix(end) {
                return format!("{s}{repl}é");
            }
        }
    }
    format!("{stem}{}", endings[p])
}

fn participle(inf: &str, irr: &Irregular) -> String {
    if let Some(p) = irr.participle {
        return p.to_string();
    }
    let (stem, class) = inf.split_at(inf.len() - 2);
    format!("{stem}{}", if class == "ar" { "ado" } else { "ido" })
}

fn gerund(inf: &str, irr: &Irregular) -> String {
    if let Some(g) = irr.gerund {
        return g.to_string();
    }
    let (stem, class) = inf.split_at(inf.len() - 2);
    format!("{stem}{}", if class == "ar" { "ando" } else { "iendo" })
}

/// Conjugates a Spanish meaning such as "comer", "reírse" or "beber agua".
/// `person` is 0..6 in the order of [`PRONOUNS`]. Returns `None` when the
/// first word is not an infinitive.
pub fn conjugate(meaning: &str, tense: Tense, person: usize, negative: bool) -> Option<String> {
    let mut words = meaning.split_whitespace();
    let first = words.next()?;
    let rest: Vec<&str> = words.collect();

    let (inf, reflexive) = match first.strip_suffix("se") {
        Some(base) if base.ends_with(['r']) => (base, true),
        _ => (first, false),
    };
    if !(inf.ends_with("ar") || inf.ends_with("er") || inf.ends_with("ir") || inf.ends_with("ír")) {
        return None;
    }
    let inf_plain = inf.replace('í', "i");
    let irr = irregular(inf);

    let verb = match tense {
        Tense::Present => irr.present.map(|f| f[person].to_string()),
        Tense::Preterite => irr.preterite.map(|f| f[person].to_string()),
        Tense::Imperfect => irr.imperfect.map(|f| f[person].to_string()),
        Tense::Future => irr
            .future_stem
            .map(|s| format!("{s}{}", ["é", "ás", "á", "emos", "án", "án"][person])),
        Tense::Progressive => Some(format!("{} {}", ESTAR[person], gerund(&inf_plain, &irr))),
        Tense::Perfect => Some(format!("{} {}", HABER[person], participle(&inf_plain, &irr))),
    }
    .unwrap_or_else(|| regular(&inf_plain, tense, person));

    let mut parts = vec![PRONOUNS[person].to_string()];
    if negative {
        parts.push("no".into());
    }
    if reflexive {
        parts.push(REFLEXIVE[person].into());
    }
    parts.push(verb);
    parts.extend(rest.iter().map(|w| w.to_string()));
    Some(parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(m: &str, t: Tense, p: usize) -> String {
        conjugate(m, t, p, false).unwrap()
    }

    #[test]
    fn regular_verbs() {
        assert_eq!(c("comer", Tense::Present, 0), "yo como");
        assert_eq!(c("comprar", Tense::Preterite, 2), "él/ella compró");
        assert_eq!(c("partir", Tense::Present, 3), "nosotros partimos");
        assert_eq!(c("lavar", Tense::Imperfect, 4), "ustedes lavaban");
        assert_eq!(c("bailar", Tense::Future, 1), "tú bailarás");
        assert_eq!(c("cosechar", Tense::Progressive, 0), "yo estoy cosechando");
        assert_eq!(c("comer", Tense::Progressive, 2), "él/ella está comiendo");
    }

    #[test]
    fn spelling_changes() {
        assert_eq!(c("buscar", Tense::Preterite, 0), "yo busqué");
        assert_eq!(c("brincar", Tense::Preterite, 0), "yo brinqué");
        assert_eq!(c("sacar", Tense::Preterite, 1), "tú sacaste");
    }

    #[test]
    fn irregular_verbs() {
        assert_eq!(c("saber", Tense::Present, 0), "yo sé");
        assert_eq!(c("saber", Tense::Preterite, 2), "él/ella supo");
        assert_eq!(c("saber", Tense::Future, 0), "yo sabré");
        assert_eq!(c("ver", Tense::Preterite, 0), "yo vi");
        assert_eq!(c("dormir", Tense::Progressive, 1), "tú estás durmiendo");
        assert_eq!(c("oír", Tense::Present, 0), "yo oigo");
        assert_eq!(c("oír", Tense::Future, 3), "nosotros oiremos");
        assert_eq!(c("moler", Tense::Present, 2), "él/ella muele");
    }

    #[test]
    fn reflexive_complement_and_negative() {
        assert_eq!(c("reírse", Tense::Preterite, 2), "él/ella se rio");
        assert_eq!(c("reírse", Tense::Future, 0), "yo me reiré");
        assert_eq!(c("beber agua", Tense::Present, 3), "nosotros bebemos agua");
        assert_eq!(conjugate("saber", Tense::Present, 0, true).unwrap(), "yo no sé");
        assert_eq!(conjugate("reírse", Tense::Present, 1, true).unwrap(), "tú no te ríes");
    }

    #[test]
    fn perfect_and_new_irregulars() {
        assert_eq!(c("ver", Tense::Perfect, 0), "yo he visto");
        assert_eq!(c("dormir", Tense::Perfect, 3), "nosotros hemos dormido");
        assert_eq!(c("morir", Tense::Perfect, 2), "él/ella ha muerto");
        assert_eq!(c("ir", Tense::Preterite, 0), "yo fui");
        assert_eq!(c("ir", Tense::Progressive, 1), "tú estás yendo");
        assert_eq!(c("ir", Tense::Future, 0), "yo iré");
        assert_eq!(c("venir", Tense::Present, 0), "yo vengo");
        assert_eq!(c("venir", Tense::Future, 2), "él/ella vendrá");
        assert_eq!(c("salir", Tense::Present, 0), "yo salgo");
        assert_eq!(c("caer", Tense::Preterite, 2), "él/ella cayó");
        assert_eq!(c("llegar", Tense::Preterite, 0), "yo llegué");
        assert_eq!(conjugate("ver", Tense::Perfect, 0, true).unwrap(), "yo no he visto");
    }

    #[test]
    fn not_a_verb() {
        assert_eq!(conjugate("agua", Tense::Present, 0, false), None);
    }
}
