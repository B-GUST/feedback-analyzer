use pyo3::prelude::*;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// Common stopwords for multiple languages
const STOPWORDS_EN: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "is", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could",
    "should", "may", "might", "shall", "can", "this", "that", "these",
    "those", "i", "you", "he", "she", "it", "we", "they", "me", "him",
    "her", "us", "them", "my", "your", "his", "its", "our", "their",
    "what", "which", "who", "whom", "where", "when", "why", "how",
    "not", "no", "nor", "so", "too", "very", "just", "about", "above",
    "after", "again", "all", "also", "any", "as", "back", "because",
    "before", "between", "both", "come", "day", "even", "find", "first",
    "get", "give", "go", "here", "if", "into", "know", "last", "let",
    "like", "long", "look", "make", "many", "more", "most", "much",
    "must", "new", "now", "old", "only", "other", "out", "over",
    "own", "part", "put", "right", "same", "see", "some", "still",
    "such", "take", "there", "through", "under", "up", "us", "use",
    "want", "way", "well", "work", "year", "than", "then",
];

const STOPWORDS_ES: &[&str] = &[
    "el", "la", "los", "las", "un", "una", "unos", "unas", "y", "o",
    "pero", "en", "de", "del", "al", "con", "por", "para", "sin",
    "sobre", "entre", "desde", "hasta", "es", "son", "está", "están",
    "fue", "eran", "ser", "estar", "haber", "tener", "hacer", "poder",
    "querer", "saber", "decir", "ir", "venir", "dar", "ver", "poner",
    "salir", "seguir", "encontrar", "llamar", "creer", "llevar", "dejar",
    "quedar", "hablar", "esperar", "trabajar", "preguntar", "intentar",
    "mantener", "comenzar", "existir", "entrar", "volver", "tomar",
    "conocer", "vivir", "sentir", "producir", "ocurrir", "correr",
    "yo", "tú", "él", "ella", "nosotros", "ellos", "ellas", "me",
    "te", "se", "nos", "mi", "tu", "su", "nuestro", "este", "esta",
    "estos", "estas", "ese", "esa", "esos", "esas", "aquel", "aquella",
    "qué", "cuál", "quién", "dónde", "cuándo", "cómo", "cuánto",
    "no", "sí", "también", "aquí", "ahí", "allí", "entonces", "así",
    "muy", "mucho", "poco", "nada", "todo", "cada", "otro", "mismo",
];

/// Extract keywords from text using TF-IDF-like scoring
/// Returns top N keywords sorted by relevance
#[pyfunction]
pub fn extract_keywords(text: &str, top_n: Option<usize>) -> Vec<String> {
    let n = top_n.unwrap_or(5);
    
    // Tokenize and normalize
    let words: Vec<String> = text.unicode_words()
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() > 2)
        .collect();
    
    if words.is_empty() {
        return Vec::new();
    }
    
    // Determine language for stopwords
    let stopwords = if text.contains("ñ") || text.contains("¿") || text.contains("¡") {
        STOPWORDS_ES
    } else {
        STOPWORDS_EN
    };
    
    // Count word frequencies
    let mut freq: HashMap<String, usize> = HashMap::new();
    for word in &words {
        if !stopwords.contains(&word.as_str()) {
            *freq.entry(word.clone()).or_insert(0) += 1;
        }
    }
    
    // Sort by frequency and return top N
    let mut keywords: Vec<(String, usize)> = freq.into_iter().collect();
    keywords.sort_by(|a, b| b.1.cmp(&a.1));
    
    keywords.into_iter()
        .take(n)
        .map(|(word, _)| word)
        .collect()
}

/// Extract keywords with scores
#[pyfunction]
pub fn extract_keywords_with_scores(text: &str, top_n: Option<usize>) -> Vec<(String, f64)> {
    let n = top_n.unwrap_or(5);
    
    let words: Vec<String> = text.unicode_words()
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() > 2)
        .collect();
    
    if words.is_empty() {
        return Vec::new();
    }
    
    let stopwords = if text.contains("ñ") || text.contains("¿") || text.contains("¡") {
        STOPWORDS_ES
    } else {
        STOPWORDS_EN
    };
    
    let total_words = words.len() as f64;
    let mut freq: HashMap<String, usize> = HashMap::new();
    for word in &words {
        if !stopwords.contains(&word.as_str()) {
            *freq.entry(word.clone()).or_insert(0) += 1;
        }
    }
    
    let mut keywords: Vec<(String, f64)> = freq.into_iter()
        .map(|(word, count)| (word, count as f64 / total_words))
        .collect();
    
    keywords.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    keywords.into_iter()
        .take(n)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords_english() {
        let text = "The product quality is excellent and the service is great";
        let keywords = extract_keywords(text, Some(3));
        assert!(keywords.len() <= 3);
        assert!(keywords.contains(&"product".to_string()) || keywords.contains(&"quality".to_string()));
    }

    #[test]
    fn test_extract_keywords_spanish() {
        let text = "La calidad del producto es excelente y el servicio es genial";
        let keywords = extract_keywords(text, Some(3));
        assert!(keywords.len() <= 3);
    }

    #[test]
    fn test_extract_empty_text() {
        let keywords = extract_keywords("", Some(5));
        assert!(keywords.is_empty());
    }

    #[test]
    fn test_extract_keywords_with_scores() {
        let text = "great product great service great quality";
        let keywords = extract_keywords_with_scores(text, Some(3));
        assert!(!keywords.is_empty());
        assert!(keywords[0].1 > 0.0);
    }
}
