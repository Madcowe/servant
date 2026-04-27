use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::content_resolver::ResolvedContent;

/// Simple LRU-like cache for resolved content.
pub struct ContentCache {
    capacity: usize,
    cache: Mutex<CacheState>,
}

struct CacheState {
    map: HashMap<[u8; 32], Arc<ResolvedContent>>,
    order: VecDeque<[u8; 32]>,
}

impl ContentCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: Mutex::new(CacheState {
                map: HashMap::new(),
                order: VecDeque::new(),
            }),
        }
    }

    pub fn get(&self, address: &[u8; 32]) -> Option<Arc<ResolvedContent>> {
        let mut state = self.cache.lock().unwrap();
        if let Some(content) = state.map.get(address) {
            let content = Arc::clone(content);
            // Move to back (most recently used)
            if let Some(pos) = state.order.iter().position(|a| a == address) {
                state.order.remove(pos);
                state.order.push_back(*address);
            }
            Some(content)
        } else {
            None
        }
    }

    pub fn insert(&self, address: [u8; 32], content: Arc<ResolvedContent>) {
        let mut state = self.cache.lock().unwrap();
        if state.map.contains_key(&address) {
            state.map.insert(address, content);
            if let Some(pos) = state.order.iter().position(|a| a == &address) {
                state.order.remove(pos);
                state.order.push_back(address);
            }
            return;
        }

        if state.map.len() >= self.capacity {
            if let Some(oldest) = state.order.pop_front() {
                state.map.remove(&oldest);
            }
        }

        state.map.insert(address, content);
        state.order.push_back(address);
    }

    pub fn clear(&self) {
        let mut state = self.cache.lock().unwrap();
        state.map.clear();
        state.order.clear();
    }
}
