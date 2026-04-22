# 🌿 AURA — The browser that breathes with you

> **Aesthetic:** Subtractive Glassmorphism · **Stack:** 100% Rust · **Philosophy:** Sanctuary, not a tool.

Aura is a radically different approach to web browsing. It replaces the traditional tab-bar and heavy chrome with a "Stillness UI" — a spatial memory system where tabs exist as glowing orbs in a radial constellation, summoned only by intent. 

It is built entirely in Rust, featuring a hot-swappable rendering engine based on Servo, a localized AI inference engine for instant summaries, and a "Cookie Island" architecture for strict domain-level security.

## Core Pillars

1. **The Hollow Shell:** A borderless Tauri v2 window that loads and renders URLs via a hot-swappable Servo engine (`aura_engine.dll`).
2. **The Stillness UI:** Zero persistent horizontal tab strip. Elements expand and contract using fluid, breathing animations.
3. **Cookie Island (Silo Security):** Each registrable domain receives its own encrypted SQLite database, ensuring strict isolation.
4. **Breathe AI:** A local Candle inference engine running small models (like Qwen2.5 1.5B) to provide instant, concise page summaries without cloud APIs.
5. **The Infinite Update:** A Rendering Handoff Protocol that allows updating the core rendering engine without a single dropped frame.

## Documentation

- [Architecture & Crate Topology](ARCHITECTURE.md)
- [Design System & UI](DESIGN_SYSTEM.md)
- [Security & Cookie Islands](SECURITY.md)
- [AI Engine & Inference](AI_ENGINE.md)
- [Hot-Swap Engine](HOT_SWAP.md)

## Development

Aura is a Cargo workspace consisting of several crates. See `ARCHITECTURE.md` for details on the crate structure.

*Built in stillness. Shipped in Rust.*
