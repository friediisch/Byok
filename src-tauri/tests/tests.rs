// use byok::llm_providers::default::OpenAIProvider;

// #[cfg(test)]
// mod tests {
// 	use byok::{llm_providers::LLMConfig, types::MessageHistory};

// 	use super::*;

// 	#[tokio::test]
// 	async fn test_openai_provider() {
// 		let provider = OpenAIProvider::new("test", "https://api.openai.com/v1/chat/completions");
// 		let messages = MessageHistory(vec![]);
// 		let config = LLMConfig::default();
// 		let response = provider.send_message(&messages, "gpt-3.5-turbo", &config).await;
// 		assert!(response.is_err()); // Will error due to invalid API key, which is expected
// 	}
// }
