use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{AhabError, Result};

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: Message,
}

impl OpenRouterClient {
    pub fn new(api_key: String, model: String) -> Self {
        let client = Client::builder()
            .user_agent("ahab/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        OpenRouterClient {
            client,
            api_key,
            model,
        }
    }

    pub async fn breakdown_to_epics(&self, content: &str) -> Result<String> {
        let system_prompt = r#"You are a technical project manager breaking down documentation into actionable epics for an engineering team.

Your task is to analyze the provided documentation and create detailed epics. Each epic should:

1. Have a clear, specific title
2. Include a comprehensive description with technical detail
3. List specific acceptance criteria that define done
4. Provide technical notes about implementation approach, dependencies, and architecture considerations
5. Be tagged appropriately for organization
6. Include relevant labels (e.g., backend, frontend, infrastructure, etc.)

Focus on technical detail. The use cases in the documentation indicate intent, but the epics should be authoritative on HOW features are completed technically. Multiple features may be grouped into a single epic if they are closely related.

Format each epic as markdown with the following structure:

# [Epic Title]

## Description

[Detailed description of what needs to be built]

## Acceptance Criteria

- [Specific criterion 1]
- [Specific criterion 2]
- [etc.]

## Technical Notes

[Implementation approach, architecture decisions, dependencies, technical considerations]

## Tags

- [tag1]
- [tag2]

## Labels

- [label1]
- [label2]

---

Separate each epic with a line of three dashes (---).
"#;

        let user_prompt = format!(
            "Please break down the following documentation into detailed technical epics:\n\n{}",
            content
        );

        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
        };

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AhabError::OpenRouterApi(format!(
                "API request failed: {} - {}",
                status, body
            )));
        }

        let api_response: OpenRouterResponse = response.json().await?;

        if api_response.choices.is_empty() {
            return Err(AhabError::OpenRouterApi("No response from API".to_string()));
        }

        Ok(api_response.choices[0].message.content.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_client_creation() {
        let client = OpenRouterClient::new(
            "test_key".to_string(),
            "anthropic/claude-sonnet-4".to_string(),
        );
        assert_eq!(client.model, "anthropic/claude-sonnet-4");
    }
}
