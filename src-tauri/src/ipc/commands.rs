use tauri::State;
use std::sync::Arc;
use tracing::{info, error};
use crate::AppState;
use crate::search::{SearchEngine, SearchResult};
use crate::timeline::{Timeline, TimelineManager};
use crate::knowledge::GraphData;

#[tauri::command]
pub async fn search_memories(
    state: State<'_, Arc<AppState>>,
    query: String,
    limit: Option<usize>,
    activity_type: Option<String>,
    app_name: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    state.metrics.increment_searches();
    
    let limit = limit.unwrap_or(20);
    
    let results = if activity_type.is_some() || app_name.is_some() {
        state.search_engine.search_with_filters(
            &query,
            activity_type.as_deref(),
            app_name.as_deref(),
            None,
            None,
            limit,
        )
    } else {
        state.search_engine.search(&query, limit)
    }.map_err(|e| e.to_string())?;
    
    Ok(results)
}

#[tauri::command]
pub async fn get_timeline(
    state: State<'_, Arc<AppState>>,
    days: Option<usize>,
) -> Result<Timeline, String> {
    let days = days.unwrap_or(7);
    state.timeline_manager.get_timeline(days).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_memory_graph(
    state: State<'_, Arc<AppState>>,
) -> Result<GraphData, String> {
    state.knowledge_graph.get_graph_data().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_pause(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let mut paused = state.is_paused.write();
    *paused = !*paused;
    info!("Tracking paused: {}", *paused);
    Ok(*paused)
}

#[tauri::command]
pub async fn get_status(
    state: State<'_, Arc<AppState>>,
) -> Result<StatusInfo, String> {
    let activity_count = state.db.get_activity_count().map_err(|e| e.to_string())?;
    let screenshot_count = state.db.get_screenshot_count().map_err(|e| e.to_string())?;
    let pending = state.db.get_pending_embeddings(1000).map_err(|e| e.to_string())?;
    
    let settings = state.settings.read();
    
    Ok(StatusInfo {
        is_paused: *state.is_paused.read(),
        activity_count,
        screenshot_count,
        pending_embeddings: pending.len(),
        activity_capture_enabled: settings.activity_capture.enabled,
        screen_capture_enabled: settings.screen_capture.enabled,
        clipboard_monitoring_enabled: settings.activity_capture.clipboard_monitoring,
    })
}

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let settings = state.settings.read();
    Ok(serde_json::to_value(&*settings).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, Arc<AppState>>,
    new_settings: serde_json::Value,
) -> Result<(), String> {
    let settings: crate::config::Settings = serde_json::from_value(new_settings)
        .map_err(|e| e.to_string())?;
    
    settings.save().map_err(|e| e.to_string())?;
    
    *state.settings.write() = settings;
    
    info!("Settings updated");
    Ok(())
}

#[tauri::command]
pub async fn delete_memory(
    state: State<'_, Arc<AppState>>,
    activity_id: Option<String>,
) -> Result<(), String> {
    if let Some(id) = activity_id {
        state.db.delete_activity(&id).map_err(|e| e.to_string())?;
        info!("Deleted activity: {}", id);
    } else {
        state.db.delete_all_data().map_err(|e| e.to_string())?;
        info!("Deleted all memory data");
    }
    Ok(())
}

#[tauri::command]
pub async fn get_debug_metrics(
    state: State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    Ok(state.metrics.get_all())
}

#[tauri::command]
pub async fn preview_activity(
    state: State<'_, Arc<AppState>>,
    activity_id: String,
) -> Result<Option<ActivityPreview>, String> {
    let activities = state.db.get_activities(1000, 0).map_err(|e| e.to_string())?;
    
    if let Some(activity) = activities.iter().find(|a| a.id == activity_id) {
        let related = state.knowledge_graph.get_related_nodes(&activity_id).ok();
        
        Ok(Some(ActivityPreview {
            id: activity.id.clone(),
            activity_type: activity.activity_type.clone(),
            title: activity.title.clone()
                .or_else(|| activity.content.clone())
                .unwrap_or_default(),
            app_name: activity.app_name.clone(),
            file_path: activity.file_path.clone(),
            url: activity.url.clone(),
            content: activity.content.clone(),
            timestamp: activity.timestamp,
            session_id: activity.session_id.clone(),
            related_nodes: related.unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusInfo {
    pub is_paused: bool,
    pub activity_count: i64,
    pub screenshot_count: i64,
    pub pending_embeddings: usize,
    pub activity_capture_enabled: bool,
    pub screen_capture_enabled: bool,
    pub clipboard_monitoring_enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActivityPreview {
    pub id: String,
    pub activity_type: String,
    pub title: String,
    pub app_name: Option<String>,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub content: Option<String>,
    pub timestamp: i64,
    pub session_id: Option<String>,
    pub related_nodes: Vec<crate::knowledge::RelatedNode>,
}
