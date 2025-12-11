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
	let query = "SELECT provider_name, api_key, display_name, api_key_valid FROM providers";
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

	let query = sqlx::query("UPDATE providers SET api_key = $1, api_key_valid = $2 WHERE provider_name = $3")
		.bind(&provider.api_key)
		.bind(is_valid)
		.bind(&provider.provider_name);

	query
		.execute(&data.db_pool)
		.await
		.map_err(|e| format!("Error updating API key for provider {}: {:?}", &provider.provider_name, e))?;

	Ok(())
}

/// Validate an API key by sending a test message
async fn validate_api_key(provider: &ProviderData) -> Result<bool, String> {
	let model = DEFAULT_MODELS
		.iter()
		.find(|m| m.provider_name == provider.provider_name)
		.ok_or_else(|| format!("No default model found for provider: {}", provider.provider_name))?;

	let llm_config = LLMConfig::default();
	let llm = Provider::new(&provider.provider_name, &provider.api_key).map_err(|e| format!("Failed to create provider: {}", e))?;

	let messages = MessageHistory(vec![Message {
		id: String::new(),
		role: "user".to_string(),
		content: "Hello".to_string(),
		model_name: model.model_name.clone(),
		blocks: None,
	}]);

	match llm.send_message(&messages, &model.model_name, &llm_config).await {
		Ok(_) => Ok(true),
		Err(e) => {
			log::error!("Error validating API key: {}", e);
			Err(e.to_string())
		}
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
