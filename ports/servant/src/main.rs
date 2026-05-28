/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servant::ant_client::AntClientManager;
use servant::content_resolver::ContentResolver;
use servant::ant_protocol::AntProtocolHandler;
use servant::cache::ContentCache;
use std::sync::Arc;
use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Bootstrap peers as comma-separated socket addresses (e.g. 1.2.3.4:10000)
    #[arg(long, value_delimiter = ',')]
    bootstrap_peers: Option<Vec<SocketAddr>>,

    /// Path to a devnet manifest JSON file
    #[arg(long)]
    devnet_manifest: Option<String>,

    /// URL or address to load
    url: Option<String>,
}

#[cfg(not(any(target_os = "android", target_env = "ohos")))]
fn main() {
    init_dlls();
    let args = Args::parse();
    println!("Servant initializing with UI...");

    servoshell::main_with_protocols(move |protocols| {
        // Initialize Autonomi client with CLI overrides
        let ant_manager = match AntClientManager::connect(args.bootstrap_peers, args.devnet_manifest) {
            Ok(manager) => Arc::new(manager),
            Err(e) => {
                eprintln!("Failed to connect to Autonomi network: {}", e);
                std::process::exit(1);
            }
        };

        let cache = Arc::new(ContentCache::new(100)); // LRU cache up to 100 entries
        let resolver = ContentResolver::new(ant_manager.client(), cache);

        // Register ant:// and autonomi:// protocols
        let handler = AntProtocolHandler::new(resolver.clone(), ant_manager);
        protocols.register("ant", handler.clone())
            .expect("Failed to register ant:// protocol handler");
        protocols.register("autonomi", handler)
            .expect("Failed to register autonomi:// protocol handler");

        // Provide a way for the UI to save files from the cache
        let resolver_clone = resolver.clone();
        servoshell::set_resource_data_provider(Box::new(move |url| {
            resolver_clone.get_cached_content_for_url(url).map(|(b, m)| (b.to_vec(), m))
        }));
            
        println!("ant:// and autonomi:// protocols successfully registered.");
    });
}

#[cfg(target_os = "windows")]
fn init_dlls() {
    use std::fs;
    use std::path::PathBuf;
    use std::os::windows::ffi::OsStrExt;

    let egl_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/libEGL.dll"));
    let gles_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/libGLESv2.dll"));

    let temp_dir = std::env::temp_dir().join("servant-egl-libs");
    fs::create_dir_all(&temp_dir).ok();

    let egl_path = temp_dir.join("libEGL.dll");
    let gles_path = temp_dir.join("libGLESv2.dll");

    let write_if_needed = |path: &PathBuf, bytes: &[u8]| {
        if !path.exists() || fs::metadata(path).map(|m| m.len() != bytes.len() as u64).unwrap_or(true) {
            fs::write(path, bytes).ok();
        }
    };

    write_if_needed(&egl_path, egl_bytes);
    write_if_needed(&gles_path, gles_bytes);

    let mut path_u16: Vec<u16> = temp_dir.as_os_str().encode_wide().collect();
    path_u16.push(0);

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
    }

    unsafe {
        SetDllDirectoryW(path_u16.as_ptr());
    }
}

#[cfg(not(target_os = "windows"))]
fn init_dlls() {}

#[cfg(any(target_os = "android", target_env = "ohos"))]
fn main() {}

