//! HTTP response types and parsing.

use std::collections::HashMap;

/// Represents the body content of an HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseBody(pub String);

impl ResponseBody {
    /// Creates a new ResponseBody from a string.
    pub fn new(content: String) -> Self {
        ResponseBody(content)
    }

    /// Returns the content as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the ResponseBody and returns the inner String.
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Represents a complete HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub http_version: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<ResponseBody>,
}

impl HttpResponse {
    /// Creates a new HttpResponse.
    pub fn new(
        http_version: String,
        status_code: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<ResponseBody>,
    ) -> Self {
        HttpResponse {
            http_version,
            status_code,
            status_text,
            headers,
            body,
        }
    }
}