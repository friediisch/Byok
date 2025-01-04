use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LLMMessage {
	pub role: String,
	pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LLMConfig {
	pub temperature: f32,
	pub max_tokens: u32,
	pub top_p: Option<f32>,
}

impl Default for LLMConfig {
	fn default() -> Self {
		Self {
			temperature: 0.7,
			max_tokens: 4096,
			top_p: None,
		}
	}
}

#[derive(Debug)]
pub enum LLMError {
	Authentication(String),
	RateLimit(String),
	InvalidRequest(String),
	NetworkError(String),
	Other(String),
}
