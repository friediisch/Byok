use sqlx::SqlitePool;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::settings::Settings;

/// Application paths configuration
#[derive(Clone)]
pub struct AppPaths {
	pub app_dir: PathBuf,
	pub settings_file: PathBuf,
	pub db: String,
}

impl AppPaths {
	/// Create AppPaths from the app identifier (for use before App is fully initialized)
	pub fn from_identifier(identifier: &str) -> Self {
		let app_dir = match env::var("DEVELOPMENT").is_ok() {
			true => env::current_dir().unwrap().join("appdata"),
			false => {
				// Compute the app data directory based on the platform
				#[cfg(target_os = "macos")]
				let base_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("~"));
				#[cfg(target_os = "windows")]
				let base_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
				#[cfg(target_os = "linux")]
				let base_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));

				base_dir.join(identifier)
			}
		};
		AppPaths {
			app_dir: app_dir.clone(),
			settings_file: app_dir.join("settings.json"),
			db: app_dir.join("byok.sqlite").to_string_lossy().to_string(),
		}
	}
}

// ============================================================================
// Separate State Types for Better Lock Granularity
// ============================================================================

/// Database pool state - no mutex needed, SqlitePool handles concurrency internally
pub struct DbPool(pub SqlitePool);

/// Type alias for database state
pub type DbState<'a> = State<'a, DbPool>;

/// Application settings state - wrapped in Arc<Mutex> for thread-safe mutation
pub struct AppSettings(pub Arc<Mutex<Settings>>);

impl AppSettings {
	pub fn new(settings: Settings) -> Self {
		Self(Arc::new(Mutex::new(settings)))
	}
}

/// Type alias for settings state
pub type SettingsState<'a> = State<'a, AppSettings>;

/// Application paths state - read-only, no mutex needed
pub struct PathsState(pub AppPaths);

/// Type alias for paths state
pub type AppPathsState<'a> = State<'a, PathsState>;

/// Main window state - wrapped in Arc for sharing
pub struct MainWindow(pub Arc<tauri::WebviewWindow>);

/// Type alias for window state  
pub type WindowState<'a> = State<'a, MainWindow>;

// ============================================================================
// Legacy Combined State (for gradual migration)
// ============================================================================

/// Combined data struct - kept for backwards compatibility during migration
/// TODO: Remove once all code is migrated to use separate states
pub struct Data {
	pub db_pool: SqlitePool,
	pub paths: AppPaths,
	pub window: tauri::WebviewWindow,
	pub settings: Settings,
}

pub type DataState<'a> = State<'a, ArcData>;

pub struct ArcData(pub Arc<Mutex<Data>>);

impl ArcData {
	pub fn new(data: Data) -> Self {
		Self(Arc::new(Mutex::new(data)))
	}
}
