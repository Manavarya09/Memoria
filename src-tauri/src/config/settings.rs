use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "data_directory")]
    pub data_dir: PathBuf,
    #[serde(rename = "log_level")]
    pub log_level: String,
    #[serde(rename = "activity_capture")]
    pub activity_capture: ActivityCaptureConfig,
    #[serde(rename = "screen_capture")]
    pub screen_capture: ScreenCaptureConfig,
    #[serde(rename = "privacy")]
    pub privacy: PrivacyConfig,
    #[serde(rename = "performance")]
    pub performance: PerformanceConfig,
    #[serde(rename = "developer")]
    pub developer: DeveloperConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityCaptureConfig {
    #[serde(rename = "enabled")]
    pub enabled: bool,
    #[serde(rename = "app_monitoring")]
    pub app_monitoring: bool,
    #[serde(rename = "file_monitoring")]
    pub file_monitoring: bool,
    #[serde(rename = "clipboard_monitoring")]
    pub clipboard_monitoring: bool,
    #[serde(rename = "browser_history")]
    pub browser_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureConfig {
    #[serde(rename = "enabled")]
    pub enabled: bool,
    #[serde(rename = "interval_seconds")]
    pub interval_seconds: u64,
    #[serde(rename = "active_window_only")]
    pub active_window_only: bool,
    #[serde(rename = "ocr_enabled")]
    pub ocr_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    #[serde(rename = "exclude_apps")]
    pub exclude_apps: Vec<String>,
    #[serde(rename = "exclude_directories")]
    pub exclude_directories: Vec<String>,
    #[serde(rename = "pause_on_lock")]
    pub pause_on_lock: bool,
    #[serde(rename = "retention_days")]
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(rename = "max_concurrent_ocr")]
    pub max_concurrent_ocr: usize,
    #[serde(rename = "embedding_batch_size")]
    pub embedding_batch_size: usize,
    #[serde(rename = "screenshot_cache_mb")]
    pub screenshot_cache_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperConfig {
    #[serde(rename = "debug_mode")]
    pub debug_mode: bool,
    #[serde(rename = "metrics_port")]
    pub metrics_port: u16,
}

impl Default for Settings {
    fn default() -> Self {
        let data_dir = ProjectDirs::from("com", "memoria", "Memoria")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~/.memoria"));

        let data_dir = shellexpand::tilde("~").into_owned();
        let data_dir = PathBuf::from(data_dir).join(".memoria");

        Self {
            data_dir: data_dir.clone(),
            log_level: "info".to_string(),
            activity_capture: ActivityCaptureConfig {
                enabled: true,
                app_monitoring: true,
                file_monitoring: true,
                clipboard_monitoring: true,
                browser_history: false,
            },
            screen_capture: ScreenCaptureConfig {
                enabled: true,
                interval_seconds: 30,
                active_window_only: true,
                ocr_enabled: true,
            },
            privacy: PrivacyConfig {
                exclude_apps: vec![
                    "1Password".to_string(),
                    "Bitwarden".to_string(),
                    "Keychain Access".to_string(),
                ],
                exclude_directories: vec![],
                pause_on_lock: true,
                retention_days: 90,
            },
            performance: PerformanceConfig {
                max_concurrent_ocr: 2,
                embedding_batch_size: 10,
                screenshot_cache_mb: 500,
            },
            developer: DeveloperConfig {
                debug_mode: false,
                metrics_port: 9090,
            },
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config_dir = ProjectDirs::from("com", "memoria", "Memoria")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~/.config/memoria"));

        let config_path = config_dir.join("settings.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let settings: Settings = toml::from_str(&content)?;
            info!("Loaded settings from {:?}", config_path);
            Ok(settings)
        } else {
            let settings = Self::default();
            std::fs::create_dir_all(&config_dir)?;
            let content = toml::to_string_pretty(&settings)?;
            std::fs::write(&config_path, content)?;
            info!("Created default settings at {:?}", config_path);
            Ok(settings)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config_dir = ProjectDirs::from("com", "memoria", "Memoria")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~/.config/memoria"));

        let config_path = config_dir.join("settings.toml");
        std::fs::create_dir_all(&config_dir)?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
