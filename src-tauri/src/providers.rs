use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::prelude::FromRow;
use tauri::{command, Emitter};

use crate::db::get_api_key;
use crate::llm_providers::{LLMConfig, Provider};
use crate::{
	data::DataState,
	db::{get_chat_display_name, get_messages, insert_chat_display_name, insert_message, insert_message_blocks},
	types::{Message, MessageBlocks, MessageHistory},
	utils::render_message,
};

#[derive(Serialize, Deserialize, Clone, Debug, FromRow, Type)]
pub struct ProviderData {
	pub provider_name: String,
	pub api_key: String,
	pub display_name: String,
	pub api_key_valid: bool,
	pub base_url: Option<String>,
	pub api_scheme: Option<String>,
	pub is_custom: bool,
}

/// Maximum length for auto-generated chat titles
const MAX_DISPLAY_NAME_LENGTH: u32 = 32;

/// Save a user message to the database and emit event
async fn save_user_message(msg: &str, chat_id: &str, model_name: &str, data: DataState<'_>) -> Result<String, String> {
	let message_id = uuid::Uuid::new_v4().to_string();
	insert_message(&message_id, "user", msg, chat_id, model_name, data.clone()).await;

	let code_theme = data.0.lock().await.settings.code_theme.clone();
	let rendered_blocks = render_message(msg, &code_theme).await;
	insert_message_blocks(&message_id, &rendered_blocks, data.clone()).await;

	// Emit event that a new message is in the database
	let _ = data.0.lock().await.window.emit("newMessage", chat_id);

	Ok(message_id)
}

/// Ensure a chat exists in the database, creating it with a placeholder name if needed
async fn ensure_chat_exists(chat_id: &str, model_name: &str, data: DataState<'_>) -> Result<bool, String> {
	match get_chat_display_name(chat_id, data.clone()).await {
		Ok(Some(_)) => Ok(false), // Chat already exists
		Ok(None) => {
			// Create new chat with placeholder name
			let placeholder_name = format!("unnamed_new_chat_{}", chat_id);
			match insert_chat_display_name(chat_id, model_name, &placeholder_name, data.clone()).await {
				Ok(_) => {
					let _ = data.0.lock().await.window.emit("newChat", chat_id);
					Ok(true) // New chat created
				}
				Err(e) => {
					log::error!("Error inserting display name into database: {}", e);
					Err(e)
				}
			}
		}
		Err(e) => {
			log::error!("Error fetching display name from database: {}", e);
			Err(e.to_string())
		}
	}
}

/// Get provider data for a specific provider
async fn get_provider_data(provider_name: &str, data: DataState<'_>) -> Result<ProviderData, String> {
	if provider_name == "local" || provider_name == "ollama" {
		return Ok(ProviderData {
			provider_name: provider_name.to_string(),
			api_key: String::new(),
			display_name: provider_name.to_string(),
			api_key_valid: true,
			base_url: None,
			api_scheme: Some("ollama".to_string()),
			is_custom: false,
		});
	}
	let query = "SELECT provider_name, api_key, display_name, api_key_valid, base_url, api_scheme, is_custom FROM providers WHERE provider_name = $1";
	match sqlx::query_as::<_, ProviderData>(query)
		.bind(provider_name)
		.fetch_one(&data.0.lock().await.db_pool)
		.await
	{
		Ok(provider) => Ok(provider),
		Err(e) => Err(format!("Error fetching provider data for {}: {}", provider_name, e)),
	}
}

/// Send message to LLM and get response
async fn get_llm_response(llm: &Provider, messages: &MessageHistory, model_name: &str, config: &LLMConfig) -> String {
	match llm.send_message(messages, model_name, config).await {
		Ok(answer) => answer,
		Err(e) => {
			log::error!("Error sending message to LLM: {}", e);
			e.to_string()
		}
	}
}

/// Save assistant response to the database and emit event
async fn save_assistant_message(answer: &str, chat_id: &str, model_name: &str, data: DataState<'_>) -> Result<String, String> {
	let message_id = uuid::Uuid::new_v4().to_string();
	insert_message(&message_id, "assistant", answer, chat_id, model_name, data.clone()).await;

	let code_theme = data.0.lock().await.settings.code_theme.clone();
	let rendered_blocks: MessageBlocks = render_message(answer, &code_theme).await;
	insert_message_blocks(&message_id, &rendered_blocks, data.clone()).await;

	// Emit event that a new message is in the database
	let _ = data.0.lock().await.window.emit("newMessage", chat_id);

	Ok(message_id)
}

/// Generate a descriptive title for a chat using the LLM
async fn generate_chat_title(llm: &Provider, user_msg: &str, assistant_msg: &str, model_name: &str) -> String {
	let prompt = format!(
		"Based on the following conversation, create a short and descriptive title (3–6 words) \
		that summarizes the main topic or purpose of the exchange:\n\
		'user': '{}',\n\
		'assistant': '{}'\n\
		Your response will be used to name the chat, therefore omit any other content from your \
		response, keep it short and use the language used in the prompt.\n\
		Do not use quotation marks. Capitalize the first letter of your answer. \
		It is okay if your answer consists of keywords, it does not need to be a complete sentence.",
		user_msg, assistant_msg
	);

	let title_messages = MessageHistory(vec![Message {
		id: String::new(),
		role: "user".to_string(),
		content: prompt,
		model_name: model_name.to_string(),
		blocks: None,
	}]);

	let title_config = LLMConfig {
		temperature: 0.0,
		max_tokens: MAX_DISPLAY_NAME_LENGTH,
		top_p: None,
	};

	match llm.send_message(&title_messages, model_name, &title_config).await {
		Ok(title) => title,
		Err(e) => {
			log::error!("Error generating chat title: {}", e);
			e.to_string()
		}
	}
}

/// Update the chat display name in the database
async fn update_chat_display_name(chat_id: &str, new_name: &str, data: DataState<'_>) -> Result<(), String> {
	let query = "UPDATE chats SET display_name = $1 WHERE id = $2";
	sqlx::query(query)
		.bind(new_name)
		.bind(chat_id)
		.execute(&data.0.lock().await.db_pool)
		.await
		.map_err(|e| {
			log::error!("Error updating display name in database: {}", e);
			e.to_string()
		})?;

	let _ = data.0.lock().await.window.emit("newChat", ());
	Ok(())
}

/// Update the last_updated timestamp for a chat
async fn update_chat_timestamp(chat_id: &str, data: DataState<'_>) -> Result<(), String> {
	let query = "UPDATE chats SET last_updated = CURRENT_TIMESTAMP WHERE id = $1";
	sqlx::query(query)
		.bind(chat_id)
		.execute(&data.0.lock().await.db_pool)
		.await
		.map_err(|e| e.to_string())?;
	Ok(())
}

/// Handle chat title generation or timestamp update after receiving a response
async fn finalize_chat(llm: &Provider, chat_id: &str, user_msg: &str, assistant_msg: &str, model_name: &str, data: DataState<'_>) -> Result<(), String> {
	let chat_result = get_chat_display_name(chat_id, data.clone()).await;

	match chat_result {
		Ok(Some((display_name,))) => {
			if display_name.starts_with("unnamed_new_chat_") {
				// Generate a new title for the chat
				let new_title = generate_chat_title(llm, user_msg, assistant_msg, model_name).await;
				log::debug!("New chat display name: {}", new_title);
				update_chat_display_name(chat_id, &new_title, data).await?;
			} else {
				// Just update the timestamp
				update_chat_timestamp(chat_id, data).await?;
			}
		}
		Ok(None) => {
			log::warn!("Chat not found in the database: {}", chat_id);
		}
		Err(e) => {
			log::error!("Error fetching display name from database: {}", e);
		}
	}

	Ok(())
}

#[command]
#[specta::specta]
pub async fn get_message(msg: String, chat_id: String, provider_name: String, model_name: String, data: DataState<'_>) -> Result<String, String> {
	// 1. Save user message
	save_user_message(&msg, &chat_id, &model_name, data.clone()).await?;

	// 2. Ensure chat exists
	ensure_chat_exists(&chat_id, &model_name, data.clone()).await?;

	// 3. Get provider data (including API key, base_url, api_scheme)
	let provider_data = get_provider_data(&provider_name, data.clone()).await?;

	// 4. Get chat history
	let messages = get_messages(&chat_id, data.clone()).await.map_err(|e| e.to_string())?;

	// 5. Create LLM provider and get response
	let llm = Provider::from_provider_data(
		&provider_data.provider_name,
		&provider_data.api_key,
		provider_data.base_url.as_deref(),
		provider_data.api_scheme.as_deref(),
	).map_err(|e| format!("Failed to create provider: {}", e))?;

	let llm_config = LLMConfig::default();
	let answer = get_llm_response(&llm, &messages, &model_name, &llm_config).await;

	// 6. Save assistant response
	save_assistant_message(&answer, &chat_id, &model_name, data.clone()).await?;

	// 7. Finalize chat (generate title or update timestamp)
	finalize_chat(&llm, &chat_id, &msg, &answer, &model_name, data).await?;

	Ok(answer)
}
