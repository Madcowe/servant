use std::sync::Arc;
use bytes::Bytes;
use ant_core::data::client::Client;
use crate::cache::ContentCache;
use crate::loading::LoadingTracker;
use hex;

#[derive(Clone)]
pub enum ResolvedContent {
    /// A single file retrieved via DataMap.
    SingleFile { data: Bytes, mime: String },

    /// A single raw chunk.
    RawChunk { data: Bytes, mime: String },
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

    pub fn clear_cache(&self) {
        println!("Clearing content cache...");
        self.cache.clear();
    }

    pub async fn resolve(&self, address: &[u8; 32], sub_path: Option<&str>) -> Result<ResolvedContent, ResolveError> {
        if let Some(path) = sub_path {
            if !path.is_empty() {
                return Err(ResolveError::NotFound("Directory support coming soon".to_string()));
            }
        }

        if let Some(cached) = self.cache.get(address) {
            return Ok((*cached).clone());
        }

        println!("Resolving ant://{} ...", hex::encode(address));
        let tracker = LoadingTracker::start();

        // Try fetching as a DataMap first with retries
        let mut last_err = None;
        let mut data_map = None;
        for attempt in 1..=3 {
            match self.client.data_map_fetch(address).await {
                Ok(dm) => {
                    data_map = Some(dm);
                    break;
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    println!("⚠️ Attempt {} to fetch DataMap failed: {}", attempt, err_msg);
                    last_err = Some(err_msg);
                    tokio::time::sleep(std::time::Duration::from_millis(500 * attempt)).await;
                }
            }
        }

        if let Some(dm) = data_map {
            println!("Found DataMap, downloading chunks...");
            let mut data = None;
            for attempt in 1..=3 {
                match self.client.data_download(&dm).await {
                    Ok(d) => {
                        data = Some(d);
                        break;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        println!("⚠️ Attempt {} to download chunks failed: {}", attempt, err_msg);
                        last_err = Some(err_msg);
                        tokio::time::sleep(std::time::Duration::from_millis(1000 * attempt)).await;
                    }
                }
            }

            if let Some(data) = data {
                println!("✅ Download complete ({} bytes).", data.len());
                tracker.finish(data.len());
                let mime = self.sniff_mime(&data, sub_path);
                let resolved = ResolvedContent::SingleFile { data, mime };
                self.cache.insert(*address, Arc::new(resolved.clone()));
                return Ok(resolved);
            }
        } else {
            // Fallback to raw chunk if it's not a DataMap
            println!("Address not a DataMap, trying raw chunk fetch...");
            match self.client.chunk_get(address).await {
                Ok(Some(chunk)) => {
                    println!("✅ Raw chunk retrieved ({} bytes).", chunk.content.len());
                    tracker.finish(chunk.content.len());
                    let mime = self.sniff_mime(&chunk.content, sub_path);
                    let resolved = ResolvedContent::RawChunk { data: chunk.content, mime };
                    self.cache.insert(*address, Arc::new(resolved.clone()));
                    return Ok(resolved);
                }
                Ok(None) => {
                    last_err = Some("Chunk not found on network".to_string());
                }
                Err(e) => {
                    last_err = Some(e.to_string());
                }
            }
        }

        let final_err = last_err.unwrap_or_else(|| "Unknown resolution error".to_string());
        tracker.error(&final_err);
        Err(ResolveError::NetworkError(final_err))
    }

    fn sniff_mime(&self, data: &[u8], sub_path: Option<&str>) -> String {
        if let Some(path) = sub_path {
            if let Some(mime) = mime_guess::from_path(path).first_raw() {
                return mime.to_string();
            }
        }

        // Standard signatures
        if data.len() >= 4 && &data[0..4] == b"\x89PNG" {
            return "image/png".to_string();
        }
        if data.len() >= 4 && &data[0..4] == b"%PDF" {
            return "application/pdf".to_string();
        }
        if data.len() >= 2 && &data[0..2] == b"\xff\xd8" {
            return "image/jpeg".to_string();
        }
        if data.len() >= 3 && &data[0..3] == b"ID3" {
            return "audio/mpeg".to_string();
        }
        if data.len() >= 3 && &data[0..2] == b"\xff\xfb" {
            return "audio/mpeg".to_string();
        }

        // Generic text/binary fallback
        if data.iter().take(512).any(|&b| b == 0) {
            "application/octet-stream".to_string()
        } else {
            let sample_vec: Vec<u8> = data.iter().take(512).cloned().collect();
            let s = String::from_utf8_lossy(&sample_vec);
            if s.contains("<html") || s.contains("<!DOCTYPE html") {
                "text/html".to_string()
            } else {
                "text/plain".to_string()
            }
        }
    }
}
