use coniugatio_core::{conjugate::Options, Engine};
use wasm_bindgen::prelude::*;

fn run(conjugation_json: &str, verb: &str, opts: &Options) -> Result<String, String> {
    let conjugation: Engine = serde_json::from_str(conjugation_json).map_err(|e| {
        if opts.english() {
            format!("Invalid language data: {e}")
        } else {
            format!("Datos de la lengua no válidos: {e}")
        }
    })?;
    let result = conjugation.conjugate_with(verb, opts).map_err(|e| e.message(opts))?;
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

/// Conjugates `verb` with a language's `conjugation` block (as JSON).
/// `lang` is the page's language ("es" or "en"): it picks the language of
/// the equivalents and of the error messages. Returns the result as JSON,
/// or a readable error message.
#[wasm_bindgen]
pub fn conjugate(conjugation_json: &str, verb: &str, lang: &str) -> Result<String, String> {
    let opts = Options { lang: Some(lang.to_string()), ..Default::default() };
    run(conjugation_json, verb, &opts)
}

/// Like `conjugate`, for a verb typed by the user with its meaning (`es`,
/// in the page's language, may be empty) and type (`kind`, empty to
/// detect it).
#[wasm_bindgen]
pub fn conjugate_with(conjugation_json: &str, verb: &str, es: &str, kind: &str, lang: &str) -> Result<String, String> {
    let opts = Options { es: Some(es.to_string()), kind: Some(kind.to_string()), lang: Some(lang.to_string()) };
    run(conjugation_json, verb, &opts)
}
