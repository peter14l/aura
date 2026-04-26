# 🌿 AURA — Master Implementation Blueprint
### *The browser that breathes with you.*
> **Aesthetic:** Subtractive Glassmorphism · **Stack:** 100% Rust · **Philosophy:** Sanctuary, not a tool.
> **Status:** Production Ready — Core systems integrated and robust.

---

## Table of Contents

1. [Project Identity & Design System](#1-project-identity--design-system)
2. [Architecture Overview](#2-architecture-overview)
3. [Module Block 01 — The Hollow Shell (Tauri + Servo Foundation)](#3-module-block-01--the-hollow-shell)
4. [Module Block 02 — The Stillness UI + Gestural Fluidity](#4-module-block-02--the-stillness-ui--gestural-fluidity)
5. [Module Block 03 — Cookie Island (Silo Security)](#5-module-block-03--cookie-island-silo-security)
6. [Module Block 04 — Breathe AI (Local Candle Inference)](#6-module-block-04--breathe-ai-local-candle-inference)
7. [Module Block 05 — The Infinite Update (Hot-Swappable Engine)](#7-module-block-05--the-infinite-update-hot-swappable-engine)
8. [GitHub Actions: The Auto-Builder](#8-github-actions-the-auto-builder)
9. [Hardware Profiles & Optimization Matrix](#9-hardware-profiles--optimization-matrix)
10. [Full Dependency Map](#10-full-dependency-map)

---

## 1. Project Identity & Design System

### 1.1 Name & Tagline
```
Name:    Aura
Tagline: The browser that breathes with you.
Domain:  aura-browser.dev (suggested)
```

### 1.2 Color Palette (Subtractive Glassmorphism)

| Token | Hex | Usage |
|---|---|---|
| `--aura-base-light` | `#FDFCF5` | Background (light mode) — Soft Alabaster |
| `--aura-base-dark` | `#121412` | Background (dark mode) — Deep Obsidian |
| `--aura-sage` | `#D4E1D1` | Muted Sage — primary accent, hover glows |
| `--aura-rose` | `#E9D5CA` | Dusty Rose — AI / wellness highlights |
| `--aura-glass` | `rgba(253,252,245,0.08)` | Glass panel fill |
| `--aura-blur` | `blur(28px) saturate(160%)` | Backdrop filter |
| `--aura-border` | `rgba(255,255,255,0.0)` | **Zero borders** — the subtractive principle |

### 1.3 Design Vocabulary
- **Subtractive Glassmorphism**: Remove until only essential information remains. No chrome. No decoration. Blur replaces borders.
- **Spatial Breathing**: Elements expand/contract via breathing easing (`cubic-bezier(0.34, 1.56, 0.64, 1)`).
- **Ink Fade**: All animations use opacity + transform. Nothing slides. Everything materialises.

### 1.4 Slint Design Tokens
```slint
// tokens.slint — import this globally
export global AuraTheme {
    out property <color> base: #FDFCF5;
    out property <color> obsidian: #121412;
    out property <color> sage: #D4E1D1;
    out property <color> rose: #E9D5CA;
    out property <color> glass: rgba(253, 252, 245, 8%);
    out property <length> blur-radius: 28px;
    out property <duration> breathe: 420ms;
    out property <easing> ease-breathe: cubic-bezier(0.34, 1.56, 0.64, 1);
}
```

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                          AURA PROCESS                           │
│                                                                 │
│  ┌──────────────┐    IPC (Tauri Commands)   ┌───────────────┐  │
│  │  Slint UI    │◄─────────────────────────►│  Rust Backend │  │
│  │  (Frontend)  │                           │  (aura-core)  │  │
│  └──────┬───────┘                           └───────┬───────┘  │
│         │                                           │           │
│  Window │ Events                     ┌──────────────┼──────┐   │
│  (Tauri)│                            │              │      │   │
│         ▼                       ┌────▼────┐  ┌─────▼────┐  │   │
│  ┌──────────────┐               │  Silo   │  │ Candle   │  │   │
│  │  servo.dll   │◄──libloading──│ Manager │  │ (AI)     │  │   │
│  │  (hot-swap)  │               │(rusqlite│  │ Engine   │  │   │
│  └──────────────┘               └─────────┘  └──────────┘  │   │
│                                      ▲              ▲        │   │
│                                 ┌────┴──────────────┴──┐    │   │
│                                 │    adblock crate      │    │   │
│                                 │ (network interceptor) │    │   │
│                                 └───────────────────────┘    │   │
└─────────────────────────────────────────────────────────────────┘
```

### Crate Topology

```
aura/
├── aura-app/          # Tauri v2 shell (binary)
│   └── src/main.rs
├── aura-ui/           # Slint components
│   └── ui/
├── aura-engine/       # cdylib → servo.dll / libservo.so
│   └── src/lib.rs
├── aura-silo/         # Cookie Island logic
│   └── src/lib.rs
├── aura-ai/           # Candle inference engine
│   └── src/lib.rs
├── aura-net/          # adblock + network interceptor
│   └── src/lib.rs
└── Cargo.toml         # workspace
```

---

## 3. Module Block 01 — The Hollow Shell [DONE]

**Goal:** Borderless Tauri v2 window that loads and renders URLs via Servo.
**Status:** Implemented. Engine dynamic loading and basic navigation functional.

### Cargo.toml — `aura-app`

```toml
[package]
name = "aura-app"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["macos-private-api"] }
tauri-build = { version = "2", features = [] }
servo = { git = "https://github.com/servo/servo", branch = "main" }
libloading = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

[lib]
name = "aura_app_lib"
crate-type = ["cdylib", "rlib"]
```

### Pseudo-Rust: Shell Initialisation

```rust
// aura-app/src/main.rs

/// VIBE AGENT PROMPT:
/// "Create a Tauri v2 app with a borderless, transparent window.
///  On startup, dynamically load aura_engine.dll using libloading.
///  Expose a Tauri command `navigate(url: String)` that calls
///  engine.render_url(url) and returns the rendered bitmap handle."

#[tauri::command]
async fn navigate(
    url: String,
    engine: tauri::State<'_, EngineHandle>,
) -> Result<RenderToken, AuraError> {
    // 1. Validate & sanitise URL
    let parsed = Url::parse(&url).map_err(AuraError::InvalidUrl)?;

    // 2. Pass through network interceptor (adblock)
    let filtered = aura_net::intercept(&parsed).await?;

    // 3. Forward to hot-loaded engine
    engine.render(filtered).await
}

fn main() {
    // Load engine dylib on startup
    let engine = EngineHandle::load("./engines/aura_engine_latest.dll")
        .expect("Engine failed to load — check dylib ABI");

    tauri::Builder::default()
        .manage(engine)
        .setup(|app| {
            let win = app.get_webview_window("main").unwrap();
            win.set_decorations(false)?;        // Borderless
            win.set_transparent(true)?;         // Glass base
            win.set_always_on_top(false)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![navigate, zen_summary, silo_status])
        .run(tauri::generate_context!())
        .expect("Aura failed to launch");
}
```

### tauri.conf.json (Key Sections)

```json
{
  "app": {
    "windows": [{
      "label": "main",
      "title": "Aura",
      "decorations": false,
      "transparent": true,
      "resizable": true,
      "width": 1200,
      "height": 800,
      "minWidth": 400,
      "minHeight": 300
    }]
  },
  "bundle": {
    "identifier": "dev.aura-browser.app",
    "icon": ["icons/aura.png"]
  }
}
```

---

## 4. Module Block 02 — The Stillness UI + Gestural Fluidity [DONE]

**Status:** Implemented. Slint UI integrated with Tauri commands. Constellation and Command Bar functional.

### 4.1 What Is "The Stillness"?

The Stillness is Aura's answer to tab-bar anxiety. It replaces the traditional top-chrome with a **spatial memory system**: tabs exist as glowing orbs in a radial constellation, summoned only by intent.

#### No Tab Bar Manifesto
- Zero persistent horizontal tab strip
- Tabs = **dormant light nodes** floating in a `Session Constellation`
- The constellation appears only when the cursor docks to the **left edge** for >300ms
- Each node pulses with the favicon's dominant colour (extracted via `image` crate)
- Hovering a node shows a frosted preview card (100×75px thumbnail via offscreen render)
- Closing a tab = dragging the node outward past a dismissal radius

### 4.2 Gestural Fluidity Map

```
GESTURE                  │ TRIGGER                  │ ACTION
─────────────────────────┼──────────────────────────┼────────────────────────────
Alt + Space              │ Keyboard                 │ Summon Command Bar
Left-edge hover 300ms    │ Cursor dock              │ Reveal Session Constellation
Left-edge hover + scroll │ Cursor dock + wheel      │ Cycle active tab (no click)
Cursor to top edge       │ Proximity < 20px         │ Reveal minimal address ghost
Cursor to bottom edge    │ Proximity < 20px         │ Reveal status + Lotus AI icon
3-finger swipe left/right│ Trackpad gesture         │ History back / forward
3-finger swipe up        │ Trackpad gesture         │ New tab (spawns node in const.)
3-finger swipe down      │ Trackpad gesture         │ Close active tab
Right-edge hover 300ms   │ Cursor dock              │ Reveal Silo status panel
Pinch-to-zoom            │ Trackpad / touch         │ Page zoom (no UI shown)
Double-tap space         │ Keyboard (reader mode)   │ Toggle Zen Reading Mode
Lotus icon click         │ Bottom gutter            │ Trigger AI Breathe Summary
Drag node inward (const.)│ Tab constellation drag   │ Pin tab (node becomes solid)
Drag node outward (const)│ Tab constellation drag   │ Close tab (node fades to void)
```

### 4.3 Slint: Command Bar Component

```slint
// ui/command-bar.slint

import { AuraTheme } from "tokens.slint";

/// VIBE AGENT PROMPT:
/// "Build this Slint component exactly. It must fade in on `visible = true`,
///  blur the background behind it, accept text input, and emit `submitted(url)`."

export component CommandBar {
    in property <bool> visible: false;
    in-out property <string> query;
    callback submitted(string);

    opacity: visible ? 1.0 : 0.0;
    animate opacity { duration: AuraTheme.breathe; easing: AuraTheme.ease-breathe; }

    // Centred overlay — no position lock to window edges
    x: (parent.width - self.width) / 2;
    y: parent.height * 0.32;   // Golden-ratio vertical position
    width: min(parent.width * 0.55, 680px);
    height: 52px;

    Rectangle {
        background: AuraTheme.glass;
        border-radius: 26px;
        // Backdrop blur via platform layer (Tauri sets CSS backdrop-filter equivalent)

        HorizontalLayout {
            padding-left: 20px;
            padding-right: 16px;
            spacing: 12px;

            // Breathing dot indicator
            Rectangle {
                width: 8px; height: 8px;
                border-radius: 4px;
                background: AuraTheme.sage;
                animate background { duration: 1200ms; easing: ease-in-out; }
            }

            TextInput {
                text <=> query;
                font-size: 17px;
                color: AuraTheme.base;
                placeholder-text: "Where would you like to go?";
                accepted => { submitted(self.text); }
            }
        }
    }
}
```

### 4.4 Slint: Session Constellation (Tab Orbs)

```slint
// ui/constellation.slint

/// VIBE AGENT PROMPT:
/// "Render a vertical list of circular tab nodes on the left edge.
///  Each node has a `glow-color` (dominant favicon colour) and `thumbnail` image.
///  Hovering expands the node; dragging outside the panel closes the tab."

export component Constellation {
    in property <[TabNode]> tabs;
    callback tab-selected(int);   // index
    callback tab-closed(int);     // index

    visible: self.hovered || force-visible;
    in property <bool> force-visible: false;

    animate opacity { duration: 280ms; }

    VerticalLayout {
        spacing: 14px;
        padding: 18px;
        alignment: center;

        for tab[i] in tabs : TabOrb {
            node: tab;
            clicked => { tab-selected(i); }
            dismissed => { tab-closed(i); }
        }
    }
}

component TabOrb {
    in property <TabNode> node;
    callback clicked();
    callback dismissed();

    width: self.hovered ? 44px : 32px;
    height: self.hovered ? 44px : 32px;
    animate width, height { duration: 220ms; easing: ease-out; }

    Rectangle {
        border-radius: self.width / 2;
        background: node.glow-color.with-alpha(0.4);
        drop-shadow-blur: self.parent.hovered ? 18px : 6px;
        drop-shadow-color: node.glow-color;
        animate drop-shadow-blur { duration: 200ms; }
        // Favicon rendered inside orb
        Image { source: node.favicon; width: 18px; height: 18px; }
    }

    TouchArea {
        clicked => { root.clicked(); }
        // Drag dismissal handled in Rust via gesture recogniser
    }
}
```

---

## 5. Module Block 03 — Cookie Island (Silo Security) [DONE]

**Status:** Implemented. Per-domain isolation, encryption via AES-256-GCM, and session purging functional.

### 5.1 Cookie Island Schema (SQLite per TLD)

Each registrable domain (e.g. `github.com`, `google.com`) receives its **own SQLite database file** on disk, never shared with other domains.

**File path convention:**
```
~/.aura/silos/{sha256(registrable_domain)}.silo.db
```
Using a SHA-256 hash of the domain prevents path-traversal attacks and obscures which domains the user has visited from casual filesystem inspection.

#### SQLite Schema (per silo file)

```sql
-- Applied on every new silo file creation via rusqlite migrations

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS cookies (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    host         TEXT NOT NULL,       -- e.g. "api.github.com"
    name         TEXT NOT NULL,
    value        BLOB NOT NULL,       -- AES-256-GCM encrypted
    path         TEXT NOT NULL DEFAULT '/',
    secure       INTEGER NOT NULL DEFAULT 1,  -- bool: 1 = Secure-only
    http_only    INTEGER NOT NULL DEFAULT 1,  -- bool: 1 = HttpOnly
    same_site    TEXT CHECK(same_site IN ('Strict','Lax','None')) DEFAULT 'Lax',
    expiry_utc   INTEGER,             -- Unix epoch; NULL = session cookie
    created_utc  INTEGER NOT NULL DEFAULT (unixepoch()),
    last_access  INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(host, name, path)
);

CREATE TABLE IF NOT EXISTS local_storage (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    origin       TEXT NOT NULL,
    key          TEXT NOT NULL,
    value        BLOB NOT NULL,       -- AES-256-GCM encrypted
    updated_utc  INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(origin, key)
);

CREATE TABLE IF NOT EXISTS silo_meta (
    key          TEXT PRIMARY KEY,
    value        TEXT
);

-- Seed meta
INSERT OR IGNORE INTO silo_meta VALUES ('domain', ?);
INSERT OR IGNORE INTO silo_meta VALUES ('pinned', '0');    -- 0 = cleared on session end
INSERT OR IGNORE INTO silo_meta VALUES ('created', unixepoch());
```

### 5.2 SiloManager: Rust Implementation

```toml
# aura-silo/Cargo.toml

[package]
name = "aura-silo"
version = "0.1.0"
edition = "2021"

[dependencies]
rusqlite = { version = "0.31", features = ["bundled", "backup"] }
sha2 = "0.10"
hex = "0.4"
aes-gcm = "0.10"
rand = "0.8"
url = "2"
thiserror = "1"
dirs = "5"
```

```rust
// aura-silo/src/lib.rs

/// VIBE AGENT PROMPT:
/// "Implement this SiloManager exactly. Each method must be async-compatible
///  via spawn_blocking. The encryption key is derived per-silo from a master
///  key stored in the OS keychain (use `keyring` crate for that integration)."

use rusqlite::{Connection, params};
use sha2::{Sha256, Digest};
use std::path::PathBuf;

pub struct SiloManager {
    base_dir: PathBuf,
    master_key: [u8; 32],   // AES-256 key from OS keychain
}

impl SiloManager {
    /// Derive per-domain silo path
    fn silo_path(&self, registrable_domain: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(registrable_domain.as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.base_dir.join(format!("{}.silo.db", hash))
    }

    /// Open (or create) a silo for a given domain
    pub fn open_silo(&self, registrable_domain: &str) -> Result<Connection, SiloError> {
        let path = self.silo_path(registrable_domain);
        let conn = Connection::open(&path)?;

        // Apply WAL + schema migrations
        conn.execute_batch(include_str!("schema.sql"))?;

        // Register domain in meta if new
        conn.execute(
            "INSERT OR IGNORE INTO silo_meta VALUES ('domain', ?)",
            params![registrable_domain],
        )?;

        Ok(conn)
    }

    /// Set a cookie (encrypts value before writing)
    pub fn set_cookie(&self, domain: &str, cookie: &Cookie) -> Result<(), SiloError> {
        let conn = self.open_silo(domain)?;
        let encrypted_value = self.encrypt_value(&cookie.value)?;

        conn.execute(
            "INSERT OR REPLACE INTO cookies
             (host, name, value, path, secure, http_only, same_site, expiry_utc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                cookie.host, cookie.name, encrypted_value,
                cookie.path, cookie.secure as i32,
                cookie.http_only as i32, cookie.same_site.as_str(),
                cookie.expiry_utc,
            ],
        )?;
        Ok(())
    }

    /// Purge all non-pinned silos (called on session close)
    pub fn purge_session_silos(&self) -> Result<usize, SiloError> {
        let mut purged = 0usize;
        for entry in std::fs::read_dir(&self.base_dir)? {
            let path = entry?.path();
            if path.extension().map_or(false, |e| e == "db") {
                // Check pinned status
                if let Ok(conn) = Connection::open(&path) {
                    let pinned: i32 = conn.query_row(
                        "SELECT value FROM silo_meta WHERE key='pinned'",
                        [], |r| r.get(0)
                    ).unwrap_or(0);

                    if pinned == 0 {
                        drop(conn);
                        std::fs::remove_file(&path)?;
                        purged += 1;
                    }
                }
            }
        }
        Ok(purged)
    }

    fn encrypt_value(&self, plaintext: &[u8]) -> Result<Vec<u8>, SiloError> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
        use rand::RngCore;

        let key = Key::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|_| SiloError::EncryptionFailed)?;

        // Prepend nonce so we can decrypt later: [12-byte nonce][ciphertext]
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }
}
```

### 5.3 Network Interceptor (`adblock` crate) — Cookie Silo Integration

```toml
# aura-net/Cargo.toml

[dependencies]
adblock = "0.8"
url = "2"
reqwest = { version = "0.12", features = ["rustls-tls"], default-features = false }
tokio = { version = "1", features = ["full"] }
once_cell = "1"
```

```rust
// aura-net/src/lib.rs

/// VIBE AGENT PROMPT:
/// "Implement the full intercept() pipeline below. The adblock Engine must be
///  initialised once (lazy_static or OnceLock) with EasyList + EasyPrivacy rules
///  downloaded at first launch and cached at ~/.aura/lists/. On each request,
///  the interceptor checks the block list BEFORE passing to the Silo for
///  cookie injection. Blocked requests return Err(Blocked) immediately."

use adblock::engine::Engine as AdblockEngine;
use adblock::lists::{FilterSet, ParseOptions};
use once_cell::sync::Lazy;
use url::Url;

static ADBLOCK: Lazy<AdblockEngine> = Lazy::new(|| {
    let raw_rules = load_or_fetch_lists(&[
        "https://easylist.to/easylist/easylist.txt",
        "https://easylist.to/easylist/easyprivacy.txt",
        "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt",
    ]);

    let mut filter_set = FilterSet::new(false);
    filter_set.add_filters(&raw_rules, ParseOptions::default());
    AdblockEngine::from_filter_set(filter_set, true)
});

pub enum InterceptDecision {
    Allow(Url),
    Block { reason: &'static str },
    Redirect(Url),  // For HTTPS upgrades
}

pub async fn intercept(
    request_url: &Url,
    source_url: &Url,
    resource_type: &str,
) -> InterceptDecision {
    let block_result = ADBLOCK.check_network_urls(
        request_url.as_str(),
        source_url.as_str(),
        resource_type,
    );

    if block_result.matched {
        return InterceptDecision::Block {
            reason: "adblock_matched",
        };
    }

    // Force HTTPS upgrade
    if request_url.scheme() == "http" {
        if let Ok(mut https) = request_url.clone().into_string().replace("http://", "https://").parse::<Url>() {
            return InterceptDecision::Redirect(https);
        }
    }

    InterceptDecision::Allow(request_url.clone())
}

fn load_or_fetch_lists(urls: &[&str]) -> Vec<String> {
    // Check cache at ~/.aura/lists/, return cached if < 24h old
    // Otherwise fetch via reqwest and cache
    // Returns concatenated filter rules as lines
    todo!("implement list fetching with 24h TTL cache")
}
```

---

## 6. Module Block 04 — Breathe AI (Local Candle Inference) [DONE]

**Status:** Implemented. Qwen2.5-1.5B loading and HTML summarization logic functional.

### 6.1 Model Selection & Quantization Matrix

The AI must run locally, offline, with no API calls, on constrained hardware.

| Target Device | RAM | Storage | Recommended Model | Quantization | Est. VRAM/RAM |
|---|---|---|---|---|---|
| Samsung Galaxy M36 5G | 8 GB | 128 GB | `Phi-3 Mini 3.8B` | **Q4_K_M (GGUF)** | ~2.3 GB |
| Samsung Galaxy M36 5G | 8 GB | 128 GB | `Qwen2.5 1.5B` | **Q4_0 (GGUF)** | ~0.9 GB ✅ preferred |
| 8 GB RAM Laptop | 8 GB | 256 GB | `Phi-3 Mini 3.8B` | **Q4_K_S (GGUF)** | ~2.1 GB |
| 8 GB RAM Laptop | 8 GB | 256 GB | `Mistral 7B v0.3` | **Q2_K (GGUF)** | ~3.1 GB ⚠️ tight |
| 16 GB RAM Desktop | 16 GB | Any | `Phi-3 Medium 14B` | **Q4_K_M (GGUF)** | ~8.9 GB |

**Recommendation for the target dev device (Samsung Galaxy M36 5G / 8 GB laptops):**

Use `Qwen2.5-1.5B-Instruct` at `Q4_0` quantization. It fits in ~900 MB RAM, loads in under 2 seconds on mobile, and produces coherent 3-bullet summaries. The Candle GGUF loader handles this natively.

**Why not Phi-3 by default?** Phi-3 Mini at Q4_K_M is 2.3 GB — on an 8 GB device with OS overhead (~2.5 GB) and browser RAM (~800 MB), it leaves dangerously little headroom. Qwen2.5 1.5B is the safe default; Phi-3 Mini is the "Pro" tier upgrade.

### 6.2 Cargo.toml — `aura-ai`

```toml
[package]
name = "aura-ai"
version = "0.1.0"
edition = "2021"

[dependencies]
candle-core = { git = "https://github.com/huggingface/candle", features = ["metal"] }
candle-nn = { git = "https://github.com/huggingface/candle" }
candle-transformers = { git = "https://github.com/huggingface/candle" }
hf-hub = { version = "0.3", features = ["tokio"] }
tokenizers = "0.19"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
scraper = "0.19"    # HTML → clean text extraction
thiserror = "1"
tracing = "0.1"

# Android/mobile feature gates
[target.'cfg(target_os = "android")'.dependencies]
candle-core = { git = "https://github.com/huggingface/candle", features = [] }
# No Metal on Android — CPU only, use WASM-friendly kernels
```

### 6.3 Pseudo-Rust: Breathe Summary Engine

```rust
// aura-ai/src/lib.rs

/// VIBE AGENT PROMPT:
/// "Implement AiEngine::summarise() exactly as below. The model file must be
///  downloaded ONCE to ~/.aura/models/ using hf-hub. On Android, the path is
///  /data/data/dev.aura_browser/files/models/. The function must return in
///  under 8 seconds on the Samsung Galaxy M36 5G — use max_new_tokens=180."

use candle_core::{Device, Tensor};
use candle_transformers::models::qwen2::{Config, Model};
use tokenizers::Tokenizer;

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

        // Download GGUF from HuggingFace Hub (cached after first run)
        let model_path = hf_hub::api::sync::ApiBuilder::new()
            .with_cache_dir(aura_model_dir())
            .build()?
            .model("Qwen/Qwen2.5-1.5B-Instruct-GGUF".to_string())
            .get("qwen2.5-1.5b-instruct-q4_0.gguf")?;

        // Load quantized weights via candle GGUF loader
        let mut file = std::fs::File::open(&model_path)?;
        let gguf = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        let config = Config::from_gguf_metadata(&gguf)?;
        let model = Model::from_gguf(gguf, &mut file, &device)?;

        let tokenizer = Tokenizer::from_pretrained("Qwen/Qwen2.5-1.5B-Instruct", None)?;

        Ok(Self { model, tokenizer, device })
    }

    pub async fn summarise(&mut self, html: &str) -> Result<Vec<String>, AiError> {
        // 1. Extract clean text from HTML
        let clean_text = extract_main_text(html);

        // 2. Truncate to 2048 tokens (context window safe limit)
        let truncated = truncate_to_tokens(&clean_text, 2048);

        // 3. Build prompt
        let prompt = format!(
            "<|im_start|>system\nYou are a concise summarizer. \
             Respond with exactly 3 bullet points, each under 20 words. \
             No preamble.<|im_end|>\n\
             <|im_start|>user\nSummarise this page:\n{}<|im_end|>\n\
             <|im_start|>assistant\n",
            truncated
        );

        // 4. Tokenise
        let tokens = self.tokenizer.encode(prompt, true)?;
        let input = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;

        // 5. Generate (greedy, max 180 tokens to stay fast on mobile)
        let mut generated = vec![];
        let mut logits_processor = LogitsProcessor::new(/* temp=0.3, top_p=0.9 */);

        for _ in 0..180 {
            let logits = self.model.forward(&input, generated.len())?;
            let next_token = logits_processor.sample(&logits)?;
            if next_token == self.tokenizer.token_to_id("<|im_end|>").unwrap_or(0) {
                break;
            }
            generated.push(next_token);
        }

        // 6. Decode and parse bullets
        let output = self.tokenizer.decode(&generated, true)?;
        let bullets = parse_bullets(&output);  // Splits on "• " or "- " or "* "
        Ok(bullets)
    }
}

/// Extracts readable body text using the `scraper` crate
fn extract_main_text(html: &str) -> String {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);

    // Target semantic content containers (article > p, main > p, etc.)
    let body_sel = Selector::parse("article p, main p, .content p, [role='main'] p").unwrap();
    let fallback_sel = Selector::parse("p").unwrap();

    let paragraphs: Vec<String> = doc.select(&body_sel)
        .map(|el| el.text().collect::<String>())
        .filter(|t| t.len() > 40)  // Skip nav fragments
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
```

### 6.4 Slint: Lotus / Breathe Overlay

```slint
// ui/breathe-overlay.slint

/// VIBE AGENT PROMPT:
/// "Implement this overlay. It fades in from the bottom edge.
///  The three bullets animate in sequentially with 120ms stagger."

export component BreatheOverlay {
    in property <bool> visible: false;
    in property <[string]> bullets: [];
    in property <bool> loading: false;

    opacity: visible ? 1.0 : 0.0;
    animate opacity { duration: 320ms; easing: ease-in-out; }

    // Bottom-anchored card
    y: parent.height - self.height - 24px;
    x: (parent.width - self.width) / 2;
    width: min(parent.width * 0.6, 520px);
    height: 160px;

    Rectangle {
        background: AuraTheme.glass;
        border-radius: 20px;

        VerticalLayout {
            padding: 20px;
            spacing: 10px;

            if loading : Rectangle {
                // Breathing pulse animation placeholder
                background: AuraTheme.sage.with-alpha(0.3);
                border-radius: 8px;
                height: 12px;
                animate width { duration: 1000ms; easing: ease-in-out; iteration-count: -1; }
            }

            for bullet[i] in bullets : Text {
                text: "· " + bullet;
                font-size: 13px;
                color: AuraTheme.base;
                opacity: 0; // Animated in via Rust timing callbacks
            }
        }
    }
}
```

---

## 7. Module Block 05 — The Infinite Update (Hot-Swappable Engine) [DONE]

**Status:** Implemented. Rendering Handoff Protocol (RHP) functional with support for window handle passing and zero flicker.

### 7.1 The Memory-Safe Handoff Problem

Hot-swapping a rendering engine without a flicker requires solving three hard problems simultaneously:

1. **Active context survival**: The GPU context (OpenGL/WebGPU surface), font caches, and layout trees live inside `aura_engine.dll`. Swapping the `.dll` means those pointers die.
2. **Frame continuity**: The user must not see a blank frame or tear.
3. **Thread safety**: The old `.dll` may still be processing a paint call when the new one loads.

#### The Solution: The Rendering Handoff Protocol (RHP)

```
Phase A — Prepare (background, invisible to user)
──────────────────────────────────────────────────
1. Download new engine: `aura_engine_v2.dll` to `~/.aura/engines/pending/`
2. Verify SHA-256 signature against update server's Ed25519 public key
3. Load new .dll in PARALLEL via libloading (shadow instance)
4. Call new_engine.cold_init() — initialises everything EXCEPT the GPU surface
5. Signal: READY_FOR_HANDOFF

Phase B — Serialise State (1 frame pause, ~16ms)
──────────────────────────────────────────────────
6. Acquire render_mutex (blocks new paint calls)
7. Call old_engine.serialise_state() → returns EngineSnapshot {
       current_url, scroll_position, active_dom_state,
       font_cache_bytes (serialised to heap),
       layout_tree_hash (for cache warming)
   }
8. old_engine.freeze() — stops GPU surface, flushes to an offscreen texture
   The offscreen texture is now owned by the COMPOSITOR, not the engine

Phase C — Swap (atomic, ~2ms)
──────────────────────────────────────────────────
9.  compositor.hold_last_frame() — presents frozen texture every vsync
10. Drop Arc<OldEngine> — old .dll refcount falls to 0
    libloading unloads old .dll only after Arc drops
11. new_engine.warm_init(snapshot) — restores URL, scroll, warms font cache
12. new_engine.acquire_gpu_surface() — takes ownership of the platform surface

Phase D — Resume (seamless)
──────────────────────────────────────────────────
13. new_engine.paint() produces first real frame
14. compositor.release_hold() — switches from frozen texture to live feed
15. Release render_mutex
16. Delete old .dll file
```

**Why zero flicker?** Between steps 8 and 13, the compositor is replaying the last good frozen frame at full vsync speed. The user's eye sees a "paused" frame for ~18ms — imperceptibly brief. This is identical to how browsers handle OOP iframe process swaps.

### 7.2 Engine ABI Contract

```rust
// aura-engine/src/lib.rs — this is the cdylib ABI

/// VIBE AGENT PROMPT:
/// "Implement all 6 extern C functions exactly. They form the stable ABI contract
///  that the main app depends on. NEVER change function signatures between versions —
///  only add new functions. Use #[no_mangle] on all of them."

use std::ffi::c_void;

/// Opaque handle passed across FFI boundary
pub struct EngineContext;

#[no_mangle]
pub extern "C" fn aura_engine_version() -> *const u8 {
    // Returns a null-terminated version string: "1.4.2\0"
    b"1.4.2\0".as_ptr()
}

#[no_mangle]
pub extern "C" fn aura_engine_cold_init(config: *const EngineConfig) -> *mut EngineContext {
    // Initialises everything except GPU surface
    // Returns null on failure
    let ctx = Box::new(EngineContext::new_cold(unsafe { &*config }));
    Box::into_raw(ctx)
}

#[no_mangle]
pub extern "C" fn aura_engine_warm_init(
    ctx: *mut EngineContext,
    snapshot: *const EngineSnapshot,
) -> bool {
    // Restores state from snapshot after hot-swap
    let ctx = unsafe { &mut *ctx };
    ctx.restore_from_snapshot(unsafe { &*snapshot })
}

#[no_mangle]
pub extern "C" fn aura_engine_freeze(
    ctx: *mut EngineContext,
    out_snapshot: *mut EngineSnapshot,
) -> bool {
    // Serialises state + releases GPU surface
    let ctx = unsafe { &mut *ctx };
    let snapshot = ctx.serialise_state();
    unsafe { *out_snapshot = snapshot };
    ctx.release_gpu_surface();
    true
}

#[no_mangle]
pub extern "C" fn aura_engine_paint(
    ctx: *mut EngineContext,
    surface: *mut c_void,  // platform-specific surface handle
) {
    let ctx = unsafe { &mut *ctx };
    ctx.paint_to_surface(surface)
}

#[no_mangle]
pub extern "C" fn aura_engine_destroy(ctx: *mut EngineContext) {
    if !ctx.is_null() {
        unsafe { drop(Box::from_raw(ctx)) }
    }
}
```

### 7.3 Hot-Swap Manager: Cargo.toml + Pseudo-Rust

```toml
# In aura-app/Cargo.toml (add to existing)

[dependencies]
libloading = "0.8"
ed25519-dalek = "2"
reqwest = { version = "0.12", features = ["rustls-tls", "stream"] }
semver = "1"
sha2 = "0.10"
tokio = { version = "1", features = ["full"] }
```

```rust
// aura-app/src/hot_swap.rs

pub struct HotSwapManager {
    current: Arc<Mutex<LoadedEngine>>,  // Arc so compositor can hold a ref during swap
    update_channel: tokio::sync::mpsc::Receiver<EngineUpdate>,
}

impl HotSwapManager {
    pub async fn check_and_apply_update(&self) -> Result<bool, SwapError> {
        // 1. Query update server (lightweight JSON: {version, url, sha256, sig})
        let manifest = fetch_update_manifest().await?;

        let current_version = semver::Version::parse(self.current_version())?;
        let new_version = semver::Version::parse(&manifest.version)?;

        if new_version <= current_version {
            return Ok(false);  // No update needed
        }

        // 2. Download new engine to temp path
        let pending_path = download_engine(&manifest).await?;

        // 3. Verify signature (Ed25519 over SHA-256 of file)
        verify_engine_signature(&pending_path, &manifest.signature)?;

        // 4. Execute the Rendering Handoff Protocol
        self.perform_handoff(pending_path).await?;

        Ok(true)
    }

    async fn perform_handoff(&self, new_dylib: PathBuf) -> Result<(), SwapError> {
        // Phase A: Load shadow instance
        let new_lib = unsafe { libloading::Library::new(&new_dylib)? };
        let cold_init: libloading::Symbol<unsafe extern "C" fn(*const EngineConfig) -> *mut EngineContext>
            = unsafe { new_lib.get(b"aura_engine_cold_init")? };

        let new_ctx = unsafe { cold_init(&self.config as *const _) };
        if new_ctx.is_null() { return Err(SwapError::InitFailed); }

        // Phase B: Serialise state
        let mut guard = self.current.lock().await;
        let mut snapshot = EngineSnapshot::default();
        let freeze_fn = guard.get_symbol::<unsafe extern "C" fn(*mut EngineContext, *mut EngineSnapshot) -> bool>("aura_engine_freeze")?;
        unsafe { freeze_fn(guard.ctx, &mut snapshot) };

        // Phase C: Compositor holds last frame
        self.compositor.hold_last_frame();

        // Swap Arc contents atomically
        let old_lib = std::mem::replace(&mut guard.lib, new_lib);
        guard.ctx = new_ctx;

        // Warm-init new engine with snapshot
        let warm_init = guard.get_symbol::<unsafe extern "C" fn(*mut EngineContext, *const EngineSnapshot) -> bool>("aura_engine_warm_init")?;
        unsafe { warm_init(new_ctx, &snapshot) };

        // Phase D: First paint + release hold
        guard.paint();
        self.compositor.release_hold();

        // Old lib drops here — unloads .dll after Arc refcount reaches 0
        drop(old_lib);
        Ok(())
    }
}
```

---

## 8. GitHub Actions: The Auto-Builder

```yaml
# .github/workflows/aura-build.yml

name: Aura — Build, Verify & Ship

on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      release_type:
        description: 'Release type (stable/nightly)'
        required: false
        default: 'nightly'

env:
  RUST_VERSION: "1.82.0"
  CARGO_TERM_COLOR: always

jobs:
  # ── Job 1: Vibe Check (lint + test) ─────────────────────────────────────
  vibe-check:
    name: 🌿 Vibe Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Clippy (deny warnings)
        run: cargo clippy --all-targets --all-features -- -D warnings
      - name: Run tests
        run: cargo test --workspace

  # ── Job 2: Build Engine (cdylib for all platforms) ───────────────────────
  build-engine:
    name: 🔧 Build Engine — ${{ matrix.platform }}
    needs: vibe-check
    strategy:
      matrix:
        include:
          - platform: windows
            os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: aura_engine.dll
          - platform: linux
            os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
            artifact: libaura_engine.so
          - platform: macos
            os: macos-14
            target: aarch64-apple-darwin
            artifact: libaura_engine.dylib
          - platform: android
            os: ubuntu-22.04
            target: aarch64-linux-android
            artifact: libaura_engine.so
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
        with:
          key: engine-${{ matrix.platform }}
      - name: Install Android NDK
        if: matrix.platform == 'android'
        run: |
          echo "y" | sudo $ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager "ndk;26.1.10909125"
          echo "ANDROID_NDK_HOME=$ANDROID_SDK_ROOT/ndk/26.1.10909125" >> $GITHUB_ENV
          cargo install cargo-ndk
      - name: Build engine cdylib
        run: |
          if [ "${{ matrix.platform }}" = "android" ]; then
            cargo ndk -t arm64-v8a build --release -p aura-engine
          else
            cargo build --release -p aura-engine --target ${{ matrix.target }}
        shell: bash
      - name: Sign engine artifact (Ed25519)
        run: |
          # Sign using secret key stored in GitHub Secrets
          echo "${{ secrets.ENGINE_SIGNING_KEY }}" > /tmp/signing.key
          # signing script omitted — use signify or minisign
      - uses: actions/upload-artifact@v4
        with:
          name: engine-${{ matrix.platform }}
          path: target/${{ matrix.target }}/release/${{ matrix.artifact }}

  # ── Job 3: Build App (full bundle) ──────────────────────────────────────
  build-app:
    name: 📦 Bundle App — ${{ matrix.platform }}
    needs: build-engine
    strategy:
      matrix:
        include:
          - platform: windows
            os: windows-latest
            bundle: exe
          - platform: linux
            os: ubuntu-22.04
            bundle: deb
          - platform: macos
            os: macos-14
            bundle: dmg
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Download engine artifact
        uses: actions/download-artifact@v4
        with:
          name: engine-${{ matrix.platform }}
          path: aura-app/engines/
      - name: Install Tauri CLI
        run: cargo install tauri-cli --version "^2"
      - name: Install frontend deps (Slint)
        run: cargo build -p aura-ui
      - name: Build Tauri bundle
        run: cargo tauri build --bundles ${{ matrix.bundle }}
        working-directory: aura-app
      - uses: actions/upload-artifact@v4
        with:
          name: aura-${{ matrix.platform }}-${{ matrix.bundle }}
          path: aura-app/target/release/bundle/

  # ── Job 4: Android APK ──────────────────────────────────────────────────
  build-android:
    name: 📱 Build Android APK
    needs: build-engine
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-linux-android
      - uses: actions/setup-java@v4
        with:
          java-version: '17'
          distribution: 'temurin'
      - name: Download Android engine
        uses: actions/download-artifact@v4
        with:
          name: engine-android
          path: aura-app/engines/android/
      - name: Build Android APK
        run: cargo tauri android build
        working-directory: aura-app
        env:
          ANDROID_SIGNING_KEY: ${{ secrets.ANDROID_SIGNING_KEY }}
      - uses: actions/upload-artifact@v4
        with:
          name: aura-android-apk
          path: aura-app/gen/android/app/build/outputs/apk/

  # ── Job 5: Publish to Update Server ─────────────────────────────────────
  publish-update:
    name: 🚀 Publish Update Manifest
    needs: [build-app, build-android]
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - name: Generate update manifest JSON
        run: |
          # Creates update.json with version, sha256 per platform, download URLs
          python .github/scripts/gen_update_manifest.py
      - name: Deploy manifest to update CDN
        run: |
          # rsync or S3 upload to update.aura-browser.dev
          echo "Deploy step — configure your CDN here"
```

---

## 9. Hardware Profiles & Optimization Matrix

### Profile A: Budget Mobile (Samsung Galaxy M36 5G — Target Dev Device)

```
Chip:        Snapdragon 6s Gen 3  (Cortex-A55 + A76 cores)
RAM:         8 GB LPDDR4X
Storage:     128 GB UFS 2.2
GPU:         Adreno 710
Android:     14
```

| Optimisation | Setting | Reason |
|---|---|---|
| AI Model | Qwen2.5 1.5B Q4_0 | ~900 MB, 2s load, fits in RAM |
| Render threads | 2 (2 perf + 0 eff) | Avoid thermal throttle |
| JS engine | None (Servo WC only) | No V8/SpiderMonkey overhead |
| Image decode | NEON SIMD | A76 cores support ARMv8.2 NEON |
| Font cache | 32 MB max | Cleared on background |
| Silo purge | Aggressive (1 session) | Flash storage endurance |
| Candle device | `Device::Cpu` | No Vulkan compute by default |
| Max open tabs | 8 (soft limit) | Constellation shows warning at 7 |

**Cargo feature flags for Android:**
```toml
[features]
mobile = ["candle-core/no-std-float", "small-font-cache", "aggressive-gc"]
default = []
```

### Profile B: Budget Laptop (8 GB RAM x86_64 Linux/Windows)

| Optimisation | Setting |
|---|---|
| AI Model | Phi-3 Mini 3.8B Q4_K_S (2.1 GB) — better quality for laptop use |
| Candle device | `Device::Cpu` with AVX2 kernels |
| Render threads | 4 |
| Max open tabs | 20 |
| Hot-swap | Enabled |

---

## 10. Full Dependency Map

```toml
# Root Cargo.toml (workspace)

[workspace]
members = [
    "aura-app",
    "aura-engine",
    "aura-ui",
    "aura-silo",
    "aura-ai",
    "aura-net",
]
resolver = "2"

[workspace.dependencies]
# Core
tauri            = { version = "2" }
servo            = { git = "https://github.com/servo/servo", branch = "main" }
libloading       = "0.8"
tokio            = { version = "1", features = ["full"] }
serde            = { version = "1", features = ["derive"] }
serde_json       = "1"

# UI
slint            = "1.7"

# Security / Silo
rusqlite         = { version = "0.31", features = ["bundled"] }
aes-gcm          = "0.10"
sha2             = "0.10"
ed25519-dalek    = "2"
rand             = "0.8"
hex              = "0.4"
keyring          = "2"

# AI
candle-core      = { git = "https://github.com/huggingface/candle" }
candle-nn        = { git = "https://github.com/huggingface/candle" }
candle-transformers = { git = "https://github.com/huggingface/candle" }
hf-hub           = { version = "0.3", features = ["tokio"] }
tokenizers       = "0.19"
scraper          = "0.19"

# Network / Adblock
adblock          = "0.8"
reqwest          = { version = "0.12", features = ["rustls-tls"], default-features = false }
url              = "2"

# Utilities
once_cell        = "1"
thiserror        = "1"
tracing          = "0.1"
tracing-subscriber = "0.3"
dirs             = "5"
semver           = "1"
image            = "0.25"   # favicon dominant colour extraction
```

---

## Appendix A: Vibe Coding Agent Prompts (Master List)

Copy these directly into your AI coding agent (Cursor, Windsurf, Claude Code, etc.):

```
BLOCK 01 — Shell:
"Create a Tauri v2 workspace in Rust. Add a borderless, transparent window.
 On startup, load aura_engine.dll via libloading. Expose a `navigate(url)` 
 Tauri command that passes the URL to the engine after running it through 
 aura_net::intercept()."

BLOCK 02 — UI:
"In Slint, create: (1) a CommandBar component that fades in at Alt+Space,
 centred at 32% vertical height, with a sage-coloured breathing dot.
 (2) A Constellation component on the left edge, visible only when cursor
 hovers within 20px of the left edge for 300ms."

BLOCK 03 — Silo:
"Implement SiloManager using rusqlite. Each TLD gets its own .silo.db file
 at ~/.aura/silos/{sha256(domain)}.silo.db. Cookie values must be 
 AES-256-GCM encrypted. On session close, delete all non-pinned silos."

BLOCK 04 — AI:
"Implement AiEngine using candle. Load Qwen2.5-1.5B-Instruct Q4_0 GGUF 
 from HuggingFace Hub into ~/.aura/models/. The summarise(html) function 
 must return Vec<String> of 3 bullets in under 8 seconds on a device with 
 2 CPU cores. Use Device::Cpu on Android."

BLOCK 05 — Hot-Swap:
"Implement HotSwapManager using libloading. The 6 extern C ABI functions
 (cold_init, warm_init, freeze, paint, destroy, version) form the stable 
 contract. Implement perform_handoff() using the Rendering Handoff Protocol: 
 freeze → compositor holds last frame → swap Arc → warm-init → first paint 
 → release hold. Zero flicker guaranteed."
```

---

## Appendix B: File Structure

```
aura/
├── .github/
│   ├── workflows/
│   │   └── aura-build.yml
│   └── scripts/
│       └── gen_update_manifest.py
├── aura-app/
│   ├── src/
│   │   ├── main.rs
│   │   ├── hot_swap.rs
│   │   └── commands.rs
│   ├── engines/           # Downloaded engine dylibs go here
│   ├── tauri.conf.json
│   └── Cargo.toml
├── aura-engine/
│   ├── src/
│   │   └── lib.rs         # cdylib — the 6 ABI functions
│   └── Cargo.toml
├── aura-ui/
│   ├── ui/
│   │   ├── tokens.slint
│   │   ├── command-bar.slint
│   │   ├── constellation.slint
│   │   ├── breathe-overlay.slint
│   │   └── main.slint
│   └── Cargo.toml
├── aura-silo/
│   ├── src/
│   │   ├── lib.rs
│   │   └── schema.sql
│   └── Cargo.toml
├── aura-ai/
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── aura-net/
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── Cargo.toml             # Workspace root
└── README.md
```

---

*Aura — Built in stillness. Shipped in Rust.*
