# The Infinite Update (Hot-Swappable Engine)

Aura can update its core rendering engine (`aura_engine.dll` / `libaura_engine.so`) without a restart or visible flicker using the **Rendering Handoff Protocol (RHP)**.

## The Rendering Handoff Protocol

1. **Phase A — Prepare**: Download the new engine, verify the signature, and load it dynamically alongside the current engine. Run `cold_init`.
2. **Phase B — Serialise State**: Acquire a render mutex, freeze the old engine, and capture a snapshot (URL, scroll position, DOM state). The compositor holds the last rendered frame.
3. **Phase C — Swap**: Drop the reference to the old engine (unloading it) and pass the snapshot to the new engine via `warm_init`.
4. **Phase D — Resume**: The new engine performs its first paint, the compositor releases the held frame, and the update is complete.

## Engine ABI Contract

The engine exposes a stable C ABI:

```c
const char* aura_engine_version();
EngineContext* aura_engine_cold_init(const EngineConfig* config);
bool aura_engine_warm_init(EngineContext* ctx, const EngineSnapshot* snapshot);
bool aura_engine_freeze(EngineContext* ctx, EngineSnapshot* out_snapshot);
void aura_engine_paint(EngineContext* ctx, void* surface);
void aura_engine_destroy(EngineContext* ctx);
```
