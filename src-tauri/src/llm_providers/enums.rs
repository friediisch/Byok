use anyhow::{anyhow, Result};
use llm::{
	builder::{LLMBackend, LLMBuilder},
	chat::ChatMessage,
	LLMProvider,
};

use crate::types::MessageHistory;

use super::LLMConfig;

/// Supported API schemes for custom providers
#[derive(Clone, Debug)]
pub enum ApiScheme {
	OpenAI,
	Anthropic,
	Groq,
	Mistral,
	Ollama,
}

impl ApiScheme {
	pub fn from_str(s: &str) -> Option<Self> {
		match s.to_lowercase().as_str() {
			"openai" => Some(Self::OpenAI),
			"anthropic" => Some(Self::Anthropic),
			"groq" => Some(Self::Groq),
			"mistral" => Some(Self::Mistral),
			"ollama" => Some(Self::Ollama),
			_ => None,
		}
	}

	pub fn to_backend(&self) -> LLMBackend {
		match self {
			Self::OpenAI => LLMBackend::OpenAI,
			Self::Anthropic => LLMBackend::Anthropic,
			Self::Groq => LLMBackend::Groq,
			Self::Mistral => LLMBackend::Mistral,
			Self::Ollama => LLMBackend::Ollama,
		}
	}
}

/// Supported LLM providers
#[derive(Clone, Debug)]
pub enum Provider {
	OpenAI { api_key: String },
	Anthropic { api_key: String },
	Groq { api_key: String },
	Mistral { api_key: String },
	Ollama { base_url: Option<String> },
	Custom {
		api_key: String,
		base_url: String,
		api_scheme: ApiScheme,
	},
}

impl Provider {
	/// Create a new provider from a provider name and API key
	/// Handles multiple name variants (e.g., "mistral" and "mistralai")
	pub fn new(provider_name: &str, api_key: &str) -> Result<Self> {
		match provider_name {
			"openai" => Ok(Self::OpenAI { api_key: api_key.to_string() }),
			"anthropic" => Ok(Self::Anthropic { api_key: api_key.to_string() }),
			"groq" | "groqcloud" => Ok(Self::Groq { api_key: api_key.to_string() }),
			"mistral" | "mistralai" => Ok(Self::Mistral { api_key: api_key.to_string() }),
			"ollama" => Ok(Self::Ollama { base_url: None }),
			_ => Err(anyhow!("Unsupported provider: {}", provider_name)),
		}
	}

	/// Create a custom provider with a specified API scheme and base URL
	pub fn new_custom(api_key: &str, base_url: &str, api_scheme: &str) -> Result<Self> {
		let scheme = ApiScheme::from_str(api_scheme)
			.ok_or_else(|| anyhow!("Unsupported API scheme: {}", api_scheme))?;
		Ok(Self::Custom {
			api_key: api_key.to_string(),
			base_url: base_url.to_string(),
			api_scheme: scheme,
		})
	}

	/// Create a provider from ProviderData, supporting both built-in and custom providers
	pub fn from_provider_data(provider_name: &str, api_key: &str, base_url: Option<&str>, api_scheme: Option<&str>) -> Result<Self> {
		// If base_url is provided, treat as custom provider
		if let Some(url) = base_url {
			if !url.is_empty() {
				let scheme = api_scheme.unwrap_or("openai");
				return Self::new_custom(api_key, url, scheme);
			}
		}
		// Otherwise use built-in provider matching
		Self::new(provider_name, api_key)
	}

	/// Get the provider name as a string
	pub fn provider_name(&self) -> &str {
		match self {
			Provider::OpenAI { .. } => "openai",
			Provider::Anthropic { .. } => "anthropic",
			Provider::Groq { .. } => "groq",
			Provider::Mistral { .. } => "mistral",
			Provider::Ollama { .. } => "ollama",
			Provider::Custom { .. } => "custom",
		}
	}

	/// Send a message to the LLM provider and get a response
	pub async fn send_message(&self, messages: &MessageHistory, model: &str, config: &LLMConfig) -> Result<String> {
		// Build the LLM client based on provider type
		let llm = self.build_llm(model, config)?;

		// Convert MessageHistory to ChatMessage format
		let chat_messages: Vec<ChatMessage> = messages
			.iter()
			.map(|msg| match msg.role.as_str() {
				"user" => ChatMessage::user().content(&msg.content).build(),
				"assistant" => ChatMessage::assistant().content(&msg.content).build(),
				// For system messages, we'll use user with a prefix since system() may not exist
				"system" => ChatMessage::user().content(&format!("[System]: {}", &msg.content)).build(),
				_ => ChatMessage::user().content(&msg.content).build(),
			})
			.collect();

		// Send the chat request
		let response = llm.chat(&chat_messages).await.map_err(|e| anyhow!("{}", e))?;

		// Extract the text from the response
		response.text().map(|s| s.to_string()).ok_or_else(|| anyhow!("No response text from LLM"))
	}

	/// Build the LLM client with the appropriate backend and configuration
	fn build_llm(&self, model: &str, config: &LLMConfig) -> Result<Box<dyn LLMProvider>> {
		let mut builder = LLMBuilder::new();

		// Configure backend and API key
		builder = match self {
			Provider::OpenAI { api_key } => builder.backend(LLMBackend::OpenAI).api_key(api_key),
			Provider::Anthropic { api_key } => builder.backend(LLMBackend::Anthropic).api_key(api_key),
			Provider::Groq { api_key } => builder.backend(LLMBackend::Groq).api_key(api_key),
			Provider::Mistral { api_key } => builder.backend(LLMBackend::Mistral).api_key(api_key),
			Provider::Ollama { base_url } => {
				let b = builder.backend(LLMBackend::Ollama);
				if let Some(url) = base_url {
					b.base_url(url)
				} else {
					b
				}
			}
			Provider::Custom { api_key, base_url, api_scheme } => {
				builder.backend(api_scheme.to_backend()).api_key(api_key).base_url(base_url)
			}
		};

		// Apply common configuration
		builder = builder.model(model).temperature(config.temperature).max_tokens(config.max_tokens);

		if let Some(top_p) = config.top_p {
			builder = builder.top_p(top_p);
		}

		builder.build().map_err(|e| anyhow!("{}", e))
	}
}
