use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::llm_providers::{LLMConfig, LLMMessage};
use crate::types::MessageHistory;

/// OpenAIProvider is used as the default implementation of "LLMProvider".
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
	pub message: LLMMessage,
	logprobs: Option<serde_json::Value>,
	finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Usage {
	prompt_tokens: u32,
	completion_tokens: u32,
	total_tokens: u32,
}

impl OpenAIProvider {
	pub fn new(api_key: &str, url: &str) -> Self {
		Self {
			api_key: api_key.to_string(),
			url: url.to_string(),
		}
	}

	pub async fn send_message(&self, messages: &MessageHistory, model: &str, config: &LLMConfig) -> Result<String> {
		let client = Client::new();

		let openai_messages: Vec<LLMMessage> = messages
			.iter()
			.map(|msg| LLMMessage {
				role: msg.role.to_string(),
				content: msg.content.to_string(),
			})
			.collect();

		let body = json!({
			"model": model,
			"messages": openai_messages,
			"temperature": config.temperature,
			"max_tokens": config.max_tokens
		});

		log::debug!("Sending message to OpenAI: {:?}", body);

		let response: Response = client
			.post(&self.url)
			.header("Content-Type", "application/json")
			.header("Authorization", format!("Bearer {}", &self.api_key))
			.json(&body)
			.send()
			.await
			.context("Failed to send message to OpenAI")?;

		let response_text = response.text().await.context("Failed to read response from OpenAI")?;

		if let Ok(parsed_response) = serde_json::from_str::<OpenAIChatCompletionResponse>(&response_text) {
			let answer = parsed_response.choices.get(0).ok_or(anyhow::anyhow!("No response"))?.message.content.clone();
			log::debug!("Answer: {}", answer);
			return Ok(answer);
		}

		Err(anyhow::anyhow!("Something went wrong when sending message to OpenAI: {}", response_text))
	}
}
