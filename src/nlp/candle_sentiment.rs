use candle_core::{Device, Tensor, DType, Result as CandleResult};
use candle_nn::VarBuilder;
use pyo3::prelude::*;
use std::path::PathBuf;

/// Sentiment analyzer using candle with trained weights
pub struct CandleSentimentAnalyzer {
    embeddings: Tensor,
    fc1_weight: Tensor,
    fc1_bias: Tensor,
    fc2_weight: Tensor,
    fc2_bias: Tensor,
    classifier_weight: Tensor,
    classifier_bias: Tensor,
    vocab: std::collections::HashMap<String, u32>,
    device: Device,
}

impl CandleSentimentAnalyzer {
    /// Load trained model from safetensors file
    pub fn from_file(model_path: &str, vocab_path: &str, device: Device) -> CandleResult<Self> {
        // Load weights from safetensors
        let weights = unsafe {
            VarBuilder::from_mmaped_safetensors(
                &[PathBuf::from(model_path)],
                DType::F32,
                &device,
            )?
        };
        
        // Load model weights
        let embeddings = weights.get((30522, 128), "embeddings.weight")?;
        let fc1_weight = weights.get((128, 128), "fc1.weight")?;
        let fc1_bias = weights.get(128, "fc1.bias")?;
        let fc2_weight = weights.get((128, 128), "fc2.weight")?;
        let fc2_bias = weights.get(128, "fc2.bias")?;
        let classifier_weight = weights.get((2, 128), "classifier.weight")?;
        let classifier_bias = weights.get(2, "classifier.bias")?;
        
        // Load vocabulary
        let vocab_content = std::fs::read_to_string(vocab_path)
            .map_err(|e| candle_core::Error::Msg(format!("Failed to read vocab: {}", e)))?;
        
        let mut vocab = std::collections::HashMap::new();
        for (idx, line) in vocab_content.lines().enumerate() {
            vocab.insert(line.trim().to_string(), idx as u32);
        }
        
        Ok(Self {
            embeddings,
            fc1_weight,
            fc1_bias,
            fc2_weight,
            fc2_bias,
            classifier_weight,
            classifier_bias,
            vocab,
            device,
        })
    }
    
    /// Forward pass through the model
    pub fn forward(&self, input_ids: &Tensor, attention_mask: &Tensor) -> CandleResult<Tensor> {
        let batch_size = input_ids.dim(0)?;
        let seq_len = input_ids.dim(1)?;
        let hidden_size = self.embeddings.dim(1)?;
        
        // Embedding lookup
        let flat_ids = input_ids.contiguous()?.reshape(batch_size * seq_len)?.to_dtype(DType::I64)?;
        let flat_embs = self.embeddings.index_select(&flat_ids, 0)?;
        let embeddings = flat_embs.reshape((batch_size, seq_len, hidden_size))?;
        
        // Attention mask pooling
        let mask = attention_mask.contiguous()?.unsqueeze(2)?.to_dtype(DType::F32)?;
        let masked = embeddings.broadcast_mul(&mask)?;
        let sum_mask = mask.sum(1)?.clamp(1e-8, f32::MAX)?;
        let pooled = masked.sum(1)?.broadcast_div(&sum_mask)?;
        
        // Forward through layers
        let h = pooled.matmul(&self.fc1_weight.t()?)?.broadcast_add(&self.fc1_bias)?;
        let h = h.gelu()?;
        let h = h.matmul(&self.fc2_weight.t()?)?.broadcast_add(&self.fc2_bias)?;
        let h = h.gelu()?;
        
        // Classification
        let logits = h.matmul(&self.classifier_weight.t()?)?.broadcast_add(&self.classifier_bias)?;
        
        candle_nn::ops::softmax(&logits, 1)
    }
    
    /// Tokenize text using vocabulary
    fn tokenize(&self, text: &str, max_len: usize) -> (Vec<u32>, Vec<u32>) {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        
        let mut ids = vec![*self.vocab.get("[CLS]").unwrap_or(&101)];
        
        for word in words {
            if ids.len() >= max_len - 1 {
                break;
            }
            if let Some(&id) = self.vocab.get(word) {
                ids.push(id);
            } else {
                // Subword tokenization
                let chars: Vec<char> = word.chars().collect();
                let mut start = 0;
                while start < chars.len() {
                    let mut found = false;
                    for end in (start + 1..=chars.len()).rev() {
                        let substr: String = chars[start..end].iter().collect();
                        let token = if start == 0 { substr.clone() } else { format!("##{}", substr) };
                        if let Some(&id) = self.vocab.get(&token) {
                            ids.push(id);
                            start = end;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        ids.push(*self.vocab.get("[UNK]").unwrap_or(&100));
                        break;
                    }
                }
            }
        }
        
        ids.push(*self.vocab.get("[SEP]").unwrap_or(&102));
        
        let orig_len = ids.len();
        ids.resize(max_len, 0);
        let mut mask = vec![1u32; orig_len];
        mask.resize(max_len, 0);
        
        (ids, mask)
    }
    
    /// Predict sentiment from text
    pub fn predict(&self, text: &str, max_len: usize) -> CandleResult<(f64, String)> {
        let (ids, mask) = self.tokenize(text, max_len);
        
        let ids_i64: Vec<i64> = ids.iter().map(|&x| x as i64).collect();
        let mask_i64: Vec<i64> = mask.iter().map(|&x| x as i64).collect();
        
        let token_ids = Tensor::new(ids_i64.as_slice(), &self.device)?.reshape((1, max_len))?;
        let attention_mask = Tensor::new(mask_i64.as_slice(), &self.device)?.reshape((1, max_len))?;
        
        let probs = self.forward(&token_ids, &attention_mask)?;
        let probs_vec = probs.to_vec2::<f32>()?;
        
        let negative_prob = probs_vec[0][0] as f64;
        let positive_prob = probs_vec[0][1] as f64;
        
        let (score, label) = if positive_prob > negative_prob {
            (positive_prob, "positive".to_string())
        } else {
            (-negative_prob, "negative".to_string())
        };
        
        Ok((score, label))
    }
}

/// Python wrapper for candle sentiment
#[pyclass]
pub struct CandleSentiment {
    analyzer: Option<CandleSentimentAnalyzer>,
    model_type: String,
}

#[pymethods]
impl CandleSentiment {
    #[new]
    fn new() -> PyResult<Self> {
        // Try to load trained model
        let model_path = "models/sentiment/trained/model.safetensors";
        let vocab_path = "models/sentiment/vocab.txt";
        
        if std::path::Path::new(model_path).exists() {
            match CandleSentimentAnalyzer::from_file(model_path, vocab_path, Device::Cpu) {
                Ok(analyzer) => {
                    println!("Loaded trained sentiment model from {}", model_path);
                    return Ok(Self { 
                        analyzer: Some(analyzer), 
                        model_type: "trained".to_string() 
                    });
                }
                Err(e) => {
                    println!("Failed to load trained model: {}. Using fallback.", e);
                }
            }
        }
        
        Ok(Self { analyzer: None, model_type: "fallback".to_string() })
    }
    
    /// Create with specific model path
    #[staticmethod]
    fn from_model(model_path: &str, vocab_path: &str) -> PyResult<Self> {
        match CandleSentimentAnalyzer::from_file(model_path, vocab_path, Device::Cpu) {
            Ok(analyzer) => Ok(Self { 
                analyzer: Some(analyzer), 
                model_type: "custom".to_string() 
            }),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        }
    }
    
    /// Analyze sentiment using candle
    fn analyze(&self, text: &str) -> PyResult<(f64, String)> {
        if let Some(ref analyzer) = self.analyzer {
            analyzer.predict(text, 64)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        } else {
            // Fallback to simple feature-based analysis
            let features = text_to_features(text);
            Ok(fallback_predict(&features))
        }
    }
    
    /// Get model info
    fn model_info(&self) -> PyResult<String> {
        Ok(format!("Candle Sentiment Model (type: {})", self.model_type))
    }
    
    /// Check if trained model is loaded
    fn is_trained(&self) -> bool {
        self.analyzer.is_some()
    }
}

/// Simple feature extraction from text (fallback)
fn text_to_features(text: &str) -> Vec<f32> {
    let mut features = vec![0.0f32; 128];
    let text_lower = text.to_lowercase();
    
    for word in text_lower.split_whitespace() {
        let hash = word.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let idx = (hash % 128) as usize;
        features[idx] += 1.0;
    }
    
    let norm: f32 = features.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for f in &mut features {
            *f /= norm;
        }
    }
    
    features
}

/// Fallback prediction using simple features
fn fallback_predict(features: &[f32]) -> (f64, String) {
    // Simple weighted sum for demo
    let positive_words = ["excellent", "great", "good", "amazing", "love", "best"];
    let negative_words = ["bad", "terrible", "horrible", "worst", "hate", "poor"];
    
    let score = features.iter().enumerate().map(|(i, &v)| {
        if i % 2 == 0 { v } else { -v }
    }).sum::<f32>();
    
    let prob = (score.tanh() + 1.0) / 2.0;
    let label = if prob > 0.5 { "positive" } else { "negative" };
    
    (prob as f64, label.to_string())
}
