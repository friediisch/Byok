//! Model-related database operations

use sqlx::SqlitePool;
use tauri::command;

use crate::data::DataState;
use crate::types::{Model, Models};

/// Insert a model into the database if it doesn't already exist
pub async fn insert_model(model: &Model, pool: &SqlitePool) -> Result<(), String> {
	// Check if model already exists
	let check_query = sqlx::query("SELECT * FROM models WHERE provider_name = $1 AND model_name = $2")
		.bind(&model.provider_name)
		.bind(&model.model_name)
		.fetch_one(pool)
		.await;

	if check_query.is_ok() {
		return Ok(());
	}

	let insert_query = "INSERT INTO models (provider_name, model_name, model_display_name, show, max_tokens, context_window) VALUES ($1, $2, $3, $4, $5, $6)";
	let _ = sqlx::query(insert_query)
		.bind(&model.provider_name)
		.bind(&model.model_name)
		.bind(&model.model_display_name)
		.bind(&model.show)
		.bind(&model.max_tokens)
		.bind(&model.context_window)
		.execute(pool)
		.await;
	Ok(())
}

/// Get models that have valid API keys configured
#[command]
#[specta::specta]
pub async fn get_models(data: DataState<'_>) -> Result<Models, String> {
	let data = data.0.lock().await;
	let query = "SELECT provider_name, model_name, model_display_name, show, max_tokens, context_window FROM models WHERE provider_name IN (SELECT provider_name FROM providers WHERE api_key != '') OR provider_name = 'local'";
	let result = sqlx::query_as::<_, Model>(query).fetch_all(&data.db_pool).await;
	match result {
		Ok(models) => Ok(Models(models)),
		Err(e) => {
			log::error!("Error fetching models from database: {}", e);
			Err(e.to_string())
		}
	}
}

/// Get all models regardless of API key status
#[command]
#[specta::specta]
pub async fn get_all_models(data: DataState<'_>) -> Result<Models, String> {
	let data = data.0.lock().await;
	let query = "SELECT provider_name, model_name, model_display_name, show, max_tokens, context_window FROM models";
	let result = sqlx::query_as::<_, Model>(query).fetch_all(&data.db_pool).await;
	match result {
		Ok(models) => Ok(Models(models)),
		Err(e) => {
			log::error!("Error fetching all models from database: {}", e);
			Err(e.to_string())
		}
	}
}

/// Add a new model
#[command]
#[specta::specta]
pub async fn add_model(model: Model, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;

	// Check if model already exists
	let exists = sqlx::query("SELECT id FROM models WHERE provider_name = $1 AND model_name = $2")
		.bind(&model.provider_name)
		.bind(&model.model_name)
		.fetch_optional(&data.db_pool)
		.await
		.map_err(|e| format!("Error checking model existence: {}", e))?;

	if exists.is_some() {
		return Err("A model with this provider and name already exists".to_string());
	}

	let query = "INSERT INTO models (provider_name, model_name, model_display_name, show, max_tokens, context_window) VALUES ($1, $2, $3, $4, $5, $6)";
	match sqlx::query(query)
		.bind(&model.provider_name)
		.bind(&model.model_name)
		.bind(&model.model_display_name)
		.bind(&model.show)
		.bind(&model.max_tokens)
		.bind(&model.context_window)
		.execute(&data.db_pool)
		.await
	{
		Ok(_) => Ok(()),
		Err(e) => {
			log::error!("Error adding model: {}", e);
			Err(e.to_string())
		}
	}
}

/// Update an existing model
#[command]
#[specta::specta]
pub async fn update_model(model: Model, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let query = "UPDATE models SET model_display_name = $1, show = $2, max_tokens = $3, context_window = $4 WHERE provider_name = $5 AND model_name = $6";
	match sqlx::query(query)
		.bind(&model.model_display_name)
		.bind(&model.show)
		.bind(&model.max_tokens)
		.bind(&model.context_window)
		.bind(&model.provider_name)
		.bind(&model.model_name)
		.execute(&data.db_pool)
		.await
	{
		Ok(result) => {
			if result.rows_affected() == 0 {
				Err("Model not found".to_string())
			} else {
				Ok(())
			}
		}
		Err(e) => {
			log::error!("Error updating model: {}", e);
			Err(e.to_string())
		}
	}
}

/// Delete a model
#[command]
#[specta::specta]
pub async fn delete_model(provider_name: String, model_name: String, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let query = "DELETE FROM models WHERE provider_name = $1 AND model_name = $2";
	match sqlx::query(query).bind(&provider_name).bind(&model_name).execute(&data.db_pool).await {
		Ok(result) => {
			if result.rows_affected() == 0 {
				Err("Model not found".to_string())
			} else {
				Ok(())
			}
		}
		Err(e) => {
			log::error!("Error deleting model: {}", e);
			Err(e.to_string())
		}
	}
}
