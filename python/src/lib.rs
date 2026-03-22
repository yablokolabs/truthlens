#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;
use pyo3::types::PyModule;

/// Convert a serde_json error to a PyErr.
fn serde_err(e: serde_json::Error) -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
}

/// Parse a JSON string into a Python object via `json.loads`.
fn json_to_py(py: Python<'_>, json_str: &str) -> PyResult<PyObject> {
    let json_mod = PyModule::import_bound(py, "json")?;
    let result = json_mod.call_method1("loads", (json_str,))?;
    Ok(result.into())
}

/// Analyze text for hallucination risk.
///
/// Returns a dict with trust score, risk level, per-claim breakdown,
/// trajectory analysis, and claim count.
#[pyfunction]
fn analyze(py: Python<'_>, text: &str) -> PyResult<PyObject> {
    let report = truthlens::analyze(text);
    let json_str = serde_json::to_string(&report).map_err(serde_err)?;
    json_to_py(py, &json_str)
}

/// Analyze text with entity verification against Wikidata.
///
/// Like `analyze()`, but also cross-references named entities against
/// Wikidata SPARQL. Requires network access.
#[pyfunction]
fn analyze_with_verification(py: Python<'_>, text: &str) -> PyResult<PyObject> {
    let report = truthlens::analyze_with_verification(text);
    let json_str = serde_json::to_string(&report).map_err(serde_err)?;
    json_to_py(py, &json_str)
}

/// Check consistency across multiple AI responses.
///
/// Pass a list of response strings. Returns a dict with consistency score,
/// contradictions, consistent claims, and unique claims.
#[pyfunction]
fn check_consistency(py: Python<'_>, responses: Vec<String>) -> PyResult<PyObject> {
    let refs: Vec<&str> = responses.iter().map(|s| s.as_str()).collect();
    let report = truthlens::check_consistency(&refs);
    let json_str = serde_json::to_string(&report).map_err(serde_err)?;
    json_to_py(py, &json_str)
}

/// Extract atomic claims from text.
///
/// Returns a list of dicts, each with `text`, `sentence_idx`,
/// `is_verifiable`, `specificity`, and `is_hedged`.
#[pyfunction]
fn extract_claims(py: Python<'_>, text: &str) -> PyResult<PyObject> {
    let claims = truthlens::extract_claims(text);
    let json_str = serde_json::to_string(&claims).map_err(serde_err)?;
    json_to_py(py, &json_str)
}

/// Extract named entities (people, places, years) from a claim.
///
/// Returns a list of strings.
#[pyfunction]
fn extract_entities(text: &str) -> Vec<String> {
    truthlens::entity::extract_entities_from_claim(text)
}

/// TruthLens Python bindings.
#[pymodule]
fn _truthlens(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(analyze, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_with_verification, m)?)?;
    m.add_function(wrap_pyfunction!(check_consistency, m)?)?;
    m.add_function(wrap_pyfunction!(extract_claims, m)?)?;
    m.add_function(wrap_pyfunction!(extract_entities, m)?)?;
    Ok(())
}
