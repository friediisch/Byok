//! Database initialization and configuration

use lazy_static::lazy_static;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Sqlite, SqlitePool};

use crate::data::AppPaths;
use crate::throw;
use crate::types::Model;

use super::models::insert_model;

lazy_static! {
	/// Default models for each provider.
	/// Provider names must match those in the database providers table.
	pub static ref DEFAULT_MODELS: Vec<Model> = vec![
		Model {
			provider_name: "openai".to_string(),
			model_name: "chatgpt-4o-latest".to_string(),
			model_display_name: "ChatGPT".to_string(),
			show: true,
			max_tokens: 16384,
			context_window: 128000,
		},
		Model {
			provider_name: "anthropic".to_string(),
			model_name: "claude-3-5-sonnet-latest".to_string(),
			model_display_name: "Claude".to_string(),
			show: true,
			max_tokens: 8192,
			context_window: 200000,
		},
		Model {
			// Database uses "mistralai", Provider::new handles both "mistral" and "mistralai"
			provider_name: "mistralai".to_string(),
			model_name: "mistral-large-latest".to_string(),
			model_display_name: "Mistral".to_string(),
			show: true,
			max_tokens: 32768,
			context_window: 32768,
		},
		Model {
			// Database uses "groqcloud", Provider::new handles both "groq" and "groqcloud"
			provider_name: "groqcloud".to_string(),
			model_name: "llama-3.1-70b-versatile".to_string(),
			model_display_name: "LLaMA".to_string(),
			show: true,
			max_tokens: 2048,
			context_window: 131072,
		},
	];
}

/// Initialize the database, running migrations and inserting default models
pub async fn init(app_paths: &AppPaths) -> Result<SqlitePool, String> {
	let exists = match Sqlite::database_exists(&app_paths.db).await {
		Ok(exists) => exists,
		Err(e) => throw!("Could not check if database exists: {}", e),
	};

	if !exists {
		if let Err(e) = std::fs::create_dir_all(&app_paths.app_dir) {
			throw!("Error creating parent folder: {}", e);
		}
		match Sqlite::create_database(&app_paths.db).await {
			Ok(_) => {}
			Err(e) => throw!("Could not create database: {}", e),
		}
	}

	let connect_options = SqliteConnectOptions::new().filename(&app_paths.db);
	let pool = match SqlitePool::connect_with(connect_options).await {
		Ok(pool) => pool,
		Err(e) => throw!("Could not open database: {}", e),
	};

	match sqlx::migrate!("./migrations").run(&pool).await {
		Ok(_) => {}
		Err(e) => throw!("Could not run database migrations: {}", e),
	};

	// Insert default models
	for model in DEFAULT_MODELS.iter() {
		let _ = insert_model(model, &pool).await;
	}

	Ok(pool)
}
