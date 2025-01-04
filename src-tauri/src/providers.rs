pub mod anthropic;
pub mod groqcloud;
pub mod mistralai;
pub mod openai;

// use langchain_rust::language_models::options::CallOptions;
use langchain_rust::memory::SimpleMemory;
use langchain_rust::schemas::messages::Message as LangChainMessage;
use langchain_rust::schemas::BaseMemory;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::prelude::FromRow;
use tauri::command;

use crate::db::get_api_key;
use crate::llm_providers::{LLMConfig, LLMProvider};
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
}

#[command]
#[specta::specta]
pub async fn get_message(msg: String, chat_id: String, provider_name: String, model_name: String, data: DataState<'_>) -> Result<String, String> {
	let messages: MessageHistory;
	let mut api_key: String = "".to_string();

	let new_message_id = uuid::Uuid::new_v4().to_string();
	insert_message(&new_message_id, "user", &msg, &chat_id, &model_name, data.clone()).await;
	let code_theme = &data.0.lock().await.settings.code_theme.clone();
	insert_message_blocks(&new_message_id, &render_message(&msg, code_theme).await, data.clone()).await;

	// emit event that a new message is in the database
	let _ = data.0.lock().await.window.emit("newMessage", &chat_id);

	match &get_chat_display_name(&chat_id, data.clone()).await {
		// If the display name exists, do nothing
		Ok(Some(_display_name)) => {}
		// If the display name does not exist, insert a new one
		Ok(None) => {
			match insert_chat_display_name(&chat_id, &model_name, &format!("unnamed_new_chat_{}", &chat_id), data.clone()).await {
				Ok(_) => {
					// emit event that a new chat is in the database
					let _ = data.0.lock().await.window.emit("newChat", &chat_id);
				}
				Err(e) => {
					eprintln!("Error inserting display name into database: {}", e);
				}
			}
		}
		Err(e) => {
			eprintln!("Error fetching display name from database: {}", e);
		}
	}

	if provider_name != "local" {
		// Get the API key from the providers table
		api_key = match get_api_key(&provider_name, data.clone()).await {
			Ok(s) => s,
			Err(e) => return Err(e),
		}
	}

	// Get the messages for the current chat from the messages table (including the latest user's message)
	messages = match get_messages(&chat_id, data.clone()).await {
		Ok(messages) => messages,
		Err(e) => {
			return Err(e.to_string());
		}
	};

	// let prompt_args
	let mut memory = SimpleMemory::new();
	//for message in messages.messages.iter().take(messages.messages.len() - 1) {
	for message in messages.iter().take(messages.len()) {
		memory.add_message({
			match message.role.as_str() {
				"user" => LangChainMessage::new_human_message(message.content.clone()),
				"assistant" => LangChainMessage::new_ai_message(message.content.clone()),
				_ => {
					panic!("Invalid message type: {}", message.role)
				}
			}
		});
	}

	// let call_options = CallOptions {
	// 	candidate_count: None,
	// 	max_tokens: Some(4096),
	// 	temperature: Some(0.7),
	// 	stop_words: None,
	// 	streaming_func: None,
	// 	top_k: None,
	// 	top_p: None,
	// 	seed: None,
	// 	min_length: None,
	// 	max_length: None,
	// 	n: None,
	// 	repetition_penalty: None,
	// 	frequency_penalty: None,
	// 	presence_penalty: None,
	// 	functions: None,
	// 	function_call_behavior: None,
	// };

	let llm_config = LLMConfig::default();

	let llm: LLMProvider = LLMProvider::new(&provider_name, api_key);

	let answer = match llm.send_message(&messages, &model_name, &llm_config).await {
		Ok(answer) => answer,
		Err(e) => {
			log::error!("Error sending message to LLM: {}", e);
			e.to_string()
		}
	};

	let new_answer_id = uuid::Uuid::new_v4().to_string();
	insert_message(&new_answer_id, "assistant", &answer, &chat_id, &model_name, data.clone()).await;
	let rendered_answer: MessageBlocks = render_message(&answer, &data.0.lock().await.settings.code_theme).await;
	insert_message_blocks(&new_answer_id, &rendered_answer, data.clone()).await;

	// emit event that a new message is in the database
	let _ = data.0.lock().await.window.emit("newMessage", &chat_id);

	let chats_result = get_chat_display_name(&chat_id, data.clone()).await;

	const MAX_DISPLAY_NAME_LENGTH: u32 = 32;

	match chats_result {
		Ok(Some((display_name,))) => {
			match display_name.starts_with("unnamed_new_chat_") {
				true => {
					let display_name_messages: MessageHistory = MessageHistory(vec![Message {
						id: "".to_string(),
						role: "user".to_string(),
						content: format!(
							"Please respond with the topic of the thread for these two messages:
								'user': '{msg}',
								'assistant': '{answer}'
								Your response will be used to name the chat, therefore omit any other content from your response, keep it short and use the language used in the prompt.
								Do not use quotation marks. Capitalize the first letter of your answer. It is okay if your answer consists of keywords, it does not need to be a complete sentence."
						),
						model_name: model_name.clone(),
						blocks: None,
					}]);

					let llm_config = LLMConfig {
						temperature: 0.0,
						max_tokens: MAX_DISPLAY_NAME_LENGTH,
						top_p: None,
					};

					let new_chat_display_name = match llm.send_message(&display_name_messages, &model_name, &llm_config).await {
						Ok(answer) => answer,
						Err(e) => {
							log::error!("Error sending message to LLM: {}", e);
							e.to_string()
						}
					};

					log::debug!("New chat display name: {}", new_chat_display_name);

					let update_chat_display_name_query: &str = "UPDATE chats SET display_name = $1 WHERE id = $2";
					let _ = sqlx::query(update_chat_display_name_query)
						.bind(&new_chat_display_name)
						.bind(&chat_id)
						.execute(&data.0.lock().await.db_pool)
						.await
						.map_err(|e| {
							eprintln!("Error updating display name in database: {}", e);
							e.to_string()
						})?;
					// emit event saying there are new chats
					let _ = &data.0.lock().await.window.emit("newChat", ());
					//let _ = data.window.emit("newMessage", &chat_id);
				}
				false => {
					//let data = data.0.lock().await;
					// update the last_updated field in the chats database to the current time
					let update_last_updated_query: &str = "UPDATE chats SET last_updated = CURRENT_TIMESTAMP WHERE id = $1";
					let _ = sqlx::query(update_last_updated_query)
						.bind(&chat_id)
						.execute(&data.0.lock().await.db_pool)
						.await
						.map_err(|e| e.to_string())?;
				}
			}
		}
		Ok(None) => {
			eprintln!("Chat not found in the database");
		}
		Err(e) => {
			eprintln!("Error fetching display name from database: {}", e);
		}
	}
	Ok(answer)
}
