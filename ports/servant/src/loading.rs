use std::time::Instant;

pub struct LoadingTracker {
    start: Instant,
}

impl LoadingTracker {
    pub fn start() -> Self {
        println!("🚀 Starting network fetch from Autonomi...");
        Self { start: Instant::now() }
    }

    pub fn finish(&self, bytes: usize) {
        println!("✅ Fetch complete! {} bytes loaded in {:?}", bytes, self.start.elapsed());
    }

    pub fn error(&self, err: &str) {
        println!("❌ Fetch failed after {:?}: {}", self.start.elapsed(), err);
    }
}
