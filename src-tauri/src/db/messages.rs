//! Message-related database operations

use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use tauri::command;

use crate::data::DataState;
use crate::types::{Message, MessageBlock, MessageBlocks, MessageHistory};

// Implement FromRow for Message to map database rows
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

/// Load all messages for a chat, including their rendered blocks
#[command]
#[specta::specta]
pub async fn load_chat(chat_id: String, data: DataState<'_>) -> Result<Vec<Message>, String> {
	let data = data.0.lock().await;
	let fetch_query = "SELECT id, role, content, model_name FROM messages WHERE chat_id = $1";
	let messages_result = sqlx::query_as::<_, Message>(fetch_query).bind(&chat_id).fetch_all(&data.db_pool).await;

	match messages_result {
		Ok(mut messages) => {
			let blocks_query = "SELECT id, type_, language, raw_content, rendered_content, copied FROM message_blocks WHERE message_id = $1";
			for message in messages.iter_mut() {
				match sqlx::query_as::<_, MessageBlock>(blocks_query).bind(&message.id).fetch_all(&data.db_pool).await {
					Ok(message_blocks) => message.blocks = Some(MessageBlocks(message_blocks)),
					Err(err) => {
						log::error!("Error fetching message blocks from database: {}", err);
					}
				}
			}
			Ok(messages)
		}
		Err(e) => {
			log::error!("Error fetching messages from database: {}", e);
			Err(e.to_string())
		}
	}
}

/// Get all messages for a chat (without blocks, for LLM context)
pub async fn get_messages(chat_id: &str, data: DataState<'_>) -> Result<MessageHistory, anyhow::Error> {
	let query = "SELECT id, role, content, model_name FROM messages WHERE chat_id = $1";
	let messages = sqlx::query_as::<_, Message>(query)
		.bind(chat_id)
		.fetch_all(&data.0.lock().await.db_pool)
		.await
		.map_err(|e| {
			log::error!("Error fetching messages from database: {}", e);
			anyhow::anyhow!("Database error: {}", e)
		})?;
	Ok(MessageHistory(messages))
}

/// Insert a new message
pub async fn insert_message(message_id: &str, role: &str, content: &str, chat_id: &str, model_name: &str, data: DataState<'_>) {
	let query = "INSERT INTO messages (id, role, content, chat_id, model_name) VALUES ($1, $2, $3, $4, $5)";
	let _ = sqlx::query(query)
		.bind(message_id)
		.bind(role)
		.bind(content)
		.bind(chat_id)
		.bind(model_name)
		.execute(&data.0.lock().await.db_pool)
		.await;
}

/// Insert rendered message blocks for a message
pub async fn insert_message_blocks(message_id: &str, message_blocks: &MessageBlocks, data: DataState<'_>) {
	let query = "INSERT INTO message_blocks (message_id, type_, language, raw_content, rendered_content, copied) VALUES ($1, $2, $3, $4, $5, $6)";
	for block in message_blocks.iter() {
		let result = sqlx::query(query)
			.bind(message_id)
			.bind(&block.type_)
			.bind(&block.language)
			.bind(&block.raw_content)
			.bind(&block.rendered_content)
			.bind(0)
			.execute(&data.0.lock().await.db_pool)
			.await;
		if let Err(e) = result {
			log::error!("Error inserting message blocks into database: {}", e);
		}
	}
}
