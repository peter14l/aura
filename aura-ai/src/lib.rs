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
    #[error("API error: {0}")]
    Api(String),
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
            .map_err(|e| AiError::Api(e.to_string()))?;

        let repo = api.model("Qwen/Qwen2.5-1.5B-Instruct-GGUF".to_string());
        let model_path = repo
            .get("qwen2.5-1.5b-instruct-q4_0.gguf")
            .await
            .map_err(|e| AiError::Api(e.to_string()))?;

        let mut file = std::fs::File::open(&model_path)?;
        let gguf = candle_core::quantized::gguf_file::Content::read(&mut file)?;

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
        let mut tokens_ids = tokens.get_ids().to_vec();

        let mut generated_ids = Vec::new();
        let eos_token_id = self.tokenizer.token_to_id("<|im_end|>").unwrap_or(0);

        for i in 0..180 {
            let input = Tensor::new(&tokens_ids[..], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, tokens_ids.len() - 1)?;
            let logits = logits.squeeze(0)?;
            let logits = logits.get(logits.dim(0)? - 1)?;

            // Greedy sampling
            let next_token_id = logits
                .to_vec1::<f32>()?
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i as u32)
                .unwrap_or(0);

            if next_token_id == eos_token_id {
                break;
            }

            tokens_ids.push(next_token_id);
            generated_ids.push(next_token_id);
        }

        let output = self
            .tokenizer
            .decode(&generated_ids, true)
            .map_err(AiError::Tokenizer)?;

        Ok(parse_bullets(&output))
    }
}

fn aura_model_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".aura")
        .join("models")
}

fn parse_bullets(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| {
            l.trim().starts_with('·')
                || l.trim().starts_with('-')
                || l.trim().starts_with('*')
                || l.trim().starts_with('1')
                || l.trim().starts_with('2')
                || l.trim().starts_with('3')
        })
        .map(|l| {
            l.trim_start_matches(|c: char| {
                c == '·' || c == '-' || c == '*' || c == ' ' || c.is_digit(10) || c == '.'
            })
            .trim()
            .to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

fn extract_main_text(html: &str) -> String {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);
    let body_sel = Selector::parse("article p, main p, .content p, [role='main'] p").unwrap();
    let fallback_sel = Selector::parse("p").unwrap();

    let paragraphs: Vec<String> = doc
        .select(&body_sel)
        .map(|el| el.text().collect::<String>())
        .filter(|t| t.len() > 40)
        .collect();

    if paragraphs.is_empty() {
        doc.select(&fallback_sel)
            .map(|el| el.text().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        paragraphs.join(" ")
    }
}

fn truncate_to_tokens(text: &str, max_tokens: usize) -> String {
    // Very rough approximation: 4 chars per token
    text.chars().take(max_tokens * 4).collect()
}
