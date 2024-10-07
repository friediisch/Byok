use anyhow;
use anyhow_tauri::{IntoTAResult, TAResult as Result};
use langchain_rust::language_models::llm::LLM;
use langchain_rust::{language_models::options::CallOptions, llm::Claude};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

pub async fn send_anthropic_message(body: Value, api_key: &str) -> Result<String> {
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
	};

	let claude = Claude::default().with_api_key(api_key).with_options(call_options);
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
				anyhow::anyhow!("Invalid API key or authentication failed").into_ta_result()
			} else {
				eprintln!("Error invoking Claude: {}", e);
				anyhow::anyhow!("An error occurred: {}", e).into_ta_result()
			}
		}
	}
}
