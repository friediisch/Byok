use sqlx::SqlitePool;
use std::env;

use crate::settings::Settings;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Config, State};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppPaths {
	pub app_dir: PathBuf,
	pub settings_file: PathBuf,
	pub db: String,
	pub models: PathBuf,
}
impl AppPaths {
	pub fn from_tauri_config(config: &Config) -> Self {
		let app_dir = match env::var("DEVELOPMENT").is_ok() {
			true => env::current_dir().unwrap().join("appdata"),
			false => tauri::api::path::app_data_dir(config).unwrap(),
		};
		AppPaths {
			app_dir: app_dir.clone(),
			settings_file: app_dir.join("settings.json"),
			db: app_dir.join("byok.sqlite").to_string_lossy().to_string(),
			models: app_dir.join("models"),
		}
	}
}

pub struct Data {
	pub db_pool: SqlitePool,
	pub paths: AppPaths,
	pub window: tauri::Window,
	pub settings: Settings,
}

pub type DataState<'a> = State<'a, ArcData>;
pub struct ArcData(pub Arc<Mutex<Data>>);
impl ArcData {
	pub fn new(data: Data) -> Self {
		Self(Arc::new(Mutex::new(data)))
	}
}
