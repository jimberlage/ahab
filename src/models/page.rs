use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub reference_num: String,
    pub name: String,
    pub body: Option<String>,
    pub html_body: Option<String>,
    pub url: Option<String>,
    pub product_id: Option<String>,
    pub parent_id: Option<String>,
}

impl Page {
    pub fn new(id: String, reference_num: String, name: String) -> Self {
        Page {
            id,
            reference_num,
            name,
            body: None,
            html_body: None,
            url: None,
            product_id: None,
            parent_id: None,
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

    pub fn with_product_id(mut self, product_id: Option<String>) -> Self {
        self.product_id = product_id;
        self
    }

    pub fn with_parent_id(mut self, parent_id: Option<String>) -> Self {
        self.parent_id = parent_id;
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
