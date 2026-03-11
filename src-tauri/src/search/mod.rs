use crate::config::Settings;
use crate::embedding::EmbeddingManager;
use crate::storage::{Activity, Database};
use crate::AppState;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info};

pub struct SearchEngine {
    db: Arc<Database>,
    embedding_manager: Arc<EmbeddingManager>,
}

impl SearchEngine {
    pub fn new(db: Arc<Database>, embedding_manager: Arc<EmbeddingManager>) -> Self {
        Self {
            db,
            embedding_manager,
        }
    }

    pub fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();

        let query_embedding = self.embedding_manager.generate_embedding(query);

        let activities = self.db.get_activities(1000, 0)?;

        let mut results: Vec<SearchResult> = Vec::new();

        for activity in activities {
            let content = activity
                .content
                .clone()
                .or_else(|| activity.title.clone())
                .or_else(|| activity.file_path.clone())
                .or_else(|| activity.url.clone())
                .unwrap_or_default();

            if content.is_empty() {
                continue;
            }

            let activity_embedding = self.embedding_manager.generate_embedding(&content);

            let similarity = self
                .embedding_manager
                .compute_similarity(&query_embedding, &activity_embedding);

            let recency_boost = self.compute_recency_boost(activity.timestamp);

            let final_score = (similarity * 0.7) + (recency_boost * 0.3);

            results.push(SearchResult {
                activity,
                score: final_score,
                similarity,
                recency_boost,
            });
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let results = results.into_iter().take(limit).collect();

        let elapsed = start.elapsed().as_millis() as u64;
        debug!("Search completed in {}ms", elapsed);

        Ok(results)
    }

    fn compute_recency_boost(&self, timestamp: i64) -> f32 {
        let now = chrono::Utc::now().timestamp();
        let hours_ago: f32 = (now - timestamp) as f32 / 3600.0;

        let half_life: f32 = 7.0 * 24.0;
        let boost = (-hours_ago.ln() / half_life.ln()).exp();

        boost.min(1.0)
    }

    pub fn search_with_filters(
        &self,
        query: &str,
        activity_type: Option<&str>,
        app_name: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = self.search(query, limit * 2)?;

        results.retain(|r| {
            if let Some(at) = activity_type {
                if &r.activity.activity_type != at {
                    return false;
                }
            }

            if let Some(app) = app_name {
                if r.activity.app_name.as_deref() != Some(app) {
                    return false;
                }
            }

            if let Some(start) = start_time {
                if r.activity.timestamp < start {
                    return false;
                }
            }

            if let Some(end) = end_time {
                if r.activity.timestamp > end {
                    return false;
                }
            }

            true
        });

        results.truncate(limit);

        Ok(results)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    #[serde(flatten)]
    pub activity: Activity,
    pub score: f32,
    pub similarity: f32,
    pub recency_boost: f32,
}
