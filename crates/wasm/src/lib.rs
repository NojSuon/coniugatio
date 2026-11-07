use coniugatio_core::{conjugate::Options, Engine};
use wasm_bindgen::prelude::*;

/// Conjugates `verb` with a language's `conjugation` block (as JSON).
/// Returns the result as JSON, or a readable error message.
#[wasm_bindgen]
pub fn conjugate(conjugation_json: &str, verb: &str) -> Result<String, String> {
    let conjugation: Engine =
        serde_json::from_str(conjugation_json).map_err(|e| format!("Datos de la lengua no válidos: {e}"))?;
    let result = conjugation.conjugate(verb).map_err(|e| e.to_string())?;
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

/// Like `conjugate`, for a verb typed by the user with its Spanish meaning
/// (`es`, may be empty) and type (`kind`, empty to detect it).
#[wasm_bindgen]
pub fn conjugate_with(conjugation_json: &str, verb: &str, es: &str, kind: &str) -> Result<String, String> {
    let conjugation: Engine =
        serde_json::from_str(conjugation_json).map_err(|e| format!("Datos de la lengua no válidos: {e}"))?;
    let opts = Options { es: Some(es.to_string()), kind: Some(kind.to_string()) };
    let result = conjugation.conjugate_with(verb, &opts).map_err(|e| e.to_string())?;
    serde_json::to_string(&result).map_err(|e| e.to_string())
}
