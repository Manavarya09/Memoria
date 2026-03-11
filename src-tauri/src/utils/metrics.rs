use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

pub struct Metrics {
    pub events_captured: AtomicU64,
    pub screenshots_captured: AtomicU64,
    pub ocr_processed: AtomicU64,
    pub embeddings_generated: AtomicU64,
    pub search_queries: AtomicU64,
    pub indexing_queue_size: AtomicUsize,
    pub last_event_time: AtomicU64,
    pub app_start_time: Instant,
    pub vector_search_latency_ms: AtomicU64,
    pub embedding_generation_ms: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            events_captured: AtomicU64::new(0),
            screenshots_captured: AtomicU64::new(0),
            ocr_processed: AtomicU64::new(0),
            embeddings_generated: AtomicU64::new(0),
            search_queries: AtomicU64::new(0),
            indexing_queue_size: AtomicUsize::new(0),
            last_event_time: AtomicU64::new(0),
            app_start_time: Instant::now(),
            vector_search_latency_ms: AtomicU64::new(0),
            embedding_generation_ms: AtomicU64::new(0),
        }
    }

    pub fn increment_events(&self) {
        self.events_captured.fetch_add(1, Ordering::Relaxed);
        self.last_event_time.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            Ordering::Relaxed,
        );
    }

    pub fn increment_screenshots(&self) {
        self.screenshots_captured.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_ocr(&self) {
        self.ocr_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_embeddings(&self) {
        self.embeddings_generated.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_searches(&self) {
        self.search_queries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_queue_size(&self, size: usize) {
        self.indexing_queue_size.store(size, Ordering::Relaxed);
    }

    pub fn record_vector_search_latency(&self, ms: u64) {
        self.vector_search_latency_ms.store(ms, Ordering::Relaxed);
    }

    pub fn record_embedding_latency(&self, ms: u64) {
        self.embedding_generation_ms.store(ms, Ordering::Relaxed);
    }

    pub fn get_all(&self) -> serde_json::Value {
        serde_json::json!({
            "events_captured": self.events_captured.load(Ordering::Relaxed),
            "screenshots_captured": self.screenshots_captured.load(Ordering::Relaxed),
            "ocr_processed": self.ocr_processed.load(Ordering::Relaxed),
            "embeddings_generated": self.embeddings_generated.load(Ordering::Relaxed),
            "search_queries": self.search_queries.load(Ordering::Relaxed),
            "indexing_queue_size": self.indexing_queue_size.load(Ordering::Relaxed),
            "last_event_time": self.last_event_time.load(Ordering::Relaxed),
            "uptime_seconds": self.app_start_time.elapsed().as_secs(),
            "vector_search_latency_ms": self.vector_search_latency_ms.load(Ordering::Relaxed),
            "embedding_generation_ms": self.embedding_generation_ms.load(Ordering::Relaxed),
        })
    }
}
