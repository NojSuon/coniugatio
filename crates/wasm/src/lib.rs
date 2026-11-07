use coniugatio_core::Engine;
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
