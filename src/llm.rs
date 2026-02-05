use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct DeepSeekClient {
    api_key: String,
    client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

impl DeepSeekClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(70))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are Teemah AI, a helpful Web3 assistant that helps users with cryptocurrency projects, investments, and blockchain transactions. Provide clear, accurate, and friendly responses.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];

        let request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages,
            temperature: 0.7,
            max_tokens: 500,
        };

        let response = self.client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await
                .map_err(|e| anyhow!("Failed to read error body: {}", e))?;
            return Err(anyhow!("API request failed with status {}: {}", status, body));
        }

        let parsed: ChatResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse JSON response: {}", e))?;
        
        parsed.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("No choices returned in API response"))
    }
}