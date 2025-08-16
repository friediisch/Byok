use anyhow::Result;
// use async_trait::async_trait;

use crate::types::MessageHistory;

use super::LLMConfig;

pub trait LLMProvider {
	fn provider_name(&self) -> &str;
	async fn send_message(&self, messages: &MessageHistory, model: &str, config: &LLMConfig) -> Result<String>;
}
