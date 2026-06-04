use pyo3::prelude::*;

/// Simple sentiment scoring based on word lists
/// Returns score between -1.0 (negative) and 1.0 (positive)
#[pyfunction]
pub fn calculate_sentiment_score(text: &str) -> f64 {
    let positive_words: Vec<&str> = vec![
        "excellent", "great", "good", "amazing", "wonderful", "fantastic",
        "outstanding", "perfect", "love", "best", "happy", "satisfied",
        "impressive", "brilliant", "superb", "magnificent", "delightful",
        "excelente", "genial", "bueno", "increíble", "maravilloso",
        "fantástico", "perfecto", "mejor", "feliz", "satisfecho",
        "increíble", "brillante", "magnífico", "encantador",
    ];
    
    let negative_words: Vec<&str> = vec![
        "bad", "terrible", "awful", "horrible", "worst", "hate",
        "poor", "disappointing", "frustrating", "annoying", "ugly",
        "boring", "slow", "broken", "useless", "waste", "problem",
        "malo", "terrible", "horrible", "peor", "odio", "pobre",
        "decepcionante", "frustrante", "molesto", "feo", "aburrido",
        "lento", "roto", "inútil", "desperdicio", "problema",
    ];
    
    let words: Vec<String> = text.to_lowercase()
        .split_whitespace()
        .map(|w| w.to_string())
        .collect();
    
    if words.is_empty() {
        return 0.0;
    }
    
    let mut positive_count = 0;
    let mut negative_count = 0;
    
    for word in &words {
        if positive_words.iter().any(|&pw| word.contains(pw)) {
            positive_count += 1;
        }
        if negative_words.iter().any(|&nw| word.contains(nw)) {
            negative_count += 1;
        }
    }
    
    let total = (positive_count + negative_count) as f64;
    if total == 0.0 {
        return 0.0;
    }
    
    (positive_count as f64 - negative_count as f64) / total
}

/// Calculate quality score of text based on multiple factors
/// Returns score between 0.0 (low quality) and 1.0 (high quality)
#[pyfunction]
pub fn calculate_quality_score(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    
    let mut score = 0.0;
    
    // Length factor (optimal: 20-200 chars)
    let len = text.len() as f64;
    let length_score = if len < 10.0 {
        len / 10.0 * 0.2
    } else if len < 20.0 {
        0.2 + (len - 10.0) / 10.0 * 0.3
    } else if len <= 200.0 {
        0.5
    } else if len <= 500.0 {
        0.5 - (len - 200.0) / 300.0 * 0.2
    } else {
        0.3
    };
    score += length_score;
    
    // Word count factor
    let word_count = text.split_whitespace().count() as f64;
    let word_score = if word_count < 3.0 {
        word_count / 3.0 * 0.2
    } else if word_count <= 50.0 {
        0.2 + 0.3
    } else {
        0.5 - (word_count - 50.0).min(50.0) / 100.0 * 0.2
    };
    score += word_score;
    
    // Punctuation diversity (indicates thoughtful writing)
    let has_period = text.contains('.');
    let has_comma = text.contains(',');
    let has_question = text.contains('?');
    let has_exclamation = text.contains('!');
    
    let punctuation_score = match (has_period, has_comma, has_question, has_exclamation) {
        (true, true, _, _) => 0.2,
        (true, _, _, _) => 0.15,
        (_, _, true, _) => 0.1,
        (_, _, _, true) => 0.1,
        _ => 0.05,
    };
    score += punctuation_score;
    
    // Capitalization (some caps is good, ALL CAPS is bad)
    let caps_count = text.chars().filter(|c| c.is_uppercase()).count() as f64;
    let total_letters = text.chars().filter(|c| c.is_alphabetic()).count() as f64;
    
    let caps_score = if total_letters == 0.0 {
        0.0
    } else {
        let caps_ratio = caps_count / total_letters;
        if caps_ratio < 0.1 {
            0.1  // Normal writing
        } else if caps_ratio < 0.3 {
            0.05 // Some emphasis
        } else {
            0.0  // ALL CAPS - not great
        }
    };
    score += caps_score;
    
    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_sentiment() {
        let score = calculate_sentiment_score("This product is excellent and amazing");
        assert!(score > 0.5);
    }

    #[test]
    fn test_negative_sentiment() {
        let score = calculate_sentiment_score("This is terrible and horrible");
        assert!(score < -0.5);
    }

    #[test]
    fn test_neutral_sentiment() {
        let score = calculate_sentiment_score("The weather is okay today");
        assert!(score.abs() < 0.5);
    }

    #[test]
    fn test_quality_score_good() {
        let score = calculate_quality_score("The product quality is excellent. I really enjoy using it every day!");
        assert!(score > 0.6);
    }

    #[test]
    fn test_quality_score_poor() {
        let score = calculate_quality_score("bad");
        assert!(score < 0.3);
    }

    #[test]
    fn test_empty_text() {
        assert_eq!(calculate_sentiment_score(""), 0.0);
        assert_eq!(calculate_quality_score(""), 0.0);
    }
}
