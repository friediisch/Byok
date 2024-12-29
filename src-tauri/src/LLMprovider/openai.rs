use anyhow::Result;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::llm_providers::LLMConfig;
use crate::types::MessageHistory;

#[derive(Clone)]
pub struct OpenAIProvider {
	api_key: String,
	url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIChatCompletionResponse {
	id: String,
	object: String,
	created: i64,
	model: String,
	pub choices: Vec<Choice>,
	usage: Usage,
	system_fingerprint: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
	index: i32,
	pub message: OpenAIMessage,
	logprobs: Option<serde_json::Value>,
	finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIMessage {
	role: String,
	content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Usage {
	prompt_tokens: i32,
	completion_tokens: i32,
	total_tokens: i32,
}

impl OpenAIProvider {
	pub fn new<T: Into<String>>(api_key: T) -> Self {
		Self {
			api_key: api_key.into(),
			url: "https://api.openai.com/v1/chat/completions".into(),
		}
	}

	pub async fn send_message(&self, messages: &MessageHistory, model: &str, config: &LLMConfig) -> Result<String> {
		let client = Client::new();

		let body = json!({
			"model": model,
			"messages": messages,
			"temperature": config.temperature,
			"max_tokens": config.max_tokens
		});

		let response: Response = client
			.post(&self.url)
			.header("Content-Type", "application/json")
			.header("Authorization", format!("Bearer {}", &self.api_key))
			.json(&body)
			.send()
			.await?;

		let response_text = response.text().await?;
		let parsed_response: OpenAIChatCompletionResponse = serde_json::from_str(&response_text)?;
		let answer = parsed_response.choices.get(0).ok_or(anyhow::anyhow!("No response"))?.message.content.clone();

		Ok(answer)
	}

	pub fn get_provider_name(&self) -> &'static str {
		"openai"
	}
}
