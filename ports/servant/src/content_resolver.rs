/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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

#[derive(Clone)]
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
        self.client.chunk_cache().clear();
    }

    pub fn is_cached(&self, address: &[u8; 32]) -> bool {
        self.cache.get(address).is_some()
    }

    pub fn get_cached_bytes_for_url(&self, url: &url::Url) -> Option<bytes::Bytes> {
        let ant_url = crate::ant_url::AntUrl::parse(url).ok()?;
        self.cache.get_bytes(&ant_url.address)
    }

    pub async fn resolve(&self, address: &[u8; 32], sub_path: Option<&str>) -> Result<ResolvedContent, ResolveError> {
        // sub_path is now used for MIME sniffing and will be used for directories later.

        if let Some(cached) = self.cache.get(address) {
            return Ok((*cached).clone());
        }

        let addr_hex = hex::encode(address);
        println!("Resolving ant://{} ...", addr_hex);
        let tracker = LoadingTracker::start(&addr_hex);

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
            tracker.update_status("Downloading file chunks...");
            println!("Found DataMap, downloading chunks...");
            
            // We use a manual download loop to track progress
            let mut data = None;
            for attempt in 1..=3 {
                match self.download_with_progress(address, &dm, &tracker).await {
                    Ok(d) => {
                        data = Some(d);
                        break;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        println!("⚠️ Attempt {} to download chunks failed: {}", attempt, err_msg);
                        tracker.update_status(&format!("Download attempt {} failed, retrying...", attempt));
                        last_err = Some(err_msg);
                        tokio::time::sleep(std::time::Duration::from_millis(1000 * attempt)).await;
                    }
                }
            }

            if let Some(data) = data {
                println!("✅ Download complete ({} bytes).", data.len());
                let mime = self.sniff_mime(&data, sub_path);
                tracker.finish_with_mime(data.len(), Some(mime.clone()));
                let resolved = ResolvedContent::SingleFile { data, mime };
                self.cache.insert(*address, Arc::new(resolved.clone()));
                return Ok(resolved);
            }
        } else {
            tracker.update_status("Address not a DataMap, trying raw chunk fetch...");
            // Fallback to raw chunk if it's not a DataMap
            println!("Address not a DataMap, trying raw chunk fetch...");
            match self.client.chunk_get(address).await {
                Ok(Some(chunk)) => {
                    println!("✅ Raw chunk retrieved ({} bytes).", chunk.content.len());
                    let mime = self.sniff_mime(&chunk.content, sub_path);
                    tracker.finish_with_mime(chunk.content.len(), Some(mime.clone()));
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

    async fn download_with_progress(&self, address: &[u8; 32], data_map: &ant_core::data::DataMap, tracker: &LoadingTracker) -> Result<Bytes, ant_core::data::error::Error> {
        use tokio::sync::mpsc;
        use ant_core::data::DownloadEvent;
        
        let (tx, mut rx) = mpsc::channel(64);
        let mut path = std::env::temp_dir();
        path.push(format!("download_{}.tmp", hex::encode(address)));
        
        let client_clone = self.client.clone();
        let dm_clone = data_map.clone();
        let path_clone = path.clone();
        
        let download_handle = tokio::spawn(async move {
            client_clone.file_download_with_progress(&dm_clone, &path_clone, Some(tx)).await
        });
        
        while let Some(event) = rx.recv().await {
            match event {
                DownloadEvent::ResolvingDataMap { total_map_chunks } => {
                    tracker.update_status(&format!("Resolving DataMap ({} chunks)...", total_map_chunks));
                }
                DownloadEvent::MapChunkFetched { fetched } => {
                    tracker.update_status(&format!("Fetched DataMap chunk {}...", fetched));
                }
                DownloadEvent::DataMapResolved { total_chunks } => {
                    tracker.update_status("DataMap resolved.");
                    tracker.update_progress(0, Some(total_chunks));
                }
                DownloadEvent::ChunksFetched { fetched, total } => {
                    tracker.update_progress(fetched, Some(total));
                }
            }
        }
        
        let _ = download_handle.await.map_err(|e| ant_core::data::error::Error::Encryption(e.to_string()))??;
        
        let data = tokio::fs::read(&path).await.map_err(|e| ant_core::data::error::Error::InvalidData(e.to_string()))?;
        let _ = tokio::fs::remove_file(&path).await;
        
        Ok(Bytes::from(data))
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
        } else if data.is_empty() {
            "text/plain".to_string()
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
