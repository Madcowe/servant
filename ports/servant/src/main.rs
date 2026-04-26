use servo::ServoBuilder;
use net::protocols::ProtocolRegistry;
use servant::ant_client::AntClientManager;
use servant::content_resolver::ContentResolver;
use servant::ant_protocol::AntProtocolHandler;
use servant::cache::ContentCache;
use std::sync::Arc;

fn main() {
    println!("Servant initializing...");

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
    let mut protocols = ProtocolRegistry::default();
    protocols.register("ant", Box::new(AntProtocolHandler::new(resolver)))
        .expect("Failed to register ant:// protocol handler");

    // Build servo with our protocol
    let servo = ServoBuilder::default()
        .protocol_registry(protocols)
        // .event_loop_waker(waker) // We'll add this when integrating servoshell's UI loop
        .build();

    println!("Servant browser engine successfully built with Autonomi integration!");
}
