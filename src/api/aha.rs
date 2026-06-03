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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhaDescription {
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildPage {
    pub reference_num: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhaPage {
    pub id: String,
    pub reference_num: String,
    pub name: String,
    pub body: Option<String>,
    pub html_body: Option<String>,
    pub description: Option<AhaDescription>,
    pub child_pages: Option<Vec<ChildPage>>,
    pub parent_id: Option<String>,
    pub product_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AhaPageResponse {
    pub page: AhaPage,
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
        // Try without fields parameter first, or with different parameter
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

        let response_text = response.text().await?;

        // Try to deserialize as wrapped response first
        if let Ok(wrapped) = serde_json::from_str::<AhaPageResponse>(&response_text) {
            let aha_page = wrapped.page;

            // Extract body from description.body, fallback to body, then html_body
            let body_content = aha_page
                .description
                .as_ref()
                .and_then(|d| d.body.clone())
                .or_else(|| aha_page.body.clone())
            .or_else(|| aha_page.html_body.clone());

        return Ok(Page::new(
            aha_page.id.clone(),
            aha_page.reference_num.clone(),
            aha_page.name.clone(),
        )
        .with_body(body_content.clone().unwrap_or_default())
        .with_html_body(body_content.unwrap_or_default())
        .with_url(format!(
            "https://{}/pages/{}",
            self.domain, aha_page.reference_num
        ))
        .with_product_id(aha_page.product_id.clone())
        .with_parent_id(aha_page.parent_id.clone()));
        }

        // Fall back to unwrapped response
        let aha_page: AhaPage = serde_json::from_str(&response_text)?;
        let body_content = aha_page
            .description
            .as_ref()
            .and_then(|d| d.body.clone())
            .or_else(|| aha_page.body.clone())
            .or_else(|| aha_page.html_body.clone());

        Ok(Page::new(
            aha_page.id.clone(),
            aha_page.reference_num.clone(),
            aha_page.name.clone(),
        )
        .with_body(body_content.clone().unwrap_or_default())
        .with_html_body(body_content.unwrap_or_default())
        .with_url(format!(
            "https://{}/pages/{}",
            self.domain, aha_page.reference_num
        ))
        .with_product_id(aha_page.product_id.clone())
        .with_parent_id(aha_page.parent_id.clone()))
    }

    /// Get children of a specific page from a pre-fetched list
    pub fn get_children_from_list(&self, all_pages: &[AhaPage], parent_page_id: &str) -> Vec<String> {
        all_pages
            .iter()
            .filter(|p| {
                p.parent_id.as_ref().map(|pid| pid == parent_page_id).unwrap_or(false)
            })
            .map(|p| p.reference_num.clone())
            .collect()
    }

    /// Fetch all pages for a product, handling pagination
    pub async fn fetch_all_pages(&self, product_id: &str) -> Result<Vec<AhaPage>> {
        let mut all_pages = Vec::new();
        let mut current_page = 1;
        
        loop {
            let url = format!(
                "{}/products/{}/pages?page={}",
                self.base_url(),
                product_id,
                current_page
            );

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
                    "Failed to list pages for product {}: {} - {}",
                    product_id, status, body
                )));
            }

            let response_text = response.text().await?;
            
            // Parse the response
            #[derive(Debug, Deserialize)]
            struct Pagination {
                total_pages: u32,
            }

            #[derive(Debug, Deserialize)]
            struct PagesResponse {
                pages: Vec<AhaPage>,
                pagination: Pagination,
            }

            let pages_response: PagesResponse = serde_json::from_str(&response_text)
                .map_err(|e| AhabError::AhaApi(format!("Failed to parse pages response: {}", e)))?;

            all_pages.extend(pages_response.pages);

            // Check if there are more pages
            if current_page >= pages_response.pagination.total_pages {
                break;
            }
            
            current_page += 1;
        }

        Ok(all_pages)
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
                "page": {
                    "id": "123",
                    "reference_num": "PAGE-1",
                    "name": "Test Page",
                    "body": "Test body",
                    "html_body": "<p>Test body</p>"
                }
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
