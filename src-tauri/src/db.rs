use std::collections::HashMap;
use std::env;

use dotenv::dotenv;
use lazy_static::lazy_static;
use log;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqliteConnectOptions, SqliteRow};
use sqlx::{Row, Sqlite, SqlitePool};
use tauri::command;

use crate::data::{AppPaths, DataState};
use crate::llm_providers::{LLMConfig, LLMProvider, Provider};
use crate::providers::ProviderData;
use crate::throw;
use crate::types::{Chat, Chats, Message, MessageBlock, MessageBlocks, MessageHistory, Model, Models};

lazy_static! {
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
			provider_name: "mistralai".to_string(),
			model_name: "mistral-large-latest".to_string(),
			model_display_name: "Mistral".to_string(),
			show: true,
			max_tokens: 32768,
			context_window: 32768,
		},
		Model {
			provider_name: "groqcloud".to_string(),
			model_name: "llama-3.1-70b-versatile".to_string(),
			model_display_name: "LLaMA".to_string(),
			show: true,
			max_tokens: 2048,
			context_window: 131072,
		},
	];
}

pub async fn init(app_paths: &AppPaths) -> Result<SqlitePool, String> {
	let exists = match Sqlite::database_exists(&app_paths.db).await {
		Ok(exists) => exists,
		Err(e) => throw!("Could not check if database exists: {}", e),
	};
	if !exists {
		if let Err(e) = std::fs::create_dir_all(&app_paths.app_dir) {
			throw!("Error creating parent folder: {}", e.to_string());
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

	for model in DEFAULT_MODELS.iter() {
		_ = insert_model(model, &pool).await;
	}

	Ok(pool)
}

// insert model into models table if it doesn't exist
pub async fn insert_model(model: &Model, pool: &SqlitePool) -> Result<(), String> {
	// check if model already exists
	let query = sqlx::query("SELECT * FROM models WHERE provider_name = $1 AND model_name = $2")
		.bind(&model.provider_name)
		.bind(&model.model_name)
		.fetch_one(pool)
		.await;

	if let Ok(_) = query {
		return Ok(());
	}
	let _ = sqlx::query("INSERT INTO models (provider_name, model_name, model_display_name, show, max_tokens, context_window) VALUES ($1, $2, $3, $4, $5, $6)")
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

#[command]
#[specta::specta]
pub async fn load_providers(data: DataState<'_>) -> Result<Vec<ProviderData>, String> {
	let data = data.0.lock().await;
	let query = "SELECT provider_name, api_key, display_name, api_key_valid FROM providers";
	let providers = sqlx::query_as::<_, ProviderData>(&query);
	match providers.fetch_all(&data.db_pool).await {
		Ok(providers) => {
			log::debug!("Loaded providers info {:?}", providers);
			return Ok(providers);
		}
		Err(e) => throw!("Error getting providers: {}", e),
	};
}

#[command]
#[specta::specta]
pub async fn set_api_key(provider: ProviderData, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let is_valid: bool = validate_api_key(&provider).await?;

	println!("API key for provider {} is valid: {}", &provider.provider_name, is_valid);

	let query = sqlx::query("UPDATE providers SET api_key = ?, api_key_valid = ? WHERE provider_name = ?")
		.bind(&provider.api_key)
		.bind(is_valid)
		.bind(&provider.provider_name);

	// Execute the update query for the current provider
	query
		.execute(&data.db_pool)
		.await
		.map_err(|e| format!("Error updating API key for provider {}: {:?}", &provider.provider_name, e))?;

	Ok(())
}

async fn validate_api_key(provider: &ProviderData) -> Result<bool, String> {
	let model = DEFAULT_MODELS.iter().find(|m| m.provider_name == provider.provider_name).unwrap();
	let llm_config = LLMConfig::default();

	let llm = Provider::new(&provider.provider_name, &provider.api_key);

	let messages = MessageHistory(vec![Message {
		id: "".to_string(),
		role: "user".to_string(),
		content: "Hello".to_string(),
		model_name: model.model_name.clone(),
		blocks: None,
	}]);

	match llm.send_message(&messages, &model.model_name, &llm_config).await {
		Ok(_) => return Ok(true),
		Err(e) => {
			log::error!("Error sending message to LLM: {}", e);
			return Err(e.to_string());
		}
	};
}

impl sqlx::FromRow<'_, SqliteRow> for Message {
	fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
		Ok(Message {
			id: row.try_get("id")?,
			role: row.try_get("role")?,
			content: row.try_get("content")?,
			model_name: row.try_get("model_name")?,
			blocks: None,
		})
	}
}

#[command]
#[specta::specta]
pub async fn get_models(data: DataState<'_>) -> Result<Models, String> {
	let data = data.0.lock().await;
	let models_query = "SELECT provider_name, model_name, model_display_name, show, max_tokens, context_window FROM models WHERE provider_name IN (SELECT provider_name FROM providers WHERE api_key != '') OR provider_name = 'local'";
	let models_query_result = sqlx::query_as::<_, Model>(models_query).fetch_all(&data.db_pool).await;
	match models_query_result {
		Ok(models) => Ok(Models(models)),
		Err(e) => {
			println!("Error fetching models from database: {}", e.to_string());
			Err(e.to_string())
		}
	}
}

#[command]
#[specta::specta]
pub async fn get_chats(data: DataState<'_>) -> Result<Chats, String> {
	let data = data.0.lock().await;
	let fetch_query = "SELECT id, display_name, creation_date, last_updated FROM chats WHERE archived = 'false' ORDER BY last_updated DESC";
	let chats = Chats(
		sqlx::query_as::<_, Chat>(fetch_query)
			.fetch_all(&data.db_pool)
			.await
			.map_err(|e| e.to_string())?,
	);
	Ok(chats)
}

#[command]
#[specta::specta]
pub async fn load_chat(chat_id: String, data: DataState<'_>) -> Result<Vec<Message>, String> {
	let data = data.0.lock().await;
	let fetch_query = "SELECT id, role, content, model_name FROM messages WHERE chat_id = $1";
	let messages_result = sqlx::query_as::<_, Message>(fetch_query).bind(&chat_id).fetch_all(&data.db_pool).await;

	match messages_result {
		Ok(mut messages) => {
			let message_blocks_fetch_query = "SELECT id, type_, language, raw_content, rendered_content, copied FROM message_blocks WHERE message_id = $1";
			for message in messages.iter_mut() {
				let _ = match sqlx::query_as::<_, MessageBlock>(message_blocks_fetch_query)
					.bind(&message.id)
					.fetch_all(&data.db_pool)
					.await
				{
					Ok(message_blocks) => message.blocks = Some(MessageBlocks(message_blocks)),
					Err(err) => {
						eprintln!("Error fetching message blocks from database: {}", err);
					}
				};
			}
			return Ok(messages);
		}
		Err(e) => {
			eprintln!("Error fetching messages from database: {}", e);
			Err(e.to_string())
		}
	}
}

pub async fn insert_message(new_message_id: &str, role: &str, message: &str, chat_id: &str, model_name: &str, data: DataState<'_>) {
	let insert_message_query: &str = "INSERT INTO messages (id, role, content, chat_id, model_name) VALUES ($1, $2, $3, $4, $5)";
	let _ = sqlx::query(insert_message_query)
		.bind(&new_message_id)
		.bind(&role)
		.bind(&message)
		.bind(&chat_id)
		.bind(&model_name)
		.execute(&data.0.lock().await.db_pool)
		.await;
}

pub async fn insert_message_blocks(message_id: &str, message_blocks: &MessageBlocks, data: DataState<'_>) {
	let insert_message_blocks_query: &str =
		"INSERT INTO message_blocks (message_id, type_, language, raw_content, rendered_content, copied) VALUES ($1, $2, $3, $4, $5, $6)";
	for block in message_blocks.iter() {
		let insert_message_blocks_query_result = sqlx::query(insert_message_blocks_query)
			.bind(&message_id)
			.bind(&block.type_)
			.bind(&block.language)
			.bind(&block.raw_content)
			.bind(&block.rendered_content)
			.bind(0)
			.execute(&data.0.lock().await.db_pool)
			.await;
		match insert_message_blocks_query_result {
			Ok(_) => (),
			Err(e) => {
				eprintln!("Error inserting message blocks into database: {}", e);
			}
		}
	}
}

pub async fn get_chat_display_name(chat_id: &str, data: DataState<'_>) -> Result<Option<(String,)>, sqlx::Error> {
	let chat_display_name_query: &str = "SELECT display_name FROM chats WHERE id = $1";
	sqlx::query_as(chat_display_name_query)
		.bind(&chat_id)
		.fetch_optional(&data.0.lock().await.db_pool)
		.await
}

pub async fn insert_chat_display_name(chat_id: &str, model_name: &str, display_name: &str, data: DataState<'_>) -> Result<(), String> {
	let insert_chat_display_name_query =
		"INSERT INTO chats (id, model, api_key_id, display_name, archived, last_updated) VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)";
	match sqlx::query(insert_chat_display_name_query)
		.bind(&chat_id)
		.bind(&model_name)
		.bind("NA")
		.bind(&display_name)
		.bind("false")
		.execute(&data.0.lock().await.db_pool)
		.await
	{
		Ok(_) => Ok(()),
		Err(e) => {
			eprintln!("Error inserting chat display name into database: {}", e);
			Err(e.to_string())
		}
	}
}

pub async fn get_api_key(provider_name: &str, data: DataState<'_>) -> Result<String, String> {
	let api_key_query: &str = "SELECT api_key FROM providers WHERE provider_name = $1";
	match sqlx::query_as::<_, (String,)>(api_key_query)
		.bind(&provider_name)
		.fetch_one(&data.0.lock().await.db_pool)
		.await
	{
		Ok(api_key) => Ok(api_key.0),
		// Models should not be provided if the API key is not set, therefore throw an error
		Err(e) => throw!("Error fetching API key for provider {}: {}", &provider_name, e),
	}
}

#[command]
#[specta::specta]
pub async fn read_api_keys_from_env(data: DataState<'_>) -> Result<(), String> {
	let data: tokio::sync::MutexGuard<'_, crate::data::Data> = data.0.lock().await;
	dotenv().ok();
	let development = env::var("DEVELOPMENT").unwrap_or_else(|_| "0".to_string());
	if development == "0" {
		println!("Not in development mode, skipping reading API keys from environment variables");
		return Ok(());
	}
	let mut api_keys = HashMap::new();
	api_keys.insert("google", env::var("google").unwrap_or("".to_string()));
	api_keys.insert("openai", env::var("openai").unwrap_or("".to_string()));
	api_keys.insert("anthropic", env::var("anthropic").unwrap_or("".to_string()));
	api_keys.insert("mistralai", env::var("mistralai").unwrap_or("".to_string()));
	api_keys.insert("groqcloud", env::var("groqcloud").unwrap_or("".to_string()));
	let insert_api_keys_query: &str = "UPDATE providers SET api_key=$1 WHERE provider_name = $2";
	for (provider_name, api_key) in api_keys.iter() {
		match sqlx::query(insert_api_keys_query)
			.bind(&api_key)
			.bind(&provider_name)
			.execute(&data.db_pool)
			.await
		{
			Ok(_) => {
				//println!("API key for provider {} saved to the database", &provider_name);
			}
			Err(e) => {
				eprintln!("Error saving API key for provider {}: {}", &provider_name, e);
				return Err(format!("Error saving API key for provider {}: {}", &provider_name, e));
			}
		}
	}
	return Ok(());
}

pub async fn get_messages(chat_id: &str, data: DataState<'_>) -> Result<MessageHistory, anyhow::Error> {
	let messages_query: &str = "SELECT id, role, content, model_name FROM messages WHERE chat_id = $1";
	let messages = sqlx::query_as::<_, Message>(messages_query)
		.bind(&chat_id)
		.fetch_all(&data.0.lock().await.db_pool)
		.await
		.map_err(|e| {
			eprintln!("Error fetching messages from database: {}", e);
			anyhow::anyhow!("Database error: {}", e)
		})?;
	Ok(MessageHistory(messages))
}

#[command]
#[specta::specta]
pub async fn rename_chat(chat_id: String, new_display_name: String, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let rename_chat_query: &str = "UPDATE chats SET display_name = $1 WHERE id = $2";
	match sqlx::query(rename_chat_query)
		.bind(&new_display_name)
		.bind(&chat_id)
		.execute(&data.db_pool)
		.await
	{
		Ok(_) => Ok(()),
		Err(e) => {
			eprintln!("Error renaming chat: {}", e);
			Err(e.to_string())
		}
	}
}

#[command]
#[specta::specta]
pub async fn archive_chat(chat_id: String, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let archive_chat_query: &str = "UPDATE chats SET archived = 'true' WHERE id = $1";
	match sqlx::query(archive_chat_query).bind(&chat_id).execute(&data.db_pool).await {
		Ok(_) => Ok(()),
		Err(e) => {
			eprintln!("Error archiving chat: {}", e);
			Err(e.to_string())
		}
	}
}

#[command]
#[specta::specta]
pub async fn delete_chat(chat_id: String, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let delete_chat_query: &str = "DELETE FROM chats WHERE id = $1";
	match sqlx::query(delete_chat_query).bind(&chat_id).execute(&data.db_pool).await {
		Ok(_) => Ok(()),
		Err(e) => {
			eprintln!("Error deleting chat: {}", e);
			Err(e.to_string())
		}
	}
}