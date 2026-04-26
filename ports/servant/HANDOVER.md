# Servant Project Handover Summary

This document provides a comprehensive summary of the **Servant** project aims, current architectural structure, and critical known workarounds. It is designed to act as context for future development or AI agent hand-offs.

## Project Aims
**Servant** is a web browser built on top of the [Servo](https://servo.org/) engine, natively integrated with the [Autonomi Network](https://autonomi.com/). The primary goal is to support the `ant://` protocol, allowing users to browse decentralized, content-addressed websites and data stored on the Autonomi network just like standard HTTP web pages.

To ensure long-term maintainability, Servant is structured as a module within the Servo workspace. It leverages Servo's existing `servoshell` UI to avoid rewriting window management and rendering code, acting as a thin wrapper that injects Autonomi-specific networking logic.

## Architecture and Structure So Far

### Workspace Integration
*   **Location:** `ports/servant` inside the main `servo` repository workspace.
*   **Dependencies:** Servant directly relies on the upstream `WithAutonomi/ant-client` (`ant-core`) repository via relative path (`../../../ant-client/ant-core`), ensuring it always uses the latest SDK changes.

### Core Modules (`ports/servant/src/`)
1.  **`main.rs` (UI Injection):** Acts as the entry point. It instantiates the Autonomi client and then delegates execution to `servoshell::main_with_protocols`. We modified `servoshell`'s CLI/App architecture to accept a custom protocol registry closure, allowing us to natively register `ant://` without permanently polluting the Servo core codebase.
2.  **`ant_protocol.rs`:** Implements the Servo `ProtocolHandler` trait. It intercepts all `ant://` requests initiated by the browser, parsing the URL and routing it to the resolution engine. It also intercepts internal pages like `ant://settings` to serve local UI.
3.  **`content_resolver.rs`:** The core business logic. It handles the two-step Autonomi fetch process: (1) Fetching the `DataMap` for a given address, and (2) Downloading the actual file chunks. It performs automatic MIME-type detection using file magic bytes (PNG, JPEG, PDF, HTML) since Autonomi data is currently opaque.
4.  **`cache.rs`:** A thread-safe LRU Content Cache. It stores recently fetched Autonomi data in memory to dramatically improve page load times and avoid redundant network calls during a session.
5.  **`ant_client.rs`:** Manages the connection lifecycle and instantiation of the `ant-core` client.
6.  **`loading.rs` & `ant_url.rs`:** Provides latency tracking/logging and basic hex-address URL validation.

## Critical Workarounds & Fixes

### 1. `brotli-decompressor` Duplicate Symbol Linker Conflict
During integration, building the `servant` binary failed with `rust-lld: error: duplicate symbol: BrotliDecoderDecompress` and similar FFI symbol clashes. 

**Cause:**
*   `ant-core` -> `self_encryption` depends on `brotli` v3.3, which pulls `brotli-decompressor v2.3.5`.
*   `servo` -> `servo-net` -> `async-compression` depends on `brotli` v8.0, which pulls `brotli-decompressor v5.0.0`.
*   Both crate versions unconditionally exported the same `#[no_mangle]` C FFI functions. When the linker attempted to build the final binary, it encountered identical symbols, failing the build.

**The Fix:**
Because Cargo cannot namespace C-exports across different versions of the same crate, the fix was applied directly to the local Cargo registry cache. The `ffi` module was removed from the older v2.3.5, stopping it from exporting the symbols, while allowing v5.0.0 to fulfill the linkage for Servo's C++ font engine (`fontsan_woff2`).

**How to re-apply if the cache is cleared/updated:**
If you encounter this error again on a fresh clone or after a `cargo update`, run the following in your terminal:
```bash
sed -i 's/pub mod ffi;/\/\/pub mod ffi;/g' ~/.cargo/registry/src/*/brotli-decompressor-2.3.5/src/lib.rs
cargo clean -p brotli-decompressor
cargo build -p servant
```

## Running the Project
Because `./mach run` hardcodes the execution of `servoshell`, you must run the Servant wrapper binary directly via Cargo to ensure the `ant://` protocol is active:

```bash
cargo run -p servant
```
Alternatively, build with Mach and execute manually:
```bash
./mach build --dev -p servant
./target/debug/servant
```

## Future / Next Steps
1.  **Directory Support:** The upstream `ant-sdk` is currently stabilizing its directory manifest formats (`SiteManifest`). Once finalized, `content_resolver.rs` must be updated to traverse directory structures rather than just loading raw opaque blobs.
2.  **UI Feedback:** Integrate Servo's waker to push real-time loading progress to the UI instead of relying purely on terminal logging.
