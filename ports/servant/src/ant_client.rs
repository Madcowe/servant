use std::sync::Arc;
use ant_core::data::client::{Client, ClientConfig};

pub struct AntClientManager {
    client: Arc<Client>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl AntClientManager {
    pub fn connect() -> Result<Self, String> {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .thread_name("ant-worker")
                .build().map_err(|e| e.to_string())?
        );

        let client = runtime.block_on(async {
            // Note: In Phase 3 we will load peers from config or devnet manifest.
            // For now, passing an empty list means ant-client will try built-in mainnet defaults.
            let peers = vec![];
            Client::connect(&peers, ClientConfig::default()).await
        }).map_err(|e| e.to_string())?;

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
