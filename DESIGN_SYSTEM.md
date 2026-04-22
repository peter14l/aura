# Aura Design System

## Vocabulary

*   **Subtractive Glassmorphism**: Remove until only essential information remains. No chrome. No decoration. Blur replaces borders.
*   **Spatial Breathing**: Elements expand/contract via breathing easing (`cubic-bezier(0.34, 1.56, 0.64, 1)`).
*   **Ink Fade**: All animations use opacity + transform. Nothing slides. Everything materialises.

## Color Palette

| Token | Hex | Usage |
|---|---|---|
| `--aura-base-light` | `#FDFCF5` | Background (light mode) — Soft Alabaster |
| `--aura-base-dark` | `#121412` | Background (dark mode) — Deep Obsidian |
| `--aura-sage` | `#D4E1D1` | Muted Sage — primary accent, hover glows |
| `--aura-rose` | `#E9D5CA` | Dusty Rose — AI / wellness highlights |
| `--aura-glass` | `rgba(253,252,245,0.08)` | Glass panel fill |
| `--aura-blur` | `blur(28px) saturate(160%)` | Backdrop filter |
| `--aura-border` | `rgba(255,255,255,0.0)` | **Zero borders** — the subtractive principle |

## Slint Design Tokens (`tokens.slint`)

```slint
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

## Gestural Fluidity Map

| GESTURE | TRIGGER | ACTION |
| :--- | :--- | :--- |
| Alt + Space | Keyboard | Summon Command Bar |
| Left-edge hover 300ms | Cursor dock | Reveal Session Constellation |
| Left-edge hover + scroll | Cursor dock + wheel | Cycle active tab (no click) |
| Cursor to top edge | Proximity < 20px | Reveal minimal address ghost |
| Cursor to bottom edge | Proximity < 20px | Reveal status + Lotus AI icon |
| 3-finger swipe left/right| Trackpad gesture | History back / forward |
| 3-finger swipe up | Trackpad gesture | New tab (spawns node in const.) |
| 3-finger swipe down | Trackpad gesture | Close active tab |
| Right-edge hover 300ms | Cursor dock | Reveal Silo status panel |
| Pinch-to-zoom | Trackpad / touch | Page zoom (no UI shown) |
| Double-tap space | Keyboard (reader mode) | Toggle Zen Reading Mode |
| Lotus icon click | Bottom gutter | Trigger AI Breathe Summary |
| Drag node inward (const.)| Tab constellation drag | Pin tab (node becomes solid) |
| Drag node outward (const)| Tab constellation drag | Close tab (node fades to void) |
