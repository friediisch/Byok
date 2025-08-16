use anyhow::Result;
// use async_trait::async_trait;

use crate::llm_providers::anthropic::AnthropicProvider;
use crate::llm_providers::default::DefaultProvider;
use crate::types::MessageHistory;

use super::{LLMConfig, LLMProvider};

#[derive(Clone)]
pub enum Provider {
	Default(DefaultProvider),
	Anthropic(AnthropicProvider),
	// Mistral(MistralProvider),
	// Groq(GroqProvider),
}

impl Provider {
	pub fn new(provider_name: &str, api_key: &str) -> Self {
		match provider_name {
			"openai" => Self::Default(DefaultProvider::new(provider_name, api_key, "https://api.openai.com/v1/chat/completions")),
			"anthropic" => Self::Anthropic(AnthropicProvider::new(api_key)),
			_ => panic!("Unsupported provider: {}", provider_name),
		}
	}
}

impl LLMProvider for Provider {
	fn provider_name(&self) -> &str {
		match self {
			Provider::Default(p) => p.provider_name(),
			Provider::Anthropic(p) => p.provider_name(),
		}
	}

	async fn send_message(&self, messages: &MessageHistory, model: &str, config: &LLMConfig) -> Result<String> {
		match self {
			Provider::Default(p) => p.send_message(messages, model, config).await,
			Provider::Anthropic(p) => p.send_message(messages, model, config).await,
		}
	}
}
