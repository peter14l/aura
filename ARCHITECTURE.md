# Aura Architecture Overview

Aura is built as a multi-process, hot-swappable architecture using Tauri v2 as the shell and Servo as the rendering engine.

## Process Map

```text
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

## Crate Topology

The project is structured as a Cargo workspace:

*   **`aura-app`**: The Tauri v2 shell (binary). Manages the window, IPC, and loads the engine.
*   **`aura-ui`**: Slint components for the Stillness UI.
*   **`aura-engine`**: A `cdylib` that wraps Servo. Loaded dynamically via `libloading`.
*   **`aura-silo`**: Manages isolated, per-domain SQLite databases for cookies and local storage.
*   **`aura-ai`**: Local AI inference using `candle` and HuggingFace models.
*   **`aura-net`**: Network interception and adblocking.
