use candle_core::{Device, Tensor, Result as CandleResult};
use pyo3::prelude::*;

/// Simple sentiment analyzer using candle
pub struct CandleSentimentAnalyzer {
    weights: Tensor,
    bias: Tensor,
    device: Device,
}

impl CandleSentimentAnalyzer {
    pub fn create(device: Device) -> CandleResult<Self> {
        let weights = Tensor::randn(0f32, 0.02f32, (128, 2), &device)?;
        let bias = Tensor::zeros(2, candle_core::DType::F32, &device)?;
        Ok(Self { weights, bias, device })
    }
    
    pub fn predict(&self, input: &Tensor) -> CandleResult<Tensor> {
        let hidden = input.matmul(&self.weights)?;
        let output = hidden.broadcast_add(&self.bias)?;
        candle_nn::ops::softmax(&output, 1)
    }
}

/// Python wrapper for candle sentiment
#[pyclass]
pub struct CandleSentiment {
    analyzer: CandleSentimentAnalyzer,
}

#[pymethods]
impl CandleSentiment {
    #[new]
    fn new() -> PyResult<Self> {
        let device = Device::Cpu;
        let analyzer = CandleSentimentAnalyzer::create(device)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { analyzer })
    }
    
    /// Analyze sentiment using candle
    fn analyze(&self, text: &str) -> PyResult<(f64, String)> {
        let features = text_to_features(text);
        
        let input = Tensor::new(&features[..], &self.analyzer.device)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
            .unsqueeze(0)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let probs = self.analyzer.predict(&input)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let probs_vec = probs.to_vec2::<f32>()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let negative_prob = probs_vec[0][0] as f64;
        let positive_prob = probs_vec[0][1] as f64;
        
        let (score, label) = if positive_prob > negative_prob {
            (positive_prob, "positive".to_string())
        } else {
            (-negative_prob, "negative".to_string())
        };
        
        Ok((score, label))
    }
    
    /// Get model info
    fn model_info(&self) -> PyResult<String> {
        Ok(format!(
            "Candle Sentiment Model ({} x {} weights)",
            self.analyzer.weights.dim(0).unwrap_or(0),
            self.analyzer.weights.dim(1).unwrap_or(0)
        ))
    }
}

/// Simple feature extraction from text
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
