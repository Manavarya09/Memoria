use std::sync::Arc;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, debug};
use crate::storage::{Database, Activity};
use crate::config::Settings;
use crate::utils::Metrics;
use crate::AppState;

pub async fn start_screen_capture(state: Arc<AppState>) {
    info!("Starting Screen Capture Pipeline");
    
    let db = Arc::clone(&state.db);
    let settings = state.settings.clone();
    let metrics = Arc::clone(&state.metrics);
    
    loop {
        let (enabled, interval, ocr_enabled, data_dir) = {
            let guard = settings.read();
            (
                guard.screen_capture.enabled,
                guard.screen_capture.interval_seconds,
                guard.screen_capture.ocr_enabled,
                guard.data_dir.clone(),
            )
        };
        
        if !enabled {
            sleep(Duration::from_secs(5)).await;
            continue;
        }
        
        if let Some(screenshot) = capture_screen(&data_dir).await {
            metrics.increment_screenshots();
            debug!("Captured screenshot");
            
            let activity = Activity::new("screenshot");
            
            if let Err(e) = db.insert_activity(&activity) {
                error!("Failed to insert screenshot activity: {}", e);
            }
            
            if ocr_enabled {
                let ocr_text = process_ocr(&screenshot).await;
                
                if let Some(text) = ocr_text {
                    metrics.increment_ocr();
                    debug!("OCR extracted {} characters", text.len());
                    
                    let mut activity = Activity::new("ocr_text");
                    activity.content = Some(text);
                    
                    if let Err(e) = db.insert_activity(&activity) {
                        error!("Failed to insert OCR activity: {}", e);
                    }
                }
            }
        }
        
        sleep(Duration::from_secs(interval)).await;
    }
}

#[cfg(target_os = "macos")]
async fn capture_screen(data_dir: &PathBuf) -> Option<PathBuf> {
    use std::process::Command;
    
    let screenshots_dir = data_dir.join("screenshots");
    std::fs::create_dir_all(&screenshots_dir).ok()?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let output_path = screenshots_dir.join(format!("screenshot_{}.png", timestamp));
    
    let output = Command::new("screencapture")
        .args(["-x", "-t", "png", output_path.to_str()?])
        .output()
        .ok()?;
    
    if output.status.success() && output_path.exists() {
        Some(output_path)
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
async fn capture_screen(_data_dir: &PathBuf) -> Option<PathBuf> {
    None
}

#[cfg(target_os = "macos")]
async fn process_ocr(image_path: &PathBuf) -> Option<String> {
    run_vision_ocr(image_path).await
}

#[cfg(not(target_os = "macos"))]
async fn process_ocr(_image_path: &PathBuf) -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
async fn run_vision_ocr(image_path: &PathBuf) -> Option<String> {
    use std::process::Command;
    
    let path_str = image_path.to_str().unwrap_or("");
    let escaped_path = path_str.replace("\\", "\\\\").replace("\"", "\\\"");
    
    let script = format!(r#"
        use framework "Vision"
        use framework "AppKit"
        
        set imagePath to "{}"
        set imageURL to current application's |NSURL|'s fileURLWithPath:imagePath
        set image to current application's NSImage's alloc()'s initWithContentsOfURL:imageURL
        
        if image is missing value then return ""
        
        set request to current application's VNRecognizeTextRequest's new()
        request's setRecognitionLevel:current application's VNRequestTextRecognitionLevelAccurate
        request's setRecognitionLanguages:@["en-US"]
        
        set handler to current application's VNImageRequestHandler's alloc()'s initWithImage:image options:{{}}
        handler's performRequests:@[request] error:(missing value)
        
        set observations to request's recognizedObservations()
        set resultText to ""
        
        repeat with observation in observations
            set recognitionCandidates to observation's topCandidates(1)
            set firstCandidate to recognitionCandidates's firstObject()
            set text to firstCandidate's |string|()
            set resultText to resultText & text & "
        end repeat
        
        return resultText
    "#, escaped_path);
    
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .ok()?;
    
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
