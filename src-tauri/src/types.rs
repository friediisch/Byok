use core::fmt;

use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Type, Clone)]
pub struct Message {
	pub id: String,
	pub role: String,
	pub content: String,
	pub model_name: String,
	pub blocks: Option<MessageBlocks>,
}

// impl fmt::Display for Message {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		write!(f, "{}", self.content)
// 	}
// }

// impl Message {
// 	pub fn render(&self, provider_name: &str) -> Value {
// 		match provider_name {
// 			"openai" => {
// 				serde_json::json!({
// 					"role": self.role,
// 					"content": self.content
// 				})
// 			}
// 			_ => serde_json::Value::Null,
// 		}
// 	}
// }

#[derive(Deref, Serialize)]
pub struct MessageHistory(pub Vec<Message>);

// impl Into<String> for MessageHistory {
// 	fn into(self) -> String {
// 		serde_json::to_string(&self).unwrap()
// 	}
// }

#[derive(Serialize, Deserialize, Debug, FromRow, Type, Clone)]
pub struct MessageBlock {
	pub id: Option<i32>,
	pub type_: String,
	pub language: Option<String>,
	pub raw_content: String,
	pub rendered_content: String,
	pub copied: Option<bool>,
}

impl fmt::Display for MessageBlock {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.language {
			Some(lang) => write!(
				f,
				"{{\"type\": \"{}\", \"language\": \"{}\", \"content\": \"{}\"}}",
				self.type_, lang, self.rendered_content
			),
			None => write!(f, "{{\"type\": \"{}\", \"content\": \"{}\"}}", self.type_, self.rendered_content),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Type, Clone, Deref, DerefMut)]
pub struct MessageBlocks(pub Vec<MessageBlock>);

#[derive(Serialize, Deserialize, Debug, Type, FromRow, Clone)]
pub struct Model {
	pub provider_name: String,
	pub model_name: String,
	pub model_display_name: String,
	pub show: bool,
	pub max_tokens: u32,
	pub context_window: u32,
}

#[derive(Serialize, Deserialize, Debug, Type, FromRow, Clone, Deref)]
pub struct Models(pub Vec<Model>);

#[derive(Serialize, Deserialize, Type, Debug, FromRow, Clone)]
pub struct Chat {
	pub id: String,
	pub display_name: String,
	pub creation_date: String,
	pub last_updated: String,
}

#[derive(Serialize, Deserialize, Type, Debug, Deref)]
pub struct Chats(pub Vec<Chat>);
