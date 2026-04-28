use std::sync::Arc;
use std::net::SocketAddr;
use ant_core::data::client::{Client, ClientConfig};
use ant_core::data::{DevnetManifest, MultiAddr, IPDiversityConfig};
use ant_core::config as ant_config;
use serde_json;

// Default bootstrap peers for Autonomi network if no config is found.
// These match the peers used in the ant_get_file utility.
const DEFAULT_BOOTSTRAP_PEERS: &[&str] = &[
    "207.148.94.42:10000",
    "45.77.50.10:10000",
    "66.135.23.83:10000",
    "149.248.9.2:10000",
    "49.12.119.240:10000",
    "5.161.25.133:10000",
    "18.228.202.183:10000",
];

pub struct AntClientManager {
    client: Arc<Client>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl AntClientManager {
    pub fn connect(
        bootstrap_override: Option<Vec<SocketAddr>>,
        devnet_manifest: Option<String>,
    ) -> Result<Self, String> {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4) // Increased for stability
                .enable_all()
                .thread_name("ant-worker")
                .build().map_err(|e| e.to_string())?
        );

        // Resolve bootstrap peers
        let peers = if let Some(overridden) = bootstrap_override {
            println!("Using CLI-provided bootstrap peers: {} nodes", overridden.len());
            overridden
        } else if let Some(manifest_path) = devnet_manifest {
            println!("Loading bootstrap peers from devnet manifest: {}", manifest_path);
            let data = std::fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
            let manifest: DevnetManifest = serde_json::from_str(&data).map_err(|e| e.to_string())?;
            manifest.bootstrap
                .iter()
                .filter_map(MultiAddr::socket_addr)
                .collect()
        } else {
            match ant_config::load_bootstrap_peers() {
                Ok(Some(config_peers)) => {
                    println!("Loaded {} bootstrap peer(s) from config file", config_peers.len());
                    config_peers
                }
                _ => {
                    println!("No bootstrap config found. Using {} hardcoded default peers.", DEFAULT_BOOTSTRAP_PEERS.len());
                    DEFAULT_BOOTSTRAP_PEERS
                        .iter()
                        .filter_map(|s| s.parse().ok())
                        .collect()
                }
            }
        };

        if peers.is_empty() {
            return Err("No bootstrap peers available. Cannot connect to Autonomi network.".to_string());
        }

        // Configure client to match ant_get_file's working configuration
        let mut config = ClientConfig::default();
        config.ipv6 = false; // Match ant_get_file explicitly disabling ipv6
        config.quote_timeout_secs = 60; // Increase timeouts for reliability
        config.store_timeout_secs = 60;
        config.quote_concurrency = 4; // Lower concurrency to fix "missing chunk" errors
        config.store_concurrency = 4;

        println!("Starting Autonomi client with {} bootstrap peers...", peers.len());
        for (i, peer) in peers.iter().enumerate().take(5) {
            println!("  [{}] {}", i, peer);
        }
        if peers.len() > 5 {
            println!("  ... and {} more", peers.len() - 5);
        }

        let client = runtime.block_on(async {
            println!("Establishing P2P connection (timeout: 60s)...");
            Client::connect(&peers, config).await
        }).map_err(|e| {
            let err_msg = format!("Autonomi connection failed: {}", e);
            println!("{}", err_msg);
            err_msg
        })?;

        println!("✅ Successfully connected to Autonomi network.");

        Ok(Self {
            client: Arc::new(client),
            runtime,
        })
    }

    pub fn client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
    
    pub fn runtime(&self) -> Arc<tokio::runtime::Runtime> {
        Arc::clone(&self.runtime)
    }
}
