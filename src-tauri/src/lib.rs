use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub mod activity;
pub mod capture;
pub mod processing;
pub mod embedding;
pub mod knowledge;
pub mod search;
pub mod timeline;
pub mod storage;
pub mod config;
pub mod ipc;
pub mod utils;

use config::Settings;
use storage::Database;
use activity::ActivityManager;
use embedding::EmbeddingManager;
use search::SearchEngine;
use timeline::TimelineManager;
use knowledge::KnowledgeGraph;
use utils::metrics::Metrics;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub activity_manager: Arc<ActivityManager>,
    pub embedding_manager: Arc<EmbeddingManager>,
    pub search_engine: Arc<SearchEngine>,
    pub timeline_manager: Arc<TimelineManager>,
    pub knowledge_graph: Arc<KnowledgeGraph>,
    pub settings: Arc<RwLock<Settings>>,
    pub metrics: Arc<Metrics>,
    pub is_paused: Arc<RwLock<bool>>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let settings = Settings::load()?;
        let db = Arc::new(Database::new(&settings.data_dir)?);
        let metrics = Arc::new(Metrics::new());
        
        let settings_arc = Arc::new(RwLock::new(settings));
        
        let activity_manager = Arc::new(ActivityManager::new(
            Arc::clone(&db),
            Arc::clone(&settings_arc),
            Arc::clone(&metrics),
        ));
        
        let embedding_manager = Arc::new(EmbeddingManager::new(
            Arc::clone(&db),
            Arc::clone(&settings_arc),
            Arc::clone(&metrics),
        ));
        
        let search_engine = Arc::new(SearchEngine::new(
            Arc::clone(&db),
            Arc::clone(&embedding_manager),
        ));
        
        let timeline_manager = Arc::new(TimelineManager::new(
            Arc::clone(&db),
            Arc::clone(&settings_arc),
        ));
        
        let knowledge_graph = Arc::new(KnowledgeGraph::new(
            Arc::clone(&db),
            Arc::clone(&settings_arc),
        ));
        
        Ok(Self {
            db,
            activity_manager,
            embedding_manager,
            search_engine,
            timeline_manager,
            knowledge_graph,
            settings: settings_arc,
            metrics,
            is_paused: Arc::new(RwLock::new(false)),
        })
    }
}

/// Initializes logging with a daily-rotating file appender and stdout output.
///
/// Returns a `WorkerGuard` that **must** be held for the lifetime of the
/// application.  Dropping the guard causes the non-blocking writer to flush
/// remaining logs and stop accepting new writes, so storing it in the caller
/// is essential for reliable file logging.
pub fn init_logging(data_dir: &std::path::Path) -> Result<tracing_appender::non_blocking::WorkerGuard, Box<dyn std::error::Error + Send + Sync>> {
    let logs_dir = data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        logs_dir,
        "memoria.log",
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout));

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Memoria logging initialized");

    Ok(guard)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = Settings::load().unwrap_or_default();
    
    // Hold the guard for the lifetime of the application so the
    // non-blocking file writer keeps flushing logs to disk.
    let _log_guard = match init_logging(&settings.data_dir) {
        Ok(guard) => Some(guard),
        Err(e) => {
            eprintln!("Failed to initialize logging: {}", e);
            None
        }
    };
    
    info!("Starting Memoria v{}", env!("CARGO_PKG_VERSION"));
    
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    
    let app_state = runtime.block_on(async {
        AppState::new().await.expect("Failed to initialize app state")
    });
    
    let state_for_async = Arc::new(app_state);
    let state_for_tauri = Arc::clone(&state_for_async);
    
    runtime.spawn(async {
        let state = state_for_async;
        
        let enabled = {
            let guard = state.settings.read();
            guard.activity_capture.enabled
        };
        
        if enabled {
            if let Err(e) = state.activity_manager.start().await {
                error!("Activity manager error: {}", e);
            }
        }
        
        let screen_enabled = {
            let guard = state.settings.read();
            guard.screen_capture.enabled
        };
        
        if screen_enabled {
            let capture_state = Arc::clone(&state);
            tokio::spawn(async move {
                capture::start_screen_capture(capture_state).await;
            });
        }
        
        let embedding_state = Arc::clone(&state);
        tokio::spawn(async move {
            embedding::start_indexing_worker(embedding_state).await;
        });
    });
    
    let app_state_owned = Arc::try_unwrap(state_for_tauri)
        .unwrap_or_else(|arc| (*arc).clone());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state_owned)
        .invoke_handler(tauri::generate_handler![
            ipc::commands::search_memories,
            ipc::commands::get_timeline,
            ipc::commands::get_memory_graph,
            ipc::commands::toggle_pause,
            ipc::commands::get_status,
            ipc::commands::get_settings,
            ipc::commands::update_settings,
            ipc::commands::delete_memory,
            ipc::commands::get_debug_metrics,
            ipc::commands::preview_activity,
        ])
        .setup(|_app| {
            info!("Memoria setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
