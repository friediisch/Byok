// use anyhow::anyhow;
// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use std::error::Error;

// #[derive(Serialize, Deserialize, Debug)]
// pub struct MistralChatCompletionResponse {
// 	id: String,
// 	object: String,
// 	created: i64,
// 	model: String,
// 	pub choices: Vec<MistralChoice>,
// 	usage: MistralUsage,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct MistralChoice {
// 	index: i32,
// 	pub message: MistralMessage,
// 	finish_reason: String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct MistralMessage {
// 	role: String,
// 	content: String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct MistralUsage {
// 	prompt_tokens: i32,
// 	completion_tokens: i32,
// 	total_tokens: i32,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct MistralErrorResponse {
// 	message: String,
// 	request_id: String,
// }

// pub async fn send_mistralai_message(body: Value, api_key: &str) -> Result<String, Box<dyn Error>> {
// 	let url = "https://api.mistral.ai/v1/chat/completions";
// 	let client = Client::new();
// 	match client
// 		.post(url)
// 		.header("Content-Type", "application/json")
// 		.header("Authorization", format!("Bearer {}", api_key))
// 		.json(&body)
// 		.send()
// 		.await
// 	{
// 		Ok(response) => {
// 			let response_text: String = response.text().await.map_err(|err| err.to_string())?;
// 			if let Ok(parsed_response) =
// 				serde_json::from_str::<MistralChatCompletionResponse>(&response_text)
// 			{
// 				let answer: String = parsed_response
// 					.choices
// 					.get(0)
// 					.ok_or("No response".to_string())?
// 					.message
// 					.content
// 					.clone();
// 				Ok(answer)
// 			} else if let Ok(error_response) =
// 				serde_json::from_str::<MistralErrorResponse>(&response_text)
// 			{
// 				eprintln!(
// 					"Error from Mistral API: {} (Request ID: {})",
// 					error_response.message, error_response.request_id
// 				);
// 				Err(anyhow!("Error from Mistral API: {}", error_response.message).into())
// 			} else {
// 				eprintln!("Unexpected response from Mistral API: {}", response_text);
// 				Err(anyhow!("Unexpected response from Mistral API").into())
// 			}
// 		}
// 		Err(err) => {
// 			eprintln!("Error sending message to Mistral: {}", err);
// 			Err(anyhow!(err).into())
// 		}
// 	}
// }
