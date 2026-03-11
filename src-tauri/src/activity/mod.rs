use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, debug};
use crate::storage::{Database, Activity};
use crate::config::Settings;
use crate::utils::Metrics;
use parking_lot::RwLock;
use chrono::Utc;

pub struct ActivityManager {
    db: Arc<Database>,
    settings: Arc<RwLock<Settings>>,
    metrics: Arc<Metrics>,
    current_session_id: Arc<RwLock<Option<String>>>,
    last_activity_time: Arc<RwLock<i64>>,
}

impl ActivityManager {
    pub fn new(db: Arc<Database>, settings: Arc<RwLock<Settings>>, metrics: Arc<Metrics>) -> Self {
        Self {
            db,
            settings,
            metrics,
            current_session_id: Arc::new(RwLock::new(None)),
            last_activity_time: Arc::new(RwLock::new(Utc::now().timestamp())),
        }
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Activity Capture Engine");
        
        let db = Arc::clone(&self.db);
        let settings = Arc::clone(&self.settings);
        let metrics = Arc::clone(&self.metrics);
        let current_session = Arc::clone(&self.current_session_id);
        let last_time = Arc::clone(&self.last_activity_time);
        
        tokio::spawn(async move {
            let mut last_app = String::new();
            let idle_threshold_minutes = 15i64;
            
            loop {
                sleep(Duration::from_secs(1)).await;
                
                let enabled = {
                    let guard = settings.read();
                    guard.activity_capture.enabled
                };
                
                if !enabled {
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
                
                let excluded_apps = {
                    let guard = settings.read();
                    guard.privacy.exclude_apps.clone()
                };
                
                if let Some(app_info) = get_active_app().await {
                    if excluded_apps.iter().any(|e| app_info.name.to_lowercase().contains(&e.to_lowercase())) {
                        continue;
                    }
                    
                    let now = Utc::now().timestamp();
                    let last_ts = *last_time.read();
                    
                    if now - last_ts > idle_threshold_minutes * 60 {
                        let session_id = create_new_session(&db, &app_info.name).await;
                        *current_session.write() = session_id;
                    }
                    
                    if app_info.name != last_app {
                        let mut activity = Activity::new("app_focus");
                        activity.app_name = Some(app_info.name.clone());
                        activity.app_bundle_id = Some(app_info.bundle_id);
                        activity.title = app_info.window_title;
                        activity.session_id = current_session.read().clone();
                        
                        if let Err(e) = db.insert_activity(&activity) {
                            error!("Failed to insert activity: {}", e);
                        } else {
                            metrics.increment_events();
                            debug!("Recorded app focus: {}", app_info.name);
                        }
                        
                        last_app = app_info.name.clone();
                    }
                    
                    *last_time.write() = now;
                }
            }
        });
        
        let db2 = Arc::clone(&self.db);
        let settings2 = Arc::clone(&self.settings);
        
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(10)).await;
                
                let enabled = {
                    let guard = settings2.read();
                    guard.activity_capture.clipboard_monitoring
                };
                
                if !enabled {
                    continue;
                }
                
                if let Ok(clipboard) = get_clipboard_content() {
                    if !clipboard.is_empty() && clipboard.len() < 10000 {
                        let activity = Activity::new("clipboard");
                        let mut activity = activity;
                        activity.content = Some(clipboard);
                        
                        if let Err(e) = db2.insert_activity(&activity) {
                            error!("Failed to insert clipboard activity: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub fn record_activity(&self, activity: Activity) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.db.insert_activity(&activity)?;
        self.metrics.increment_events();
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct AppInfo {
    name: String,
    bundle_id: String,
    window_title: Option<String>,
}

#[cfg(target_os = "macos")]
async fn get_active_app() -> Option<AppInfo> {
    use std::process::Command;
    
    let output = Command::new("osascript")
        .args(["-e", r#"
            tell application "System Events"
                set frontApp to first application process whose frontmost is true
                set appName to name of frontApp
                set bundleId to bundle identifier of frontApp
                try
                    set windowTitle to name of first window of frontApp
                on error
                    set windowTitle to ""
                end try
                return appName & "||" & bundleId & "||" & windowTitle
            end tell
        "#])
        .output()
        .ok()?;
    
    let result = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = result.trim().split("||").collect();
    
    if parts.len() >= 2 {
        Some(AppInfo {
            name: parts[0].to_string(),
            bundle_id: parts[1].to_string(),
            window_title: parts.get(2).map(|s| s.to_string()).filter(|s| !s.is_empty()),
        })
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
async fn get_active_app() -> Option<AppInfo> {
    None
}

#[cfg(target_os = "macos")]
fn get_clipboard_content() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use std::process::Command;
    
    let output = Command::new("osascript")
        .args(["-e", "the clipboard as text"])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(not(target_os = "macos"))]
fn get_clipboard_content() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(String::new())
}

async fn create_new_session(db: &Arc<Database>, app_name: &str) -> Option<String> {
    let session = crate::storage::Session::new("general");
    let session_id = session.id.clone();
    
    if let Err(e) = db.insert_session(&session) {
        error!("Failed to create session: {}", e);
        return None;
    }
    
    info!("Created new session: {}", session_id);
    Some(session_id)
}
