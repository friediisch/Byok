use anyhow::Result;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::{language_models::options::CallOptions, llm::Claude};
use serde::{Deserialize, Serialize};

use crate::types::MessageHistory;

use super::{LLMConfig, LLMProvider};

#[derive(Clone)]
pub struct AnthropicProvider {
	api_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicChatResponse {
	content: Vec<MessageContent>,
	id: String,
	model: String,
	role: String,
	stop_reason: String,
	stop_sequence: Option<String>,
	#[serde(rename = "type")]
	message_type: String,
	usage: AnthropicUsage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageContent {
	text: String,
	#[serde(rename = "type")]
	content_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AnthropicUsage {
	input_tokens: i32,
	output_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicErrorResponse {
	#[serde(rename = "type")]
	error_type: String,
	error: AnthropicErrorDetails,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicErrorDetails {
	#[serde(rename = "type")]
	error_type: String,
	message: String,
}

impl AnthropicProvider {
	pub fn new(api_key: &str) -> Self {
		Self { api_key: api_key.to_string() }
	}
}

impl LLMProvider for AnthropicProvider {
	fn provider_name(&self) -> &str {
		"anthropic"
	}

	async fn send_message(&self, messages: &MessageHistory, model: &str, config: &LLMConfig) -> Result<String> {
		let body = serde_json::json!({
			"messages": messages,
			"model": model,
			"max_tokens": config.max_tokens,
			"temperature": config.temperature,
		});

		let call_options = CallOptions {
			candidate_count: None,
			max_tokens: body["max_tokens"].as_u64().map(|v| v as u32),
			temperature: body["temperature"].as_f64().map(|v| v as f32),
			stop_words: None,
			streaming_func: None,
			top_k: None,
			top_p: None,
			seed: None,
			min_length: None,
			max_length: None,
			n: None,
			repetition_penalty: None,
			frequency_penalty: None,
			presence_penalty: None,
			functions: None,
			function_call_behavior: None,
			stream_usage: None,
		};

		println!("body: {:?}", body);

		let claude = Claude::default().with_api_key(&self.api_key).with_options(call_options);
		let messages = body["messages"].as_array().ok_or_else(|| anyhow::anyhow!("Missing 'messages' array"))?;
		let history: String = messages
			.iter()
			.map(|msg| format!("{}: {}", msg["role"].as_str().unwrap_or(""), msg["content"].as_str().unwrap_or("")))
			.collect::<Vec<String>>()
			.join("\n");

		match claude.invoke(&history).await {
			Ok(response) => Ok(response),
			Err(e) => {
				if e.to_string().to_lowercase().contains("authentication") {
					eprintln!("Authentication error with Claude: {}", e);
					Err(anyhow::anyhow!("Invalid API key or authentication failed"))
				} else {
					eprintln!("Error invoking Claude: {}", e);
					Err(anyhow::anyhow!("An error occurred: {}", e))
				}
			}
		}
	}
}
