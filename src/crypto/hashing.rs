use pyo3::prelude::*;
use sha2::{Sha256, Digest};

/// Hash data using SHA-256
#[pyfunction]
pub fn hash_data(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Hash multiple chunks and return the combined hash
#[pyfunction]
pub fn hash_chunks(chunks: Vec<String>) -> String {
    let mut hasher = Sha256::new();
    for chunk in chunks {
        hasher.update(chunk.as_bytes());
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_data() {
        let hash = hash_data("hello world");
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_hash_chunks() {
        let chunks = vec!["hello".to_string(), " world".to_string()];
        let hash = hash_chunks(chunks);
        assert_eq!(hash, hash_data("hello world"));
    }
}
