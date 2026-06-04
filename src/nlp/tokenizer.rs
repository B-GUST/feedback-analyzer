use std::collections::HashMap;

/// Simple BERT tokenizer that works with vocab.txt
pub struct SimpleTokenizer {
    vocab: HashMap<String, u32>,
    ids_to_tokens: HashMap<u32, String>,
    max_length: usize,
}

impl SimpleTokenizer {
    /// Create tokenizer from vocab.txt file
    pub fn from_vocab_file(vocab_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(vocab_path)?;
        let mut vocab = HashMap::new();
        let mut ids_to_tokens = HashMap::new();
        
        for (idx, line) in content.lines().enumerate() {
            let token = line.trim().to_string();
            vocab.insert(token.clone(), idx as u32);
            ids_to_tokens.insert(idx as u32, token);
        }
        
        Ok(Self {
            vocab,
            ids_to_tokens,
            max_length: 128,
        })
    }
    
    /// Tokenize text into BERT input IDs
    /// Returns (token_ids, attention_mask, token_type_ids)
    pub fn encode(&self, text: &str) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
        let tokens = self.tokenize(text);
        let mut token_ids: Vec<u32> = Vec::new();
        let mut attention_mask: Vec<u32> = Vec::new();
        let mut token_type_ids: Vec<u32> = Vec::new();
        
        // Add [CLS] token
        token_ids.push(self.vocab_id("[CLS]").unwrap_or(101));
        attention_mask.push(1);
        token_type_ids.push(0);
        
        // Add tokens
        for token in &tokens {
            if token_ids.len() >= self.max_length - 1 {
                break;
            }
            if let Some(&id) = self.vocab.get(token.as_str()) {
                token_ids.push(id);
                attention_mask.push(1);
                token_type_ids.push(0);
            }
        }
        
        // Add [SEP] token
        token_ids.push(self.vocab_id("[SEP]").unwrap_or(102));
        attention_mask.push(1);
        token_type_ids.push(0);
        
        // Pad to max_length
        let pad_id = self.vocab_id("[PAD]").unwrap_or(0);
        while token_ids.len() < self.max_length {
            token_ids.push(pad_id);
            attention_mask.push(0);
            token_type_ids.push(0);
        }
        
        (token_ids, attention_mask, token_type_ids)
    }
    
    /// Simple tokenization: lowercase and split on whitespace/punctuation
    fn tokenize(&self, text: &str) -> Vec<String> {
        let text = text.to_lowercase();
        let mut tokens = Vec::new();
        let mut current = String::new();
        
        for c in text.chars() {
            if c.is_alphanumeric() || c == '\'' {
                current.push(c);
            } else {
                if !current.is_empty() {
                    // WordPiece tokenization (simplified)
                    tokens.extend(self.wordpiece(&current));
                    current.clear();
                }
            }
        }
        
        if !current.is_empty() {
            tokens.extend(self.wordpiece(&current));
        }
        
        tokens
    }
    
    /// Simplified WordPiece tokenization
    fn wordpiece(&self, word: &str) -> Vec<String> {
        if self.vocab.contains_key(word) {
            return vec![word.to_string()];
        }
        
        let mut tokens = Vec::new();
        let mut remaining = word.to_string();
        
        while !remaining.is_empty() {
            let mut found = false;
            let len = remaining.len();
            
            for i in (1..=len).rev() {
                let substr = &remaining[..i];
                let token = if tokens.is_empty() {
                    substr.to_string()
                } else {
                    format!("##{}", substr)
                };
                
                if self.vocab.contains_key(token.as_str()) {
                    tokens.push(token);
                    remaining = remaining[i..].to_string();
                    found = true;
                    break;
                }
            }
            
            if !found {
                // Unknown token
                tokens.push("[UNK]".to_string());
                remaining.clear();
            }
        }
        
        tokens
    }
    
    fn vocab_id(&self, token: &str) -> Option<u32> {
        self.vocab.get(token).copied()
    }
}
