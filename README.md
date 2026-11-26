# ODIT Rust Plugin SDK (`odit-sdk`)

Write sandboxed WebAssembly (`.wasm`) plugins for **Onyx Editor (`odit`)** in idiomatic Rust — **zero external dependencies**, fast offline builds, and full Host ABI v1.0.0 coverage.

## Features

- **`#[odit_plugin(...)]` proc-macro** (`odit-sdk-macros`, zero deps): generates the JSON manifest and automatically exports all 9 required ODIT WASM ABI symbols (`_initialize`, `odit_abi_version`, `odit_alloc`, `odit_free`, `odit_plugin_manifest`, `odit_plugin_init`, `odit_plugin_shutdown`, `odit_plugin_event`, `odit_plugin_render`).
- **Typed `Event<'a>` enum:** `ActiveFileChanged`, `BufferModified`, `CursorMoved`, `DiagnosticsUpdated`, `SymbolsUpdated`, `Command`, `DebugStateChanged`, `KeyInput`, `WorkspaceChanged`.
- **Safe Immediate-Mode UI & Layout:** `ui::VStack`, `ui::card`, `ui::scroll_area`, `ui::clipped`, `ui::table`, `ui::on_click`, `ui::input_text`, `ui::dropdown`, `ui::Texture`, `ui::Sound` (`Sound::load_memory(include_bytes!(...), ".mp3")`).
- **Declarative Macros:** `log_info!`, `log_warn!`, `log_error!`, `notify!`, `text!`.
- **Full Host API Coverage:** `host` (commands, clipboard, `Ctrl+,` settings), `editor`, `workspace`, `lsp`, `process`, and `debug`.

## Quick Start

1. Ensure the `wasm32-unknown-unknown` target is installed:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. Add `odit-sdk` to your `cdylib` plugin crate's `Cargo.toml`:
   ```toml
   [lib]
   crate-type = ["cdylib"]

   [dependencies]
   odit-sdk = { git = "https://github.com/pigeonowo/odit_rust_sdk" }
   ```

3. Build & install the included example plugin:
   ```bash
   cargo build -p hello_plugin --target wasm32-unknown-unknown --release
   mkdir -p ~/.config/odit/plugins
   cp target/wasm32-unknown-unknown/release/hello_plugin.wasm ~/.config/odit/plugins/
   ```

4. Inside `odit`, press `Ctrl+P` → **`Plugins: Reload All .wasm Plugins`**, then `Ctrl+P` → **`Plugin: Open hello_plugin.wasm`**.

## Running Unit Tests

```bash
cargo test
```
