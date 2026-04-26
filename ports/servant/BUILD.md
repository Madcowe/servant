# Building Servant

Servant is integrated directly into the `servo` workspace as a member crate. Because of this, it can be built and run using Servo's standard build system (`mach`) or standard Cargo commands (if your environment is set up for it).

## Prerequisites
Ensure your development environment has the necessary tools to build Servo. This generally includes:
- **Rust Toolchain:** (usually managed via `rustup`). Servo automatically handles its own specific nightly toolchain requirements via `mach`.
- **Python & `uv`:** Servo's `mach` script uses `uv` for managing python dependencies.
- **System Dependencies:** Standard dependencies required by Servo (like `cmake`, `pkg-config`, `libx11-dev`, `libegl1-mesa-dev`, etc., depending on your OS).

*Note: If you have successfully built the `servo` or `servoshell` crate in this workspace before, your environment is already set up.*

## Building and Running

### Option 1: Using `mach` (Recommended)
Servo's `mach` script is the most reliable way to build components as it handles environment variables and toolchains correctly.

To build the `servant` binary:
```bash
./mach build --dev -p servant
```

To run the `servant` binary:
```bash
./mach run -p servant
```

*Note: `mach` commands must be executed from the root of the `servo` repository.*

### Option 2: Using Cargo
If you prefer standard cargo commands and your environment handles the native dependencies gracefully:

```bash
cargo build -p servant
cargo run -p servant
```

## Running with Autonomi configuration

Servant connects to the Autonomi network upon startup. By default, it will attempt to connect to the mainnet. 

**Using a Devnet:**
If you want to test against a local or custom devnet, you can pass the devnet manifest directly to servant:
```bash
./mach run -p servant -- --devnet-manifest /path/to/devnet.json
```

**Using Custom Bootstrap Peers:**
```bash
./mach run -p servant -- --bootstrap-peers addr1,addr2
```

## Cross-Compilation
Because Servant uses standard Servo dependencies and `ant-core`, it inherits their cross-compilation support. Target OS includes Linux, Android, Windows, and macOS.
You can use `mach` to cross-compile for Android:
```bash
./mach build --android -p servant
```
(Refer to standard Servo documentation for setting up the Android NDK and other cross-compilation targets).

## Troubleshooting

- **`uv: not found`**: If `./mach` fails complaining about `uv`, you need to install the `uv` python package manager. You can install it via `pip install uv` or `curl -LsSf https://astral.sh/uv/install.sh | sh`.
- **Compilation errors in `ant-core`**: The `ant-client` project is actively developed. If the crate fails to compile due to changes in `ant-core`, please ensure your `ant-client` is checked out to a compatible commit, or that `content_resolver.rs` and `ant_client.rs` are updated to match the latest `ant-core` API.
