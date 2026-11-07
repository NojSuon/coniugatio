//! A small Spanish conjugator, used to show the Spanish equivalent next to
//! each conjugated form ("dá pädi" → "yo supe", "k'ab'et inwenel" →
//! "yo necesito dormir").
//!
//! Regular -ar/-er/-ir verbs follow the rules; irregular verbs used in the
//! lexicons and as auxiliaries are listed in `irregular`. A meaning like
//! "beber agua" conjugates the first word and keeps the rest; "reírse" is
//! conjugated as reflexive.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tense {
    Present,
    Preterite,
    Future,
    Imperfect,
    /// estoy durmiendo
    Progressive,
    /// estaba durmiendo
    #[serde(rename = "past_progressive")]
    PastProgressive,
    /// he visto, has dormido…
    Perfect,
    /// (que) yo duerma
    Subjunctive,
    /// ¡duerme!, ¡duerman! (2nd person only)
    Imperative,
    /// yo podría
    Conditional,
    /// ¡hazlo! (tú, ustedes) / que yo lo haga (other persons): the Nahuatl
    /// optative mood.
    Optative,
}

/// How a tense is said in Spanish. Either just a tense ("present"), or a
/// construction:
///
/// ```json
/// { "tense": "present", "aux": "tener", "link": "que" }   // tengo que dormir
/// { "tense": "present", "before": "cuando" }             // cuando yo duermo
/// { "tense": "preterite", "adverb": "casi" }             // yo casi dormí
/// { "tense": "present", "aux": "ir", "link": "a", "pronoun": false } // vamos a dormir
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Es {
    Simple(Tense),
    Spec(Spec),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    pub tense: Tense,
    /// Auxiliary verb, conjugated; the meaning follows as an infinitive.
    #[serde(default)]
    pub aux: Option<String>,
    /// Word between the auxiliary and the infinitive: "a", "de", "que".
    #[serde(default)]
    pub link: Option<String>,
    /// Words before the subject pronoun: "cuando", "que".
    #[serde(default)]
    pub before: Option<String>,
    /// Adverb after the subject pronoun: "casi", "ya".
    #[serde(default)]
    pub adverb: Option<String>,
    /// Show the subject pronoun (yo, tú…). Default: yes.
    #[serde(default = "yes")]
    pub pronoun: bool,
    /// Always negative in Spanish ("que no…"), even if the table isn't.
    #[serde(default)]
    pub negative: bool,
    /// Said as a question: ¿él canta?
    #[serde(default)]
    pub question: bool,
}

fn yes() -> bool {
    true
}

impl Es {
    fn spec(&self) -> Spec {
        match self {
            Es::Simple(tense) => Spec {
                tense: *tense,
                aux: None,
                link: None,
                before: None,
                adverb: None,
                pronoun: true,
                negative: false,
                question: false,
            },
            Es::Spec(s) => s.clone(),
        }
    }
}

/// yo, tú, él/ella, nosotros, ustedes, ellos/ellas
pub const PRONOUNS: [&str; 6] = ["yo", "tú", "él/ella", "nosotros", "ustedes", "ellos/ellas"];
const REFLEXIVE: [&str; 6] = ["me", "te", "se", "nos", "se", "se"];
const ESTAR: [&str; 6] = ["estoy", "estás", "está", "estamos", "están", "están"];
const ESTABA: [&str; 6] = [
    "estaba",
    "estabas",
    "estaba",
    "estábamos",
    "estaban",
    "estaban",
];
const HABER: [&str; 6] = ["he", "has", "ha", "hemos", "han", "han"];

type Six = [&'static str; 6];

/// Full forms for irregular verbs. `None` means "use the regular rule".
struct Irregular {
    present: Option<Six>,
    preterite: Option<Six>,
    imperfect: Option<Six>,
    subjunctive: Option<Six>,
    future_stem: Option<&'static str>,
    gerund: Option<&'static str>,
    participle: Option<&'static str>,
    /// Affirmative "tú" imperative: sal, ven, ve.
    imperative: Option<&'static str>,
}

const NONE: Irregular = Irregular {
    present: None,
    preterite: None,
    imperfect: None,
    subjunctive: None,
    future_stem: None,
    gerund: None,
    participle: None,
    imperative: None,
};

fn irregular(inf: &str) -> Irregular {
    match inf {
        "saber" => Irregular {
            present: Some(["sé", "sabes", "sabe", "sabemos", "saben", "saben"]),
            preterite: Some(["supe", "supiste", "supo", "supimos", "supieron", "supieron"]),
            subjunctive: Some(["sepa", "sepas", "sepa", "sepamos", "sepan", "sepan"]),
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
            present: Some([
                "duermo", "duermes", "duerme", "dormimos", "duermen", "duermen",
            ]),
            preterite: Some([
                "dormí",
                "dormiste",
                "durmió",
                "dormimos",
                "durmieron",
                "durmieron",
            ]),
            subjunctive: Some([
                "duerma", "duermas", "duerma", "durmamos", "duerman", "duerman",
            ]),
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
            ..NONE
        },
        "nacer" => Irregular {
            present: Some(["nazco", "naces", "nace", "nacemos", "nacen", "nacen"]),
            ..NONE
        },
        "despertar" => Irregular {
            present: Some([
                "despierto",
                "despiertas",
                "despierta",
                "despertamos",
                "despiertan",
                "despiertan",
            ]),
            ..NONE
        },
        "volver" => Irregular {
            present: Some([
                "vuelvo", "vuelves", "vuelve", "volvemos", "vuelven", "vuelven",
            ]),
            participle: Some("vuelto"),
            ..NONE
        },
        "sentar" => Irregular {
            present: Some([
                "siento", "sientas", "sienta", "sentamos", "sientan", "sientan",
            ]),
            ..NONE
        },
        "acostar" => Irregular {
            present: Some([
                "acuesto",
                "acuestas",
                "acuesta",
                "acostamos",
                "acuestan",
                "acuestan",
            ]),
            ..NONE
        },
        "traer" => Irregular {
            present: Some(["traigo", "traes", "trae", "traemos", "traen", "traen"]),
            preterite: Some([
                "traje", "trajiste", "trajo", "trajimos", "trajeron", "trajeron",
            ]),
            gerund: Some("trayendo"),
            participle: Some("traído"),
            ..NONE
        },
        "jugar" => Irregular {
            present: Some(["juego", "juegas", "juega", "jugamos", "juegan", "juegan"]),
            subjunctive: Some([
                "juegue", "juegues", "juegue", "juguemos", "jueguen", "jueguen",
            ]),
            ..NONE
        },
        "escribir" => Irregular {
            participle: Some("escrito"),
            ..NONE
        },
        "moler" => Irregular {
            present: Some(["muelo", "mueles", "muele", "molemos", "muelen", "muelen"]),
            ..NONE
        },
        "reír" => Irregular {
            present: Some(["río", "ríes", "ríe", "reímos", "ríen", "ríen"]),
            preterite: Some(["reí", "reíste", "rio", "reímos", "rieron", "rieron"]),
            imperfect: Some(["reía", "reías", "reía", "reíamos", "reían", "reían"]),
            subjunctive: Some(["ría", "rías", "ría", "riamos", "rían", "rían"]),
            future_stem: Some("reir"),
            gerund: Some("riendo"),
            participle: Some("reído"),
            ..NONE
        },
        "estar" => Irregular {
            present: Some(["estoy", "estás", "está", "estamos", "están", "están"]),
            preterite: Some([
                "estuve",
                "estuviste",
                "estuvo",
                "estuvimos",
                "estuvieron",
                "estuvieron",
            ]),
            subjunctive: Some(["esté", "estés", "esté", "estemos", "estén", "estén"]),
            imperative: Some("está"),
            ..NONE
        },
        "ir" => Irregular {
            present: Some(["voy", "vas", "va", "vamos", "van", "van"]),
            preterite: Some(["fui", "fuiste", "fue", "fuimos", "fueron", "fueron"]),
            imperfect: Some(["iba", "ibas", "iba", "íbamos", "iban", "iban"]),
            subjunctive: Some(["vaya", "vayas", "vaya", "vayamos", "vayan", "vayan"]),
            gerund: Some("yendo"),
            imperative: Some("ve"),
            ..NONE
        },
        "venir" => Irregular {
            present: Some(["vengo", "vienes", "viene", "venimos", "vienen", "vienen"]),
            preterite: Some(["vine", "viniste", "vino", "vinimos", "vinieron", "vinieron"]),
            future_stem: Some("vendr"),
            gerund: Some("viniendo"),
            imperative: Some("ven"),
            ..NONE
        },
        "salir" => Irregular {
            present: Some(["salgo", "sales", "sale", "salimos", "salen", "salen"]),
            future_stem: Some("saldr"),
            imperative: Some("sal"),
            ..NONE
        },
        "morir" => Irregular {
            present: Some(["muero", "mueres", "muere", "morimos", "mueren", "mueren"]),
            preterite: Some([
                "morí", "moriste", "murió", "morimos", "murieron", "murieron",
            ]),
            subjunctive: Some(["muera", "mueras", "muera", "muramos", "mueran", "mueran"]),
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
        "hacer" => Irregular {
            present: Some(["hago", "haces", "hace", "hacemos", "hacen", "hacen"]),
            preterite: Some(["hice", "hiciste", "hizo", "hicimos", "hicieron", "hicieron"]),
            future_stem: Some("har"),
            participle: Some("hecho"),
            imperative: Some("haz"),
            ..NONE
        },
        "decir" => Irregular {
            present: Some(["digo", "dices", "dice", "decimos", "dicen", "dicen"]),
            preterite: Some(["dije", "dijiste", "dijo", "dijimos", "dijeron", "dijeron"]),
            future_stem: Some("dir"),
            gerund: Some("diciendo"),
            participle: Some("dicho"),
            imperative: Some("di"),
            ..NONE
        },
        "dar" => Irregular {
            present: Some(["doy", "das", "da", "damos", "dan", "dan"]),
            preterite: Some(["di", "diste", "dio", "dimos", "dieron", "dieron"]),
            subjunctive: Some(["dé", "des", "dé", "demos", "den", "den"]),
            ..NONE
        },
        "conocer" => Irregular {
            present: Some([
                "conozco",
                "conoces",
                "conoce",
                "conocemos",
                "conocen",
                "conocen",
            ]),
            ..NONE
        },
        "leer" => Irregular {
            preterite: Some(["leí", "leíste", "leyó", "leímos", "leyeron", "leyeron"]),
            gerund: Some("leyendo"),
            participle: Some("leído"),
            ..NONE
        },
        // Auxiliaries used in periphrases.
        "poder" => Irregular {
            present: Some(["puedo", "puedes", "puede", "podemos", "pueden", "pueden"]),
            preterite: Some(["pude", "pudiste", "pudo", "pudimos", "pudieron", "pudieron"]),
            future_stem: Some("podr"),
            gerund: Some("pudiendo"),
            ..NONE
        },
        "tener" => Irregular {
            present: Some(["tengo", "tienes", "tiene", "tenemos", "tienen", "tienen"]),
            preterite: Some(["tuve", "tuviste", "tuvo", "tuvimos", "tuvieron", "tuvieron"]),
            future_stem: Some("tendr"),
            imperative: Some("ten"),
            ..NONE
        },
        "querer" => Irregular {
            present: Some([
                "quiero", "quieres", "quiere", "queremos", "quieren", "quieren",
            ]),
            preterite: Some([
                "quise",
                "quisiste",
                "quiso",
                "quisimos",
                "quisieron",
                "quisieron",
            ]),
            future_stem: Some("querr"),
            ..NONE
        },
        "soler" => Irregular {
            present: Some(["suelo", "sueles", "suele", "solemos", "suelen", "suelen"]),
            ..NONE
        },
        "empezar" => Irregular {
            present: Some([
                "empiezo",
                "empiezas",
                "empieza",
                "empezamos",
                "empiezan",
                "empiezan",
            ]),
            ..NONE
        },
        "comenzar" => Irregular {
            present: Some([
                "comienzo",
                "comienzas",
                "comienza",
                "comenzamos",
                "comienzan",
                "comienzan",
            ]),
            ..NONE
        },
        _ => NONE,
    }
}

const FUTURE: Six = ["é", "ás", "á", "emos", "án", "án"];
const CONDITIONAL: Six = ["ía", "ías", "ía", "íamos", "ían", "ían"];

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
        (Tense::Future, _) => return format!("{inf}{}", FUTURE[p]),
        (Tense::Conditional, _) => return format!("{inf}{}", CONDITIONAL[p]),
        _ => unreachable!("composite tense"),
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

/// Present subjunctive, built from the "yo" form of the present:
/// salgo → salga, busco → busque, muelo → muela / molamos.
fn subjunctive(inf: &str, irr: &Irregular, p: usize) -> String {
    if let Some(f) = irr.subjunctive {
        return f[p].to_string();
    }
    let (inf_stem, class) = inf.split_at(inf.len() - 2);
    let yo = irr
        .present
        .map(|f| f[0].to_string())
        .unwrap_or_else(|| regular(inf, Tense::Present, 0));
    let mut stem = yo.strip_suffix('o').unwrap_or(&yo).to_string();
    // Stem-changing verbs keep the infinitive stem for "nosotros": molamos.
    let diphthong = |s: &str| s.contains("ue") || s.contains("ie");
    if p == 3 && diphthong(&stem) && !diphthong(inf_stem) {
        stem = inf_stem.to_string();
    }
    let endings: Six = if class == "ar" {
        ["e", "es", "e", "emos", "en", "en"]
    } else {
        ["a", "as", "a", "amos", "an", "an"]
    };
    if class == "ar" {
        for (end, repl) in [("c", "qu"), ("g", "gu"), ("z", "c")] {
            if let Some(s) = stem.strip_suffix(end) {
                return format!("{s}{repl}{}", endings[p]);
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

/// One finite form of an infinitive (no pronouns). Imperative and its
/// negative only exist for "tú" (1) and "ustedes" (4).
fn finite(inf: &str, tense: Tense, p: usize, negative: bool) -> String {
    let plain = inf.replace('í', "i");
    let irr = irregular(inf);
    let pick = |forms: Option<Six>| forms.map(|f| f[p].to_string());
    match tense {
        Tense::Present => pick(irr.present).unwrap_or_else(|| regular(&plain, tense, p)),
        Tense::Preterite => pick(irr.preterite).unwrap_or_else(|| regular(&plain, tense, p)),
        Tense::Imperfect => pick(irr.imperfect).unwrap_or_else(|| regular(&plain, tense, p)),
        Tense::Future => irr
            .future_stem
            .map(|s| format!("{s}{}", FUTURE[p]))
            .unwrap_or_else(|| regular(&plain, tense, p)),
        Tense::Conditional => irr
            .future_stem
            .map(|s| format!("{s}{}", CONDITIONAL[p]))
            .unwrap_or_else(|| regular(&plain, tense, p)),
        Tense::Progressive => format!("{} {}", ESTAR[p], gerund(&plain, &irr)),
        Tense::PastProgressive => format!("{} {}", ESTABA[p], gerund(&plain, &irr)),
        Tense::Perfect => format!("{} {}", HABER[p], participle(&plain, &irr)),
        Tense::Subjunctive => subjunctive(&plain, &irr, p),
        // "tú": ¡duerme! / ¡no duermas!; "ustedes": ¡duerman! / ¡no duerman!
        Tense::Imperative if p == 1 && !negative => irr
            .imperative
            .map(str::to_string)
            // The "tú" imperative is the él/ella present: duerme, come.
            .unwrap_or_else(|| {
                irr.present
                    .map(|f| f[2].to_string())
                    .unwrap_or_else(|| regular(&plain, Tense::Present, 2))
            }),
        Tense::Imperative => subjunctive(&plain, &irr, if p == 1 { 1 } else { 4 }),
        // Resolved to imperative or subjunctive in `render`.
        Tense::Optative => subjunctive(&plain, &irr, p),
    }
}

/// A meaning split into its parts: "reírse" → reflexive, "hacerlo" →
/// object pronoun "lo", "beber agua" → rest "agua".
struct Meaning {
    inf: String,
    reflexive: bool,
    object: Option<&'static str>,
    rest: Vec<String>,
}

fn is_infinitive(w: &str) -> bool {
    w.ends_with("ar") || w.ends_with("er") || w.ends_with("ir") || w.ends_with("ír")
}

fn split(meaning: &str) -> Option<Meaning> {
    let mut words = meaning.split_whitespace();
    let first = words.next()?;
    let rest = words.map(str::to_string).collect();
    let mut inf = first;
    let mut object = None;
    for clitic in ["los", "las", "lo", "la"] {
        if let Some(base) = inf.strip_suffix(clitic).filter(|b| is_infinitive(b)) {
            inf = base;
            object = Some(clitic);
            break;
        }
    }
    let reflexive = match inf.strip_suffix("se") {
        Some(base) if base.ends_with('r') => {
            inf = base;
            true
        }
        _ => false,
    };
    if !is_infinitive(inf) {
        return None;
    }
    Some(Meaning {
        inf: inf.to_string(),
        reflexive,
        object,
        rest,
    })
}

/// Puts a written accent on the stressed syllable before adding an
/// enclitic: mira + lo → míralo, hagan + lo → háganlo, haz + lo → hazlo.
fn with_enclitic(form: &str, clitic: &str) -> String {
    let chars: Vec<char> = form.chars().collect();
    if chars.iter().any(|c| "áéíóú".contains(*c)) {
        return format!("{form}{clitic}");
    }
    let strong = |c: char| "aeo".contains(c);
    let vowel = |c: char| "aeiouü".contains(c);
    // Syllable nuclei: runs of vowels, split between two strong vowels (le-e).
    let mut nuclei: Vec<Vec<usize>> = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if !vowel(c) {
            continue;
        }
        let joins = i > 0
            && vowel(chars[i - 1])
            && !(strong(chars[i - 1]) && strong(c))
            && !nuclei.is_empty();
        if joins {
            nuclei.last_mut().unwrap().push(i);
        } else {
            nuclei.push(vec![i]);
        }
    }
    if nuclei.len() < 2 {
        return format!("{form}{clitic}");
    }
    let nucleus = &nuclei[nuclei.len() - 2];
    let at = nucleus
        .iter()
        .rev()
        .find(|&&i| strong(chars[i]))
        .or(nucleus.last())
        .copied()
        .unwrap();
    let accented = match chars[at] {
        'a' => 'á',
        'e' => 'é',
        'i' => 'í',
        'o' => 'ó',
        'u' => 'ú',
        c => c,
    };
    let mut out: String = chars
        .iter()
        .enumerate()
        .map(|(i, &c)| if i == at { accented } else { c })
        .collect();
    out.push_str(clitic);
    out
}

/// Says a meaning ("dormir", "reírse", "hacerlo", "beber agua") in Spanish
/// for one person (0..6, in the order of [`PRONOUNS`]). Returns `None` when
/// the meaning doesn't start with an infinitive, or the form doesn't exist
/// (an imperative for "yo").
pub fn render(meaning: &str, es: &Es, person: usize, negative: bool) -> Option<String> {
    let mut spec = es.spec();
    let negative = negative || spec.negative;
    let m = split(meaning)?;
    // Optative: an order for tú / ustedes, "que …" for the other persons.
    if spec.tense == Tense::Optative {
        if person == 1 || person == 4 {
            spec.tense = Tense::Imperative;
        } else {
            spec.tense = Tense::Subjunctive;
            spec.before.get_or_insert_with(|| "que".to_string());
        }
    }
    let imperative = spec.tense == Tense::Imperative;
    if imperative && person != 1 && person != 4 {
        return None;
    }
    // Pronouns that go with the verb: me/te/se… and lo/la.
    let clitics: Vec<&str> = [m.reflexive.then_some(REFLEXIVE[person]), m.object]
        .into_iter()
        .flatten()
        .collect();

    let mut parts: Vec<String> = Vec::new();
    if let Some(b) = &spec.before {
        parts.push(b.clone());
    }
    if spec.pronoun && !imperative {
        parts.push(PRONOUNS[person].into());
    }
    if let Some(a) = &spec.adverb {
        parts.push(a.clone());
    }
    if negative {
        parts.push("no".into());
    }
    match &spec.aux {
        Some(aux) => {
            // tengo que dormir, acabo de reírme, voy a hacerlo
            parts.push(finite(aux, spec.tense, person, negative));
            if let Some(l) = &spec.link {
                parts.push(l.clone());
            }
            parts.push(format!("{}{}", m.inf, clitics.concat()));
        }
        // ¡hazlo!, ¡alégrate!: affirmative orders take the pronouns after.
        None if imperative && !negative && !clitics.is_empty() => {
            parts.push(with_enclitic(
                &finite(&m.inf, spec.tense, person, false),
                &clitics.concat(),
            ));
        }
        None => {
            parts.extend(clitics.iter().map(|c| c.to_string()));
            parts.push(finite(&m.inf, spec.tense, person, negative));
        }
    }
    parts.extend(m.rest);
    let text = parts.join(" ");
    Some(if imperative {
        format!("¡{text}!")
    } else if spec.question {
        format!("¿{text}?")
    } else {
        text
    })
}

/// Shorthand for a simple tense.
pub fn conjugate(meaning: &str, tense: Tense, person: usize, negative: bool) -> Option<String> {
    render(meaning, &Es::Simple(tense), person, negative)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(m: &str, t: Tense, p: usize) -> String {
        conjugate(m, t, p, false).unwrap()
    }

    fn spec(json: &str) -> Es {
        serde_json::from_str(json).unwrap()
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
        assert_eq!(c("jugar", Tense::Present, 0), "yo juego");
        assert_eq!(c("nacer", Tense::Present, 0), "yo nazco");
        assert_eq!(c("despertar", Tense::Imperative, 1), "¡despierta!");
        assert_eq!(c("volver", Tense::Perfect, 0), "yo he vuelto");
        assert_eq!(c("sentarse", Tense::Present, 0), "yo me siento");
        assert_eq!(c("acostarse", Tense::Subjunctive, 1), "tú te acuestes");
        assert_eq!(c("traer", Tense::Progressive, 0), "yo estoy trayendo");
        assert_eq!(c("jugar", Tense::Imperative, 1), "¡juega!");
        assert_eq!(c("escribir", Tense::Perfect, 0), "yo he escrito");
    }

    #[test]
    fn reflexive_complement_and_negative() {
        assert_eq!(c("reírse", Tense::Preterite, 2), "él/ella se rio");
        assert_eq!(c("reírse", Tense::Future, 0), "yo me reiré");
        assert_eq!(c("beber agua", Tense::Present, 3), "nosotros bebemos agua");
        assert_eq!(
            conjugate("saber", Tense::Present, 0, true).unwrap(),
            "yo no sé"
        );
        assert_eq!(
            conjugate("reírse", Tense::Present, 1, true).unwrap(),
            "tú no te ríes"
        );
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
        assert_eq!(
            conjugate("ver", Tense::Perfect, 0, true).unwrap(),
            "yo no he visto"
        );
    }

    #[test]
    fn subjunctive_imperative_conditional() {
        let que = spec(r#"{"tense":"subjunctive","before":"que"}"#);
        assert_eq!(render("dormir", &que, 0, false).unwrap(), "que yo duerma");
        assert_eq!(
            render("dormir", &que, 3, false).unwrap(),
            "que nosotros durmamos"
        );
        assert_eq!(render("salir", &que, 1, false).unwrap(), "que tú salgas");
        assert_eq!(
            render("buscar", &que, 2, false).unwrap(),
            "que él/ella busque"
        );
        assert_eq!(render("llegar", &que, 0, false).unwrap(), "que yo llegue");
        assert_eq!(
            render("moler", &que, 3, false).unwrap(),
            "que nosotros molamos"
        );
        assert_eq!(render("ir", &que, 0, false).unwrap(), "que yo vaya");
        let imp = Es::Simple(Tense::Imperative);
        assert_eq!(render("dormir", &imp, 1, false).unwrap(), "¡duerme!");
        assert_eq!(render("dormir", &imp, 4, false).unwrap(), "¡duerman!");
        assert_eq!(render("dormir", &imp, 1, true).unwrap(), "¡no duermas!");
        assert_eq!(render("salir", &imp, 1, false).unwrap(), "¡sal!");
        assert_eq!(render("venir", &imp, 1, false).unwrap(), "¡ven!");
        assert_eq!(render("dormir", &imp, 0, false), None);
        assert_eq!(c("caer", Tense::Conditional, 1), "tú caerías");
    }

    #[test]
    fn periphrases() {
        let r = |json: &str, p: usize| render("dormir", &spec(json), p, false).unwrap();
        assert_eq!(
            r(r#"{"tense":"present","aux":"necesitar"}"#, 0),
            "yo necesito dormir"
        );
        assert_eq!(
            r(r#"{"tense":"present","aux":"tener","link":"que"}"#, 0),
            "yo tengo que dormir"
        );
        assert_eq!(
            r(r#"{"tense":"present","aux":"poder"}"#, 2),
            "él/ella puede dormir"
        );
        assert_eq!(
            r(r#"{"tense":"present","aux":"soler"}"#, 1),
            "tú sueles dormir"
        );
        assert_eq!(
            r(r#"{"tense":"present","aux":"querer"}"#, 0),
            "yo quiero dormir"
        );
        assert_eq!(
            r(r#"{"tense":"present","aux":"acabar","link":"de"}"#, 0),
            "yo acabo de dormir"
        );
        assert_eq!(
            r(r#"{"tense":"preterite","aux":"empezar","link":"a"}"#, 0),
            "yo empecé a dormir"
        );
        assert_eq!(
            r(r#"{"tense":"preterite","aux":"comenzar","link":"a"}"#, 0),
            "yo comencé a dormir"
        );
        assert_eq!(
            r(r#"{"tense":"preterite","aux":"terminar","link":"de"}"#, 2),
            "él/ella terminó de dormir"
        );
        assert_eq!(
            r(r#"{"tense":"preterite","adverb":"casi"}"#, 0),
            "yo casi dormí"
        );
        assert_eq!(
            r(r#"{"tense":"preterite","adverb":"ya"}"#, 0),
            "yo ya dormí"
        );
        assert_eq!(
            r(r#"{"tense":"present","before":"cuando"}"#, 0),
            "cuando yo duermo"
        );
        assert_eq!(
            r(r#"{"tense":"present","aux":"ir","link":"a"}"#, 0),
            "yo voy a dormir"
        );
        assert_eq!(
            r(
                r#"{"tense":"present","aux":"ir","link":"a","pronoun":false}"#,
                3
            ),
            "vamos a dormir"
        );
        assert_eq!(
            r(r#"{"tense":"imperative","aux":"venir","link":"a"}"#, 1),
            "¡ven a dormir!"
        );
        assert_eq!(
            r(r#"{"tense":"imperative","aux":"venir","link":"a"}"#, 4),
            "¡vengan a dormir!"
        );
        assert_eq!(
            r(
                r#"{"tense":"conditional","aux":"poder","before":"cuidado:"}"#,
                1
            ),
            "cuidado: tú podrías dormir"
        );
        assert_eq!(
            render(
                "reírse",
                &spec(r#"{"tense":"present","aux":"acabar","link":"de"}"#),
                0,
                false
            )
            .unwrap(),
            "yo acabo de reírme"
        );
    }

    #[test]
    fn object_pronouns_and_optative() {
        assert_eq!(c("hacerlo", Tense::Present, 0), "yo lo hago");
        assert_eq!(c("verlo", Tense::Perfect, 1), "tú lo has visto");
        assert_eq!(
            c("decirlo", Tense::Progressive, 2),
            "él/ella lo está diciendo"
        );
        let r = |m: &str, json: &str, p: usize, neg: bool| render(m, &spec(json), p, neg).unwrap();
        assert_eq!(
            r(
                "hacerlo",
                r#"{"tense":"present","aux":"ir","link":"a"}"#,
                0,
                false
            ),
            "yo voy a hacerlo"
        );
        let opt = r#""optative""#;
        assert_eq!(r("hacerlo", opt, 1, false), "¡hazlo!");
        assert_eq!(r("hacerlo", opt, 4, false), "¡háganlo!");
        assert_eq!(r("hacerlo", opt, 0, false), "que yo lo haga");
        assert_eq!(r("hacerlo", opt, 1, true), "¡no lo hagas!");
        assert_eq!(r("verlo", opt, 1, false), "¡velo!");
        assert_eq!(r("leerlo", opt, 1, false), "¡léelo!");
        assert_eq!(r("escucharlo", opt, 1, false), "¡escúchalo!");
        assert_eq!(r("alegrarse", opt, 1, false), "¡alégrate!");
        assert_eq!(r("alegrarse", opt, 4, false), "¡alégrense!");
        assert_eq!(r("dormir", opt, 2, false), "que él/ella duerma");
        assert_eq!(r("decirlo", opt, 1, false), "¡dilo!");
        assert_eq!(r("darlo", opt, 0, false), "que yo lo dé");
        assert_eq!(
            r(
                "conocerlo",
                r#"{"tense":"optative","negative":true}"#,
                0,
                false
            ),
            "que yo no lo conozca"
        );
        assert_eq!(c("leer", Tense::Preterite, 2), "él/ella leyó");
    }

    #[test]
    fn not_a_verb() {
        assert_eq!(conjugate("agua", Tense::Present, 0, false), None);
    }
}
