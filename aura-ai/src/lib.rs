// aura-ai/src/lib.rs

use candle_core::{Device, Tensor};
use candle_transformers::models::qwen2::{Config, Model};
use std::path::PathBuf;
use tokenizers::Tokenizer;

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Candle error: {0}")]
    Candle(#[from] candle_core::Error),
    #[error("Tokenizer error: {0}")]
    Tokenizer(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("API error")]
    Api,
}

pub struct AiEngine {
    model: Model,
    tokenizer: Tokenizer,
    device: Device,
}

impl AiEngine {
    pub async fn load() -> Result<Self, AiError> {
        let device = if cfg!(target_os = "macos") {
            Device::new_metal(0).unwrap_or(Device::Cpu)
        } else {
            Device::Cpu
        };

        let api = hf_hub::api::tokio::ApiBuilder::new()
            .with_cache_dir(aura_model_dir())
            .build()
            .map_err(|_| AiError::Api)?;

        let repo = api.model("Qwen/Qwen2.5-1.5B-Instruct-GGUF".to_string());
        let model_path = repo
            .get("qwen2.5-1.5b-instruct-q4_0.gguf")
            .await
            .map_err(|_| AiError::Api)?;

        let mut file = std::fs::File::open(&model_path)?;
        let gguf = candle_core::quantized::gguf_file::Content::read(&mut file)?;

        // Note: candle-transformers version might vary, this is a general structure
        let model = Model::from_gguf(gguf, &mut file, &device)?;
        let tokenizer = Tokenizer::from_pretrained("Qwen/Qwen2.5-1.5B-Instruct", None)
            .map_err(AiError::Tokenizer)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    pub async fn summarise(&mut self, html: &str) -> Result<Vec<String>, AiError> {
        let clean_text = extract_main_text(html);
        let truncated = truncate_to_tokens(&clean_text, 2048);

        let prompt = format!(
            "<|im_start|>system\nYou are a concise summarizer. Respond with exactly 3 bullet points, each under 20 words. No preamble.<|im_end|>\n<|im_start|>user\nSummarise this page:\n{}<|im_end|>\n<|im_start|>assistant\n",
            truncated
        );

        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(AiError::Tokenizer)?;
        let input = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;

        // Simple greedy generation (simplified for skeleton)
        let mut generated = vec![];
        for _ in 0..180 {
            let logits = self.model.forward(&input, generated.len())?;
            // Sampling logic would go here
            // let next_token = ...
            // generated.push(next_token);
            break; // Placeholder for actual generation loop
        }

        let output = self
            .tokenizer
            .decode(&generated, true)
            .map_err(AiError::Tokenizer)?;
        Ok(parse_bullets(&output))
    }
}

fn aura_model_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".aura").join("models")
}

fn parse_bullets(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| l.starts_with("·") || l.starts_with("-") || l.starts_with("*"))
        .map(|l| {
            l.trim_start_matches(|c| c == '·' || c == '-' || c == '*' || c == ' ')
                .to_string()
        })
        .collect()
}

fn extract_main_text(html: &str) -> String {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);
    let fallback_sel = Selector::parse("p").unwrap();
    doc.select(&fallback_sel)
        .map(|el| el.text().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_to_tokens(text: &str, max_tokens: usize) -> String {
    text.chars().take(max_tokens * 4).collect() // Very rough approximation
}
