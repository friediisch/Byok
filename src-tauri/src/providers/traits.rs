use crate::MessageHistory;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait LLMProvider {
	// Core methods that all providers must implement
	fn new(api_key: String) -> Self
	where
		Self: Sized;

	async fn send_message(&self, messages: &MessageHistory, model: &str) -> Result<String, Box<dyn std::error::Error>>;

	// Optional methods with default implementations
	fn get_default_params(&self) -> Value {
		serde_json::json!({
			"temperature": 0.7,
			"max_tokens": 4096
		})
	}

	fn validate_api_key(&self) -> bool {
		true // Default implementation assumes valid
	}

	fn get_provider_name(&self) -> &'static str;
}
