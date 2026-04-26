
use servant::ant_client::AntClientManager;
use servant::content_resolver::ContentResolver;
use servant::ant_protocol::AntProtocolHandler;
use servant::cache::ContentCache;
use std::sync::Arc;

fn main() {
    println!("Servant initializing with UI...");

    servoshell::main_with_protocols(|protocols| {
        // Initialize Autonomi client
        let ant_manager = match AntClientManager::connect() {
            Ok(manager) => manager,
            Err(e) => {
                eprintln!("Failed to connect to Autonomi network: {}", e);
                std::process::exit(1);
            }
        };

        let cache = Arc::new(ContentCache::new(100)); // LRU cache up to 100 entries
        let resolver = ContentResolver::new(ant_manager.client(), cache);

        // Register ant:// protocol
        protocols.register("ant", AntProtocolHandler::new(resolver))
            .expect("Failed to register ant:// protocol handler");
            
        println!("ant:// protocol successfully registered.");
    });
}
