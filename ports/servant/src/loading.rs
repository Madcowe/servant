/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref PROGRESS: Mutex<HashMap<String, LoadingProgress>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Debug)]
pub struct LoadingProgress {
    pub status: String,
    pub bytes_loaded: usize,
    pub total_bytes: Option<usize>,
    pub error: Option<String>,
    pub finished: bool,
}

pub struct LoadingTracker {
    address: String,
    start: Instant,
}

impl LoadingTracker {
    pub fn start(address: &str) -> Self {
        println!("🚀 Starting network fetch from Autonomi for {}...", address);
        let mut progress = PROGRESS.lock().unwrap();
        progress.insert(address.to_string(), LoadingProgress {
            status: "Initializing...".to_string(),
            bytes_loaded: 0,
            total_bytes: None,
            error: None,
            finished: false,
        });
        Self { address: address.to_string(), start: Instant::now() }
    }

    pub fn update_status(&self, status: &str) {
        let mut progress = PROGRESS.lock().unwrap();
        if let Some(p) = progress.get_mut(&self.address) {
            p.status = status.to_string();
        }
    }

    pub fn update_progress(&self, bytes: usize, total: Option<usize>) {
        let mut progress = PROGRESS.lock().unwrap();
        if let Some(p) = progress.get_mut(&self.address) {
            p.bytes_loaded = bytes;
            if total.is_some() {
                p.total_bytes = total;
            }
        }
    }

    pub fn finish(&self, bytes: usize) {
        println!("✅ Fetch complete! {} bytes loaded in {:?}", bytes, self.start.elapsed());
        let mut progress = PROGRESS.lock().unwrap();
        if let Some(p) = progress.get_mut(&self.address) {
            p.status = "Complete".to_string();
            p.bytes_loaded = bytes;
            p.total_bytes = Some(bytes);
            p.finished = true;
        }
    }

    pub fn error(&self, err: &str) {
        println!("❌ Fetch failed after {:?}: {}", self.start.elapsed(), err);
        let mut progress = PROGRESS.lock().unwrap();
        if let Some(p) = progress.get_mut(&self.address) {
            p.status = "Error".to_string();
            p.error = Some(err.to_string());
            p.finished = true;
        }
    }

    pub fn get_progress(address: &str) -> Option<LoadingProgress> {
        let progress = PROGRESS.lock().unwrap();
        progress.get(address).cloned()
    }
}
