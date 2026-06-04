use pyo3::prelude::*;

pub mod crypto;
pub mod data;
pub mod nlp;

use crypto::{hash_data, sign_data, verify_signature, generate_keypair};
use nlp::{detect_language, extract_keywords};

/// Feedback Analyzer Rust Core
#[pymodule]
fn rust_analyzer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Crypto functions
    m.add_function(wrap_pyfunction!(hash_data, m)?)?;
    m.add_function(wrap_pyfunction!(sign_data, m)?)?;
    m.add_function(wrap_pyfunction!(verify_signature, m)?)?;
    m.add_function(wrap_pyfunction!(generate_keypair, m)?)?;
    
    // NLP functions
    m.add_function(wrap_pyfunction!(detect_language, m)?)?;
    m.add_function(wrap_pyfunction!(extract_keywords, m)?)?;
    
    Ok(())
}
