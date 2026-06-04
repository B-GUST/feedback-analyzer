use candle_core::{Device, Tensor, DType, Result as CandleResult};
use candle_nn::{VarBuilder, VarMap, AdamW, Optimizer, ParamsAdamW, Linear, Module};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Fine-tune sentiment model")]
struct Args {
    #[arg(long, default_value = "models/sentiment")]
    data_dir: String,
    
    #[arg(long, default_value = "models/sentiment/vocab.txt")]
    vocab_path: String,
    
    #[arg(long, default_value = "models/sentiment/trained")]
    output_dir: String,
    
    #[arg(long, default_value = "3")]
    epochs: usize,
    
    #[arg(long, default_value = "128")]
    batch_size: usize,
    
    #[arg(long, default_value = "0.00005")]
    learning_rate: f64,
    
    #[arg(long, default_value = "64")]
    max_length: usize,
    
    #[arg(long, default_value = "5000")]
    max_samples: usize,
}

#[derive(Debug, Clone)]
struct Dataset {
    texts: Vec<String>,
    labels: Vec<u32>,
}

impl Dataset {
    fn load(path: &str, max_samples: Option<usize>) -> Self {
        let file = File::open(path).expect("Failed to open dataset");
        let reader = BufReader::new(file);
        let mut texts = Vec::new();
        let mut labels = Vec::new();
        
        for line in reader.lines() {
            if let Some(max) = max_samples {
                if texts.len() >= max {
                    break;
                }
            }
            let line = line.expect("Failed to read line");
            if let Ok(item) = serde_json::from_str::<serde_json::Value>(&line) {
                if let (Some(text), Some(label)) = (item["text"].as_str(), item["label"].as_u64()) {
                    texts.push(text.to_string());
                    labels.push(label as u32);
                }
            }
        }
        Self { texts, labels }
    }
    
    fn len(&self) -> usize {
        self.texts.len()
    }
}

struct SentimentClassifier {
    embeddings: Tensor,
    fc1: Linear,
    fc2: Linear,
    classifier: Linear,
}

impl SentimentClassifier {
    fn new(vs: VarBuilder, vocab_size: usize, hidden_size: usize) -> CandleResult<Self> {
        // Create embeddings with proper initialization
        let embeddings = vs.get((vocab_size, hidden_size), "embeddings.weight")?;
        println!("Embeddings shape: {:?}", embeddings.shape());
        let fc1 = candle_nn::linear(hidden_size, hidden_size, vs.pp("fc1"))?;
        let fc2 = candle_nn::linear(hidden_size, hidden_size, vs.pp("fc2"))?;
        let classifier = candle_nn::linear(hidden_size, 2, vs.pp("classifier"))?;
        
        Ok(Self { embeddings, fc1, fc2, classifier })
    }
    
    fn forward(&self, input_ids: &Tensor, attention_mask: &Tensor) -> CandleResult<Tensor> {
        let batch_size = input_ids.dim(0)?;
        let seq_len = input_ids.dim(1)?;
        let hidden_size = self.embeddings.dim(1)?;
        
        // Make input_ids contiguous and flatten
        let flat_ids = input_ids.contiguous()?.reshape(batch_size * seq_len)?.to_dtype(DType::I64)?;
        let flat_embs = self.embeddings.index_select(&flat_ids, 0)?;
        let embeddings = flat_embs.reshape((batch_size, seq_len, hidden_size))?;
        
        let mask = attention_mask.contiguous()?.unsqueeze(2)?.to_dtype(DType::F32)?;
        let masked = embeddings.broadcast_mul(&mask)?;
        let sum_mask = mask.sum(1)?.clamp(1e-8, f32::MAX)?;
        let pooled = masked.sum(1)?.broadcast_div(&sum_mask)?;
        
        let h = self.fc1.forward(&pooled)?;
        let h = h.gelu()?;
        let h = self.fc2.forward(&h)?;
        let h = h.gelu()?;
        self.classifier.forward(&h)
    }
}

fn softmax_cross_entropy(logits: &Tensor, labels: &Tensor) -> CandleResult<Tensor> {
    let probs = candle_nn::ops::softmax(logits, 1)?;
    let log_probs = probs.log()?;
    let n_classes = log_probs.dim(1)?;
    let batch_size = labels.dim(0)?;
    
    // Create one-hot encoding
    let labels_exp = labels.unsqueeze(1)?.expand((batch_size, n_classes))?;
    let zeros = Tensor::zeros((batch_size, n_classes), DType::F32, logits.device())?;
    let ones = Tensor::ones((batch_size, n_classes), DType::F32, logits.device())?;
    let one_hot = zeros.contiguous()?.scatter_add(&labels_exp.contiguous()?, &ones.contiguous()?, 1)?;
    
    // Compute loss
    let loss = log_probs.broadcast_mul(&one_hot)?.sum(1)?;
    let loss = loss.neg()?;
    loss.mean(0)
}

fn tokenize_batch_simple<F>(texts: &[String], max_length: usize, tokenize_fn: &F) -> CandleResult<(Vec<u32>, Vec<u32>)> 
where
    F: Fn(&str, usize) -> (Vec<u32>, Vec<u32>),
{
    let mut all_ids = Vec::new();
    let mut all_masks = Vec::new();
    
    for text in texts {
        let (ids, mask) = tokenize_fn(text, max_length);
        all_ids.extend(ids);
        all_masks.extend(mask);
    }
    
    Ok((all_ids, all_masks))
}

fn to_tensor_1d(data: &[u32], device: &Device) -> CandleResult<Tensor> {
    let data_i64: Vec<i64> = data.iter().map(|&x| x as i64).collect();
    Tensor::new(data_i64.as_slice(), device)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Fine-tuning Sentiment Model ===");
    
    let train_data = Dataset::load(&format!("{}/data/train.jsonl", args.data_dir), Some(args.max_samples));
    let val_data = Dataset::load(&format!("{}/data/val.jsonl", args.data_dir), Some(1000));
    println!("Train: {}, Val: {}", train_data.len(), val_data.len());
    
    // Create a simple tokenizer from vocab.txt
    let vocab_content = std::fs::read_to_string(&args.vocab_path)
        .map_err(|e| candle_core::Error::Msg(format!("Failed to read vocab: {}", e)))?;
    
    let mut vocab: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for (idx, line) in vocab_content.lines().enumerate() {
        vocab.insert(line.trim().to_string(), idx as u32);
    }
    
    // Clone vocab for use in closure
    let vocab_clone = vocab.clone();
    
    // Simple tokenizer function
    let tokenize = move |text: &str, max_len: usize| -> (Vec<u32>, Vec<u32>) {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        
        let mut ids = vec![*vocab_clone.get("[CLS]").unwrap_or(&101)]; // CLS token
        
        for word in words {
            if ids.len() >= max_len - 1 {
                break;
            }
            // Try whole word first
            if let Some(&id) = vocab_clone.get(word) {
                ids.push(id);
            } else {
                // Try subword tokenization (simplified) using chars
                let chars: Vec<char> = word.chars().collect();
                let mut start = 0;
                while start < chars.len() {
                    let mut found = false;
                    for end in (start + 1..=chars.len()).rev() {
                        let substr: String = chars[start..end].iter().collect();
                        let token = if start == 0 { substr.clone() } else { format!("##{}", substr) };
                        if let Some(&id) = vocab_clone.get(&token) {
                            ids.push(id);
                            start = end;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        ids.push(*vocab_clone.get("[UNK]").unwrap_or(&100));
                        break;
                    }
                }
            }
        }
        
        ids.push(*vocab_clone.get("[SEP]").unwrap_or(&102)); // SEP token
        
        let orig_len = ids.len();
        ids.resize(max_len, 0);
        
        let mut mask = vec![1u32; orig_len];
        mask.resize(max_len, 0);
        
        (ids, mask)
    };
    
    let device = Device::Cpu;
    let varmap = VarMap::new();
    let vs = VarBuilder::from_varmap(&varmap, DType::F32, &device);
    
    let model = SentimentClassifier::new(vs, 30522, 128)?;
    
    let mut best_acc = 0.0;
    
    for epoch in 0..args.epochs {
        println!("\n--- Epoch {}/{} ---", epoch + 1, args.epochs);
        
        let adamw_params = ParamsAdamW { lr: args.learning_rate, ..Default::default() };
        let mut optimizer = AdamW::new(varmap.all_vars(), adamw_params)?;
        
        let mut total_loss = 0.0;
        let mut correct = 0;
        let mut total = 0;
        let mut batch_count = 0;
        
        for batch_start in (0..train_data.len()).step_by(args.batch_size) {
            let batch_end = (batch_start + args.batch_size).min(train_data.len());
            let batch_texts = &train_data.texts[batch_start..batch_end];
            let batch_labels: Vec<i64> = train_data.labels[batch_start..batch_end].iter().map(|&x| x as i64).collect();
            
            let (flat_ids, flat_masks) = tokenize_batch_simple(batch_texts, args.max_length, &tokenize)?;
            
            let ids_i64: Vec<i64> = flat_ids.iter().map(|&x| x as i64).collect();
            let masks_i64: Vec<i64> = flat_masks.iter().map(|&x| x as i64).collect();
            
            let token_ids = Tensor::new(ids_i64.as_slice(), &device)?.reshape((batch_texts.len(), args.max_length))?;
            let attention_mask = Tensor::new(masks_i64.as_slice(), &device)?.reshape((batch_texts.len(), args.max_length))?;
            let labels = Tensor::new(batch_labels.as_slice(), &device)?;
            
            let logits = model.forward(&token_ids, &attention_mask)?;
            let loss = softmax_cross_entropy(&logits, &labels)?;
            
            optimizer.backward_step(&loss)?;
            
            total_loss += loss.to_scalar::<f32>()?;
            let preds = logits.argmax(1)?.to_dtype(DType::I64)?;
            correct += preds.eq(&labels)?.to_vec1::<u8>()?.iter().map(|&x| x as usize).sum::<usize>();
            total += batch_labels.len();
            batch_count += 1;
            
            if batch_count % 50 == 0 {
                println!("  Batch {}: loss={:.4}, acc={:.2}%", batch_count, total_loss / batch_count as f32, 100.0 * correct as f32 / total as f32);
            }
        }
        
        println!("Train: loss={:.4}, acc={:.2}%", total_loss / batch_count as f32, 100.0 * correct as f32 / total as f32);
        
        // Eval
        let mut val_correct = 0;
        let mut val_total = 0;
        
        for batch_start in (0..val_data.len()).step_by(args.batch_size) {
            let batch_end = (batch_start + args.batch_size).min(val_data.len());
            let batch_texts = &val_data.texts[batch_start..batch_end];
            let batch_labels: Vec<i64> = val_data.labels[batch_start..batch_end].iter().map(|&x| x as i64).collect();
            
            let (flat_ids, flat_masks) = tokenize_batch_simple(batch_texts, args.max_length, &tokenize)?;
            let ids_i64: Vec<i64> = flat_ids.iter().map(|&x| x as i64).collect();
            let masks_i64: Vec<i64> = flat_masks.iter().map(|&x| x as i64).collect();
            
            let token_ids = Tensor::new(ids_i64.as_slice(), &Device::Cpu)?.reshape((batch_texts.len(), args.max_length))?;
            let attention_mask = Tensor::new(masks_i64.as_slice(), &Device::Cpu)?.reshape((batch_texts.len(), args.max_length))?;
            let labels = Tensor::new(batch_labels.as_slice(), &Device::Cpu)?;
            
            let logits = model.forward(&token_ids, &attention_mask)?;
            let preds = logits.argmax(1)?.to_dtype(DType::I64)?;
            
            val_correct += preds.eq(&labels)?.to_vec1::<u8>()?.iter().map(|&x| x as usize).sum::<usize>();
            val_total += batch_labels.len();
        }
        
        let val_acc = 100.0 * val_correct as f32 / val_total as f32;
        println!("Val: acc={:.2}%", val_acc);
        
        if val_acc > best_acc {
            best_acc = val_acc;
            let output_dir = PathBuf::from(&args.output_dir);
            std::fs::create_dir_all(&output_dir)?;
            varmap.save(output_dir.join("model.safetensors"))?;
            println!("  Saved model");
        }
    }
    
    println!("\n=== Done! Best val acc: {:.2}% ===", best_acc);
    Ok(())
}
