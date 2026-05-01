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

fn main() {
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

        // Register ant:// protocol
        protocols.register("ant", AntProtocolHandler::new(resolver.clone(), ant_manager))
            .expect("Failed to register ant:// protocol handler");

        // Provide a way for the UI to save files from the cache
        let resolver_clone = resolver.clone();
        servoshell::set_resource_data_provider(Box::new(move |url| {
            resolver_clone.get_cached_bytes_for_url(url).map(|b| b.to_vec())
        }));
            
        println!("ant:// protocol successfully registered.");
    });
}
