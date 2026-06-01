use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{AhabError, Result};
use crate::models::{Epic, Page};

pub struct AhaClient {
    client: Client,
    token: String,
    pub(crate) domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AhaPage {
    pub id: String,
    pub reference_num: String,
    pub name: String,
    pub body: Option<String>,
    pub html_body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AhaEpic {
    pub id: Option<String>,
    pub reference_num: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AhaEpicResponse {
    pub epic: AhaEpic,
}

#[derive(Debug, Serialize, Deserialize)]
struct AhaComment {
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AhaCommentRequest {
    pub comment: AhaComment,
}

#[derive(Debug, Serialize, Deserialize)]
struct AhaCommentResponse {
    pub comment: AhaComment,
}

impl AhaClient {
    pub fn new(token: String, domain: String) -> Self {
        let client = Client::builder()
            .user_agent("ahab/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        AhaClient {
            client,
            token,
            domain,
        }
    }

    fn base_url(&self) -> String {
        // If domain starts with http:// or https://, use it as-is (for testing)
        if self.domain.starts_with("http://") || self.domain.starts_with("https://") {
            format!("{}/api/v1", self.domain)
        } else {
            format!("https://{}/api/v1", self.domain)
        }
    }

    pub async fn get_page(&self, page_id: &str) -> Result<Page> {
        let url = format!("{}/pages/{}", self.base_url(), page_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AhabError::AhaApi(format!(
                "Failed to fetch page {}: {} - {}",
                page_id, status, body
            )));
        }

        let aha_page: AhaPage = response.json().await?;

        Ok(Page::new(aha_page.id.clone(), aha_page.name.clone())
            .with_body(aha_page.body.unwrap_or_default())
            .with_html_body(aha_page.html_body.unwrap_or_default())
            .with_url(format!("https://{}/pages/{}", self.domain, aha_page.reference_num)))
    }

    pub async fn create_epic(&self, product_id: &str, epic: &Epic) -> Result<String> {
        let url = format!("{}/products/{}/epics", self.base_url(), product_id);

        let mut body_parts = vec![epic.description.clone()];

        if let Some(criteria) = &epic.acceptance_criteria {
            if !criteria.is_empty() {
                body_parts.push("\n\n## Acceptance Criteria\n".to_string());
                for criterion in criteria {
                    body_parts.push(format!("- {}\n", criterion));
                }
            }
        }

        if let Some(notes) = &epic.technical_notes {
            if !notes.trim().is_empty() {
                body_parts.push(format!("\n\n## Technical Notes\n\n{}", notes));
            }
        }

        let full_description = body_parts.join("");

        let payload = json!({
            "epic": {
                "name": epic.title,
                "description": full_description,
                "tags": epic.tags.as_ref().unwrap_or(&vec![]),
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AhabError::AhaApi(format!(
                "Failed to create epic: {} - {}",
                status, body
            )));
        }

        let epic_response: AhaEpicResponse = response.json().await?;
        let epic_ref = epic_response
            .epic
            .reference_num
            .unwrap_or_else(|| epic_response.epic.id.unwrap_or_default());

        Ok(format!("https://{}/epics/{}", self.domain, epic_ref))
    }

    pub async fn add_comment_to_page(&self, page_id: &str, comment: &str) -> Result<()> {
        let url = format!("{}/pages/{}/comments", self.base_url(), page_id);

        let request = AhaCommentRequest {
            comment: AhaComment {
                body: comment.to_string(),
            },
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AhabError::AhaApi(format!(
                "Failed to add comment: {} - {}",
                status, body
            )));
        }

        // Deserialize response to ensure it's valid
        let _comment_response: AhaCommentResponse = response.json().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_get_page() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/pages/PAGE-1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": "123",
                "reference_num": "PAGE-1",
                "name": "Test Page",
                "body": "Test body",
                "html_body": "<p>Test body</p>"
            }"#,
            )
            .create_async()
            .await;

        // Use the mockito server URL with http://
        let domain = server.url();
        let client = AhaClient::new("test_token".to_string(), domain);

        let page = client.get_page("PAGE-1").await.unwrap();
        assert_eq!(page.name, "Test Page");

        mock.assert_async().await;
    }
}
