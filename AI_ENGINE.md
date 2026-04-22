# Breathe AI Engine

Aura uses local AI inference to provide page summaries without sending data to the cloud.

## Models

We use quantized GGUF models running via the `candle` framework.

| Target Device | Recommended Model | Quantization | Est. VRAM/RAM |
|---|---|---|---|
| Budget Mobile (8GB) | `Qwen2.5 1.5B` | **Q4_0 (GGUF)** | ~0.9 GB |
| Budget Laptop (8GB) | `Phi-3 Mini 3.8B` | **Q4_K_S (GGUF)** | ~2.1 GB |
| High-end Desktop | `Phi-3 Medium 14B`| **Q4_K_M (GGUF)** | ~8.9 GB |

The default safe choice across platforms is `Qwen2.5 1.5B` at `Q4_0`, which loads quickly and consumes under 1GB of memory.

## Pipeline

1. **Extraction**: `scraper` extracts semantic content (`article p`, `main p`).
2. **Truncation**: Text is truncated to a safe context window (2048 tokens).
3. **Inference**: A strict prompt requests exactly 3 bullet points, under 20 words each.
4. **Presentation**: The Slint UI displays the bullets in the `BreatheOverlay` with a staggered fade-in animation.
