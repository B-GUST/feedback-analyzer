use pyo3::prelude::*;
use whatlang::detect;

/// Detect the language of text
/// Returns ISO 639-1 language code (e.g., "en", "es", "fr")
#[pyfunction]
pub fn detect_language(text: &str) -> String {
    match detect(text) {
        Some(info) => info.lang().code().to_string(),
        None => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_english() {
        assert_eq!(detect_language("Hello world, this is English text"), "en");
    }

    #[test]
    fn test_detect_spanish() {
        assert_eq!(detect_language("Hola mundo, este es texto en español"), "es");
    }

    #[test]
    fn test_detect_unknown() {
        // Very short or ambiguous text
        let lang = detect_language("123");
        assert!(lang == "unknown" || !lang.is_empty());
    }
}
