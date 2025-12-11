//! Chat-related database operations

use tauri::command;

use crate::data::DataState;
use crate::types::{Chat, Chats};

/// Get all non-archived chats, ordered by last updated
#[command]
#[specta::specta]
pub async fn get_chats(data: DataState<'_>) -> Result<Chats, String> {
	let data = data.0.lock().await;
	let query = "SELECT id, display_name, creation_date, last_updated FROM chats WHERE archived = 'false' ORDER BY last_updated DESC";
	let chats = Chats(sqlx::query_as::<_, Chat>(query).fetch_all(&data.db_pool).await.map_err(|e| e.to_string())?);
	Ok(chats)
}

/// Get the display name for a specific chat
pub async fn get_chat_display_name(chat_id: &str, data: DataState<'_>) -> Result<Option<(String,)>, sqlx::Error> {
	let query = "SELECT display_name FROM chats WHERE id = $1";
	sqlx::query_as(query).bind(chat_id).fetch_optional(&data.0.lock().await.db_pool).await
}

/// Insert a new chat with a display name
pub async fn insert_chat_display_name(chat_id: &str, model_name: &str, display_name: &str, data: DataState<'_>) -> Result<(), String> {
	let query = "INSERT INTO chats (id, model, api_key_id, display_name, archived, last_updated) VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)";
	match sqlx::query(query)
		.bind(chat_id)
		.bind(model_name)
		.bind("NA")
		.bind(display_name)
		.bind("false")
		.execute(&data.0.lock().await.db_pool)
		.await
	{
		Ok(_) => Ok(()),
		Err(e) => {
			log::error!("Error inserting chat display name into database: {}", e);
			Err(e.to_string())
		}
	}
}

/// Rename a chat
#[command]
#[specta::specta]
pub async fn rename_chat(chat_id: String, new_display_name: String, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let query = "UPDATE chats SET display_name = $1 WHERE id = $2";
	match sqlx::query(query).bind(&new_display_name).bind(&chat_id).execute(&data.db_pool).await {
		Ok(_) => Ok(()),
		Err(e) => {
			log::error!("Error renaming chat: {}", e);
			Err(e.to_string())
		}
	}
}

/// Archive a chat (soft delete)
#[command]
#[specta::specta]
pub async fn archive_chat(chat_id: String, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let query = "UPDATE chats SET archived = 'true' WHERE id = $1";
	match sqlx::query(query).bind(&chat_id).execute(&data.db_pool).await {
		Ok(_) => Ok(()),
		Err(e) => {
			log::error!("Error archiving chat: {}", e);
			Err(e.to_string())
		}
	}
}

/// Permanently delete a chat
#[command]
#[specta::specta]
pub async fn delete_chat(chat_id: String, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	let query = "DELETE FROM chats WHERE id = $1";
	match sqlx::query(query).bind(&chat_id).execute(&data.db_pool).await {
		Ok(_) => Ok(()),
		Err(e) => {
			log::error!("Error deleting chat: {}", e);
			Err(e.to_string())
		}
	}
}
