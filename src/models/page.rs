use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub name: String,
    pub body: Option<String>,
    pub html_body: Option<String>,
    pub url: Option<String>,
}

impl Page {
    pub fn new(id: String, name: String) -> Self {
        Page {
            id,
            name,
            body: None,
            html_body: None,
            url: None,
        }
    }

    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_html_body(mut self, html_body: String) -> Self {
        self.html_body = Some(html_body);
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    /// Convert HTML body to markdown
    pub fn to_markdown(&self) -> String {
        if let Some(html) = &self.html_body {
            html2md::parse_html(html)
        } else if let Some(body) = &self.body {
            body.clone()
        } else {
            String::new()
        }
    }
}
