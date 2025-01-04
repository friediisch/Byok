use anyhow::Result;
// use async_trait::async_trait;

use crate::llm_providers::openai::OpenAIProvider;
use crate::types::MessageHistory;

use super::LLMConfig;

#[derive(Clone)]
pub enum LLMProvider {
	OpenAI(OpenAIProvider),
	// Anthropic(AnthropicProvider),
	// Mistral(MistralProvider),
	// Groq(GroqProvider),
}

// #[async_trait]
// pub trait LLMProvider {
// 	fn new(api_key: String) -> Self;
// 	async fn send_message(&self, messages: &MessageHistory, model: &str) -> Result<String>;
// 	fn get_provider_name(&self) -> &'static str;
// }

impl LLMProvider {
	pub fn new(provider_name: &str, api_key: String) -> Self {
		match provider_name {
			"openai" => Self::OpenAI(OpenAIProvider::new(api_key)),
			// "anthropic" => Self::Anthropic(AnthropicProvider::new(api_key)),
			// "mistral" => Self::Mistral(MistralProvider::new(api_key)),
			// "groq" => Self::Groq(GroqProvider::new(api_key)),
			_ => panic!("Unsupported provider: {}", provider_name),
		}
	}

	pub async fn send_message(&self, messages: &MessageHistory, model: &str, config: &LLMConfig) -> Result<String> {
		match self {
			Self::OpenAI(provider) => provider.send_message(messages, model, config).await,
			// Self::Anthropic(provider) => provider.send_message(messages, model).await,
			// Self::Mistral(provider) => provider.send_message(messages, model).await,
			// Self::Groq(provider) => provider.send_message(messages, model).await,
		}
	}
}
