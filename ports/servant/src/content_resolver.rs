use std::sync::Arc;
use bytes::Bytes;
use ant_core::data::client::Client;
use crate::cache::ContentCache;
use crate::loading::LoadingTracker;

#[derive(Clone)]
pub enum ResolvedContent {
    /// A single file retrieved via DataMap.
    SingleFile { data: Bytes, mime: String },

    // Future variants:
    // Directory { manifest: DirectoryManifest },
    // RawChunk { data: Bytes, mime: String },
}

#[derive(Debug)]
pub enum ResolveError {
    NetworkError(String),
    NotFound(String),
    DecodeError(String),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not Found: {}", msg),
            Self::DecodeError(msg) => write!(f, "Decode Error: {}", msg),
        }
    }
}

impl std::error::Error for ResolveError {}

pub struct ContentResolver {
    client: Arc<Client>,
    cache: Arc<ContentCache>,
}

impl ContentResolver {
    pub fn new(client: Arc<Client>, cache: Arc<ContentCache>) -> Self {
        Self { client, cache }
    }

    pub async fn resolve(&self, address: &[u8; 32], sub_path: Option<&str>) -> Result<ResolvedContent, ResolveError> {
        if let Some(path) = sub_path {
            if !path.is_empty() {
                // For now, no sub-path (directory) support
                return Err(ResolveError::NotFound("Directory support coming soon".to_string()));
            }
        }

        if let Some(cached) = self.cache.get(address) {
            return Ok((*cached).clone());
        }

        let tracker = LoadingTracker::start();

        let data_map = self.client.data_map_fetch(address).await
            .map_err(|e| {
                tracker.error(&e.to_string());
                ResolveError::NetworkError(e.to_string())
            })?;

        let data = self.client.data_download(&data_map).await
            .map_err(|e| {
                tracker.error(&e.to_string());
                ResolveError::NetworkError(e.to_string())
            })?;

        tracker.finish(data.len());

        let mime = if data.len() >= 4 && &data[0..4] == b"\x89PNG" {
            "image/png".to_string()
        } else if data.len() >= 4 && &data[0..4] == b"%PDF" {
            "application/pdf".to_string()
        } else if data.len() >= 2 && &data[0..2] == b"\xff\xd8" {
            "image/jpeg".to_string()
        } else if data.iter().take(512).any(|&b| b == 0) {
            "application/octet-stream".to_string()
        } else {
            let sample_vec: Vec<u8> = data.iter().take(512).cloned().collect();
            let s = String::from_utf8_lossy(&sample_vec);
            if s.contains("<html") || s.contains("<!DOCTYPE html") {
                "text/html".to_string()
            } else {
                "text/plain".to_string()
            }
        };

        let resolved = ResolvedContent::SingleFile { data, mime };
        self.cache.insert(*address, Arc::new(resolved.clone()));

        Ok(resolved)
    }
}
