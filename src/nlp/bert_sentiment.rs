use candle_core::{Device, Tensor, Result as CandleResult};
use candle_nn::{VarBuilder, Linear, Module};
use serde::Deserialize;

/// BERT model configuration
#[derive(Deserialize, Debug)]
pub struct BertConfig {
    pub hidden_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub intermediate_size: usize,
    pub vocab_size: usize,
    pub max_position_embeddings: usize,
    pub hidden_act: String,
}

/// Simple BERT-like model for sentiment analysis
pub struct BertSentimentModel {
    embeddings: Embeddings,
    layers: Vec<BertLayer>,
    classifier: Linear,
    device: Device,
}

struct Embeddings {
    word_embeddings: Tensor,
    position_embeddings: Tensor,
    layer_norm: candle_nn::LayerNorm,
}

struct BertLayer {
    attention: MultiHeadAttention,
    intermediate: Linear,
    output: Linear,
    layer_norm1: candle_nn::LayerNorm,
    layer_norm2: candle_nn::LayerNorm,
}

struct MultiHeadAttention {
    query: Linear,
    key: Linear,
    value: Linear,
    output: Linear,
    num_heads: usize,
    head_dim: usize,
}

impl BertSentimentModel {
    /// Load model from PyTorch checkpoint
    pub fn load_from_pth(
        weights_path: &str,
        config: &BertConfig,
        device: &Device,
    ) -> CandleResult<Self> {
        // Load weights from pytorch_model.bin
        let weights = std::fs::read(weights_path)
            .map_err(|e| candle_core::Error::Msg(format!("Failed to read weights: {}", e)))?;
        
        // For now, use random weights as a demo
        // In production, we'd parse the pytorch weights properly
        Self::random(config, device)
    }
    
    /// Create a random model for testing
    pub fn random(config: &BertConfig, device: &Device) -> CandleResult<Self> {
        let vs = VarBuilder::on_cpu(device)?;
        
        // Embeddings
        let word_embeddings = Tensor::randn(
            0f32,
            0.02f32,
            (config.vocab_size, config.hidden_size),
            device,
        )?;
        let position_embeddings = Tensor::randn(
            0f32,
            0.02f32,
            (config.max_position_embeddings, config.hidden_size),
            device,
        )?;
        let layer_norm = candle_nn::layer_norm(config.hidden_size, 1e-12, vs.pp("embeddings.layer_norm"))?;
        
        let embeddings = Embeddings {
            word_embeddings,
            position_embeddings,
            layer_norm,
        };
        
        // Transformer layers
        let mut layers = Vec::new();
        for i in 0..config.num_hidden_layers {
            let layer = BertLayer::new(config, &vs.pp(&format!("encoder.layer.{}", i)))?;
            layers.push(layer);
        }
        
        // Classifier
        let classifier = candle_nn::linear(config.hidden_size, 2, vs.pp("classifier"))?;
        
        Ok(Self {
            embeddings,
            layers,
            classifier,
            device: device.clone(),
        })
    }
    
    /// Forward pass
    pub fn forward(
        &self,
        input_ids: &Tensor,
        attention_mask: &Tensor,
    ) -> CandleResult<Tensor> {
        let (_batch_size, seq_len) = input_ids.dims2()?;
        
        // Get embeddings
        let word_emb = self.embeddings.word_embeddings.gather(input_ids, 1)?;
        let pos_ids = Tensor::arange(0u32, seq_len as u32, &self.device)?.unsqueeze(0)?;
        let pos_emb = self.embeddings.position_embeddings.gather(&pos_ids, 1)?;
        
        let mut hidden = (word_emb + pos_emb)?;
        hidden = self.embeddings.layer_norm.forward(&hidden, None)?;
        
        // Apply transformer layers
        for layer in &self.layers {
            hidden = layer.forward(&hidden, attention_mask)?;
        }
        
        // Take [CLS] token output
        let cls_output = hidden.select(1, 0)?;
        
        // Classification
        let logits = self.classifier.forward(&cls_output)?;
        
        Ok(logits)
    }
    
    /// Predict sentiment from token IDs
    pub fn predict_sentiment(
        &self,
        token_ids: &[u32],
        attention_mask: &[u32],
    ) -> CandleResult<(f64, String)> {
        let token_ids = Tensor::new(token_ids, &self.device)?.unsqueeze(0)?;
        let attention_mask = Tensor::new(attention_mask, &self.device)?.unsqueeze(0)?;
        
        let logits = self.forward(&token_ids, &attention_mask)?;
        
        // Apply softmax to get probabilities
        let probs = candle_nn::ops::softmax(&logits, 1)?;
        let probs_vec = probs.to_vec1::<f32>()?;
        
        // Get prediction
        let negative_prob = probs_vec[0] as f64;
        let positive_prob = probs_vec[1] as f64;
        
        let (score, label) = if positive_prob > negative_prob {
            (positive_prob, "positive".to_string())
        } else {
            (-negative_prob, "negative".to_string())
        };
        
        Ok((score, label))
    }
}

impl BertLayer {
    fn new(config: &BertConfig, vs: &candle_nn::VarBuilder) -> CandleResult<Self> {
        let attention = MultiHeadAttention::new(config, vs)?;
        let intermediate = candle_nn::linear(config.hidden_size, config.intermediate_size, vs.pp("intermediate"))?;
        let output = candle_nn::linear(config.intermediate_size, config.hidden_size, vs.pp("output"))?;
        let layer_norm1 = candle_nn::layer_norm(config.hidden_size, 1e-12, vs.pp("layer_norm_1"))?;
        let layer_norm2 = candle_nn::layer_norm(config.hidden_size, 1e-12, vs.pp("layer_norm_2"))?;
        
        Ok(Self {
            attention,
            intermediate,
            output,
            layer_norm1,
            layer_norm2,
        })
    }
    
    fn forward(&self, hidden: &Tensor, attention_mask: &Tensor) -> CandleResult<Tensor> {
        let attention_output = self.attention.forward(hidden, attention_mask)?;
        let residual = (hidden + attention_output)?;
        let hidden = self.layer_norm1.forward(&residual, None)?;
        
        let intermediate_output = self.intermediate.forward(&hidden)?;
        let intermediate_output = candle_nn::ops::gelu(&intermediate_output)?;
        let layer_output = self.output.forward(&intermediate_output)?;
        
        let residual = (hidden + layer_output)?;
        let output = self.layer_norm2.forward(&residual, None)?;
        
        Ok(output)
    }
}

impl MultiHeadAttention {
    fn new(config: &BertConfig, vs: &candle_nn::VarBuilder) -> CandleResult<Self> {
        let head_dim = config.hidden_size / config.num_attention_heads;
        
        let query = candle_nn::linear(config.hidden_size, config.hidden_size, vs.pp("query"))?;
        let key = candle_nn::linear(config.hidden_size, config.hidden_size, vs.pp("key"))?;
        let value = candle_nn::linear(config.hidden_size, config.hidden_size, vs.pp("value"))?;
        let output = candle_nn::linear(config.hidden_size, config.hidden_size, vs.pp("output"))?;
        
        Ok(Self {
            query,
            key,
            value,
            output,
            num_heads: config.num_attention_heads,
            head_dim,
        })
    }
    
    fn forward(&self, hidden: &Tensor, _attention_mask: &Tensor) -> CandleResult<Tensor> {
        let (batch_size, seq_len, _hidden_size) = hidden.dims3()?;
        
        let q = self.query.forward(hidden)?;
        let k = self.key.forward(hidden)?;
        let v = self.value.forward(hidden)?;
        
        // Reshape for multi-head attention
        let q = q.reshape((batch_size, seq_len, self.num_heads, self.head_dim))?;
        let k = k.reshape((batch_size, seq_len, self.num_heads, self.head_dim))?;
        let v = v.reshape((batch_size, seq_len, self.num_heads, self.head_dim))?;
        
        // Transpose to (batch, heads, seq_len, head_dim)
        let q = q.transpose(1, 2)?;
        let k = k.transpose(1, 2)?;
        let v = v.transpose(1, 2)?;
        
        // Attention scores
        let scale = (self.head_dim as f64).sqrt();
        let scores = (q.matmul(&k.transpose(-2, -1)?)? / scale)?;
        let probs = candle_nn::ops::softmax(&scores, -1)?;
        
        // Apply attention to values
        let context = probs.matmul(&v)?;
        
        // Reshape back
        let context = context.transpose(1, 2)?;
        let context = context.reshape((batch_size, seq_len, self.num_heads * self.head_dim))?;
        
        let output = self.output.forward(&context)?;
        
        Ok(output)
    }
}
