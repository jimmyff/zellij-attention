# Project Instructions

## Project Overview

Zellij WASM plugin that adds notification icons to tab names when panes need attention. Designed for Claude Code users running multiple sessions across tabs.

**Flow:** External process → `zellij pipe --name "zellij-attention::EVENT::PANE_ID"` → plugin updates tab name → focus clears notification.

## Architecture

- `src/lib.rs` — Core plugin logic: event handling, tab renaming, pipe parsing, focus clearing
- `src/main.rs` — Plugin registration only (`register_plugin!`)
- `src/state.rs` — `NotificationType` enum (attention/working/done) + display `priority()`
- `src/config.rs` — User configuration parsing (enabled, attention_icon, working_icon, done_icon)
- Build target: `wasm32-wasip1` (Zellij WASM plugin); built on demand via the project `flake.nix` devshell

## Key Design Decisions

- **Single global plugin instance** via `load_plugins` in `config.kdl` (no visible pane)
- **Notification state:** `HashMap<u32, NotificationType>` — pane_id → one state. Latest event **replaces** (no stacking per pane)
- **Three states by priority:** `attention 🚨` > `working ⏳` > `done ✅` (default icons, all config-overridable). A tab with several Claude panes shows the highest `priority()`.
- **Live status vs. completion flag:** attention/working mirror what Claude is doing now and are **never** focus-cleared; `done` is a completion flag that **focus clears** (incl. when it arrives while the tab is already focused)
- **Pipe verbs:** `attention` / `working` / `done` set state; `clear` removes the pane's marker
- **No position-keyed caches** — original tab names derived via `strip_icons()` at rename time, making tab reordering safe
- **`rename_tab()` is 1-indexed** — Zellij API quirk, always pass `position + 1`
- **`updating_tabs` flag** prevents re-entrancy from `rename_tab()` → `TabUpdate` → `update_tab_names()` loop

## Zellij Plugin Gotchas

- Use broadcast pipes (`zellij pipe --name`) NOT targeted pipes (`--plugin`) — targeted pipes create new instances due to config mismatch
- Plugin state must use `/host/` path (shared), NOT `/data/` (sandboxed per-instance)
- `load_plugins` in config.kdl supports configuration via plugin aliases or inline config blocks
- Plugin pane IDs overlap with terminal pane IDs — always filter `is_plugin` when mapping panes
- `rename_tab()` triggers a synchronous `TabUpdate` event — beware of race conditions between rename and the resulting event
- `load_plugins` plugins may be lost after session resurrection (zellij attach) — see [#4156](https://github.com/zellij-org/zellij/issues/4156)
- After rebuilding WASM, clear Zellij's compiled-plugin cache (path is OS-specific) and start a fresh session:
  - Linux: `find ~/.cache/zellij -path "*zellij-attention*" -delete`
  - macOS: `find ~/Library/Caches/org.Zellij-Contributors.Zellij -path "*zellij-attention*" -delete`

## Build & Test

```bash
# Build
cargo build --release --target wasm32-wasip1

# Install
cp target/wasm32-wasip1/release/zellij-attention.wasm ~/.config/zellij/plugins/

# Test manually
zellij pipe --name "zellij-attention::attention::$ZELLIJ_PANE_ID"
zellij pipe --name "zellij-attention::working::$ZELLIJ_PANE_ID"
zellij pipe --name "zellij-attention::done::$ZELLIJ_PANE_ID"
zellij pipe --name "zellij-attention::clear::$ZELLIJ_PANE_ID"
```
