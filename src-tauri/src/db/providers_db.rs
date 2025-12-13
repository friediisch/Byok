//! Provider-related database operations

use std::collections::HashMap;
use std::env;

use dotenv::dotenv;
use tauri::command;

use crate::data::DataState;
use crate::llm_providers::{LLMConfig, Provider};
use crate::providers::ProviderData;
use crate::throw;
use crate::types::{Message, MessageHistory};

use super::init::DEFAULT_MODELS;

/// Load all provider configurations
#[command]
#[specta::specta]
pub async fn load_providers(data: DataState<'_>) -> Result<Vec<ProviderData>, String> {
	let data = data.0.lock().await;
	let query = "SELECT provider_name, api_key, display_name, api_key_valid, base_url, api_scheme, is_custom FROM providers";
	let providers = sqlx::query_as::<_, ProviderData>(query);
	match providers.fetch_all(&data.db_pool).await {
		Ok(providers) => {
			log::debug!("Loaded providers info {:?}", providers);
			Ok(providers)
		}
		Err(e) => throw!("Error getting providers: {}", e),
	}
}

/// Set and validate an API key for a provider
#[command]
#[specta::specta]
pub async fn set_api_key(provider: ProviderData, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let is_valid = validate_api_key(&provider).await?;

	log::info!("API key for provider {} is valid: {}", &provider.provider_name, is_valid);

	let query = sqlx::query("UPDATE providers SET api_key = $1, api_key_valid = $2, base_url = $3 WHERE provider_name = $4")
		.bind(&provider.api_key)
		.bind(is_valid)
		.bind(&provider.base_url)
		.bind(&provider.provider_name);

	query
		.execute(&data.db_pool)
		.await
		.map_err(|e| format!("Error updating API key for provider {}: {:?}", &provider.provider_name, e))?;

	Ok(())
}

/// Validate an API key by sending a test message
async fn validate_api_key(provider: &ProviderData) -> Result<bool, String> {
	// For custom providers, try to find a model associated with this provider
	// or use a default test model name
	let model_name = DEFAULT_MODELS
		.iter()
		.find(|m| m.provider_name == provider.provider_name)
		.map(|m| m.model_name.clone())
		.unwrap_or_else(|| "gpt-3.5-turbo".to_string()); // Fallback for custom providers

	let llm_config = LLMConfig::default();
	let llm = Provider::from_provider_data(
		&provider.provider_name,
		&provider.api_key,
		provider.base_url.as_deref(),
		provider.api_scheme.as_deref(),
	).map_err(|e| format!("Failed to create provider: {}", e))?;

	let messages = MessageHistory(vec![Message {
		id: String::new(),
		role: "user".to_string(),
		content: "Hello".to_string(),
		model_name: model_name.clone(),
		blocks: None,
	}]);

	match llm.send_message(&messages, &model_name, &llm_config).await {
		Ok(_) => Ok(true),
		Err(e) => {
			log::error!("Error validating API key: {}", e);
			Err(e.to_string())
		}
	}
}

/// Add a new custom provider
#[command]
#[specta::specta]
pub async fn add_provider(provider: ProviderData, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;

	// Check if provider already exists
	let exists = sqlx::query("SELECT id FROM providers WHERE provider_name = $1")
		.bind(&provider.provider_name)
		.fetch_optional(&data.db_pool)
		.await
		.map_err(|e| format!("Error checking provider existence: {}", e))?;

	if exists.is_some() {
		return Err("A provider with this name already exists".to_string());
	}

	let query = "INSERT INTO providers (provider_name, api_key, display_name, api_key_valid, base_url, api_scheme, is_custom) VALUES ($1, $2, $3, $4, $5, $6, $7)";
	sqlx::query(query)
		.bind(&provider.provider_name)
		.bind(&provider.api_key)
		.bind(&provider.display_name)
		.bind(provider.api_key_valid)
		.bind(&provider.base_url)
		.bind(&provider.api_scheme)
		.bind(true) // is_custom = true for new providers
		.execute(&data.db_pool)
		.await
		.map_err(|e| format!("Error adding provider: {}", e))?;

	Ok(())
}

/// Update an existing provider
#[command]
#[specta::specta]
pub async fn update_provider(provider: ProviderData, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;

	let query = "UPDATE providers SET display_name = $1, base_url = $2, api_scheme = $3, api_key = $4 WHERE provider_name = $5";
	let result = sqlx::query(query)
		.bind(&provider.display_name)
		.bind(&provider.base_url)
		.bind(&provider.api_scheme)
		.bind(&provider.api_key)
		.bind(&provider.provider_name)
		.execute(&data.db_pool)
		.await
		.map_err(|e| format!("Error updating provider: {}", e))?;

	if result.rows_affected() == 0 {
		return Err("Provider not found".to_string());
	}

	Ok(())
}

/// Delete a custom provider
#[command]
#[specta::specta]
pub async fn delete_provider(provider_name: String, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;

	// Only allow deleting custom providers
	let is_custom: Option<(bool,)> = sqlx::query_as("SELECT is_custom FROM providers WHERE provider_name = $1")
		.bind(&provider_name)
		.fetch_optional(&data.db_pool)
		.await
		.map_err(|e| format!("Error checking provider: {}", e))?;

	match is_custom {
		Some((true,)) => {
			// Delete associated models first
			sqlx::query("DELETE FROM models WHERE provider_name = $1")
				.bind(&provider_name)
				.execute(&data.db_pool)
				.await
				.map_err(|e| format!("Error deleting provider models: {}", e))?;

			// Delete the provider
			sqlx::query("DELETE FROM providers WHERE provider_name = $1")
				.bind(&provider_name)
				.execute(&data.db_pool)
				.await
				.map_err(|e| format!("Error deleting provider: {}", e))?;

			Ok(())
		}
		Some((false,)) => Err("Cannot delete built-in providers".to_string()),
		None => Err("Provider not found".to_string()),
	}
}

/// Get the API key for a specific provider
pub async fn get_api_key(provider_name: &str, data: DataState<'_>) -> Result<String, String> {
	let query = "SELECT api_key FROM providers WHERE provider_name = $1";
	match sqlx::query_as::<_, (String,)>(query)
		.bind(provider_name)
		.fetch_one(&data.0.lock().await.db_pool)
		.await
	{
		Ok(api_key) => Ok(api_key.0),
		Err(e) => throw!("Error fetching API key for provider {}: {}", provider_name, e),
	}
}

/// Read API keys from environment variables (development mode only)
#[command]
#[specta::specta]
pub async fn read_api_keys_from_env(data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	dotenv().ok();

	let development = env::var("DEVELOPMENT").unwrap_or_else(|_| "0".to_string());
	if development == "0" {
		log::info!("Not in development mode, skipping reading API keys from environment variables");
		return Ok(());
	}

	let mut api_keys = HashMap::new();
	api_keys.insert("google", env::var("google").unwrap_or_default());
	api_keys.insert("openai", env::var("openai").unwrap_or_default());
	api_keys.insert("anthropic", env::var("anthropic").unwrap_or_default());
	api_keys.insert("mistralai", env::var("mistralai").unwrap_or_default());
	api_keys.insert("groqcloud", env::var("groqcloud").unwrap_or_default());

	let query = "UPDATE providers SET api_key = $1 WHERE provider_name = $2";
	for (provider_name, api_key) in api_keys.iter() {
		match sqlx::query(query).bind(api_key).bind(provider_name).execute(&data.db_pool).await {
			Ok(_) => {}
			Err(e) => {
				log::error!("Error saving API key for provider {}: {}", provider_name, e);
				return Err(format!("Error saving API key for provider {}: {}", provider_name, e));
			}
		}
	}

	Ok(())
}
