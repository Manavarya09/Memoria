use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use parking_lot::RwLock;
use tracing::{info, error, debug};
use crate::storage::{Database, Embedding, PendingEmbedding, Activity};
use crate::config::Settings;
use crate::utils::Metrics;
use crate::processing::{clean_text, truncate_for_embedding};
use crate::AppState;

pub struct EmbeddingManager {
    db: Arc<Database>,
    settings: Arc<RwLock<Settings>>,
    metrics: Arc<Metrics>,
}

impl EmbeddingManager {
    pub fn new(db: Arc<Database>, settings: Arc<RwLock<Settings>>, metrics: Arc<Metrics>) -> Self {
        Self {
            db,
            settings,
            metrics,
        }
    }
    
    pub fn generate_embedding(&self, text: &str) -> Vec<f32> {
        let start = std::time::Instant::now();
        
        let hash = simple_hash(text);
        let dimensions = 384;
        let mut embedding = vec![0.0f32; dimensions];
        
        for (i, val) in embedding.iter_mut().enumerate() {
            let combined = hash.wrapping_add((i as u64).wrapping_mul(31));
            *val = ((combined as f64).sin() * 10000.0).rem_euclid(2.0) as f32 - 1.0;
        }
        
        let magnitude = (embedding.iter().map(|x| x * x).sum::<f32>()).sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }
        
        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.record_embedding_latency(elapsed);
        
        embedding
    }
    
    pub fn compute_similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f32 {
        if embedding1.len() != embedding2.len() {
            return 0.0;
        }
        
        embedding1.iter()
            .zip(embedding2.iter())
            .map(|(a, b)| a * b)
            .sum()
    }
}

fn simple_hash(text: &str) -> u64 {
    let mut hash: u64 = 0;
    for (i, byte) in text.bytes().enumerate() {
        hash = hash.wrapping_add(((byte as u64).wrapping_mul(31)).wrapping_pow(i as u32));
        hash = hash.rotate_left(5);
    }
    hash
}

pub async fn start_indexing_worker(state: Arc<AppState>) {
    info!("Starting Embedding Indexing Worker");
    
    let db = Arc::clone(&state.db);
    let settings = state.settings.clone();
    let metrics = Arc::clone(&state.metrics);
    
    loop {
        sleep(Duration::from_millis(500)).await;
        
        let (batch_size, enabled) = {
            let guard = settings.read();
            (guard.performance.embedding_batch_size, guard.activity_capture.enabled)
        };
        
        if !enabled {
            sleep(Duration::from_secs(5)).await;
            continue;
        }
        
        if let Ok(pending) = db.get_pending_embeddings(batch_size) {
            metrics.set_queue_size(pending.len());
            
            if pending.is_empty() {
                continue;
            }
            
            debug!("Processing {} pending embeddings", pending.len());
            
            for p in &pending {
                let embedding_manager = EmbeddingManager::new(
                    Arc::clone(&db),
                    Arc::clone(&settings),
                    Arc::clone(&metrics),
                );
                
                let embedding_vec = embedding_manager.generate_embedding(&p.content);
                
                let embedding = Embedding {
                    id: uuid::Uuid::new_v4().to_string(),
                    activity_id: p.activity_id.clone(),
                    embedding: embedding_vec.iter()
                        .flat_map(|f| f.to_le_bytes())
                        .collect(),
                    model_name: "memoria-simple".to_string(),
                    dimensions: 384,
                    created_at: chrono::Utc::now().timestamp(),
                };
                
                if let Err(e) = db.insert_embedding(&embedding) {
                    error!("Failed to insert embedding: {}", e);
                    continue;
                }
                
                if let Err(e) = db.delete_pending_embedding(&p.id) {
                    error!("Failed to delete pending embedding: {}", e);
                }
                
                metrics.increment_embeddings();
            }
        }
    }
}

impl crate::AppState {
    pub fn queue_activity_for_embedding(self: &Arc<Self>, activity: &Activity) {
        let db = Arc::clone(&self.db);
        
        let content = activity.content.clone()
            .or_else(|| activity.title.clone())
            .or_else(|| activity.file_path.clone())
            .or_else(|| activity.url.clone());
        
        if let Some(text) = content {
            let cleaned = clean_text(&text);
            if cleaned.len() > 10 {
                let truncated = truncate_for_embedding(&cleaned, 5000);
                let pending = PendingEmbedding::new(&activity.id, &truncated);
                if let Err(e) = db.insert_pending_embedding(&pending) {
                    error!("Failed to queue embedding: {}", e);
                }
            }
        }
    }
}
