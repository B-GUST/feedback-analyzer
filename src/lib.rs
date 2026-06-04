use pyo3::prelude::*;

pub mod crypto;
pub mod data;
pub mod nlp;

use crypto::{hash_data, sign_data, verify_signature, generate_keypair};
use crypto::merkle::{compute_merkle_root, generate_merkle_proof, verify_merkle_proof, build_merkle_tree};
use nlp::{detect_language, extract_keywords};
use data::{calculate_sentiment_score, calculate_quality_score};

/// Feedback Analyzer Rust Core
#[pymodule]
fn rust_analyzer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Crypto functions
    m.add_function(wrap_pyfunction!(hash_data, m)?)?;
    m.add_function(wrap_pyfunction!(sign_data, m)?)?;
    m.add_function(wrap_pyfunction!(verify_signature, m)?)?;
    m.add_function(wrap_pyfunction!(generate_keypair, m)?)?;
    
    // Merkle tree functions
    m.add_function(wrap_pyfunction!(compute_merkle_root, m)?)?;
    m.add_function(wrap_pyfunction!(generate_merkle_proof, m)?)?;
    m.add_function(wrap_pyfunction!(verify_merkle_proof, m)?)?;
    m.add_function(wrap_pyfunction!(build_merkle_tree, m)?)?;
    
    // NLP functions
    m.add_function(wrap_pyfunction!(detect_language, m)?)?;
    m.add_function(wrap_pyfunction!(extract_keywords, m)?)?;
    
    // Data/Metrics functions
    m.add_function(wrap_pyfunction!(calculate_sentiment_score, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_quality_score, m)?)?;
    
    Ok(())
}
