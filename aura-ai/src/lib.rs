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
        // Select device: Metal on macOS, CPU on Android/Linux without CUDA
        let device = if cfg!(target_os = "macos") {
            Device::new_metal(0).unwrap_or(Device::Cpu)
        } else {
            Device::Cpu
        };

        // Pseudo implementation for loading model
        // let model_path = ...
        // let tokenizer = ...

        // Ok(Self { model, tokenizer, device })
        Err(AiError::Api)
    }

    pub async fn summarise(&mut self, html: &str) -> Result<Vec<String>, AiError> {
        let clean_text = extract_main_text(html);
        let _truncated = truncate_to_tokens(&clean_text, 2048);

        // Pseudo generation
        Ok(vec![
            "Aura is a minimalist browser built in Rust.".into(),
            "It uses a Stillness UI and Cookie Island architecture.".into(),
            "Local AI generates these summaries securely.".into(),
        ])
    }
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
