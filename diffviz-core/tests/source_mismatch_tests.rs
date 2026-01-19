//! Tests for source/AST node mismatch scenarios
//!
//! This module tests scenarios where AST nodes from one source version
//! are incorrectly used with a different source version, causing index
//! out of bounds errors in tree-sitter.

use diffviz_core::{
    ast_diff::{ChangeDetectionStrategies, SourceCode, diff_ast_trees_with_strategies},
    common::ProgrammingLanguage,
    renderable_diff::RenderableDiff,
    reviewable_diff::expand_changes_to_reviewable_diffs,
};
use tree_sitter::Parser;

mod test_utils;
use test_utils::get_parser_for_language;

#[test]
fn test_large_to_small_source_mismatch() {
    // This test reproduces the exact scenario that caused the panic:
    // - Large old source (like HEAD commit)
    // - Small new source (like heavily modified working directory)
    // - Full pipeline: AST diff → ReviewableDiff → RenderableDiff

    // Large old source (simulating a HEAD commit with lots of content)
    let old_code = r#"
use std::collections::HashMap;
use std::fmt;
use std::error::Error as StdError;
use std::io;

/// Documentation for HttpError
#[derive(Debug, Clone, PartialEq)]
pub enum HttpError {
    NetworkError(String),
    ParseError(String),
    TimeoutError(u64),
    AuthError { code: u32, message: String },
    ValidationError { field: String, reason: String },
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            HttpError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            HttpError::TimeoutError(duration) => write!(f, "Timeout after {} ms", duration),
            HttpError::AuthError { code, message } => write!(f, "Auth error {}: {}", code, message),
            HttpError::ValidationError { field, reason } => write!(f, "Validation error in {}: {}", field, reason),
        }
    }
}

impl StdError for HttpError {}

/// Configuration for HTTP client
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub user_agent: String,
    pub default_headers: HashMap<String, String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        HttpConfig {
            base_url: "http://localhost:8080".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
            user_agent: "DiffViz/1.0".to_string(),
            default_headers: headers,
        }
    }
}

/// HTTP client for making requests
pub struct HttpClient {
    config: HttpConfig,
    headers: HashMap<String, String>,
    retry_count: u32,
}

impl HttpClient {
    /// Create a new HTTP client with configuration
    pub fn new(config: HttpConfig) -> Self {
        Self {
            config,
            headers: HashMap::new(),
            retry_count: 0,
        }
    }

    /// Add a header to the client
    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    /// Remove a header from the client
    pub fn remove_header(&mut self, key: &str) {
        self.headers.remove(key);
    }

    /// Get the current configuration
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Result<String, HttpError> {
        self.make_request("GET", path, None).await
    }

    /// Make a POST request with body
    pub async fn post(&self, path: &str, body: Option<&str>) -> Result<String, HttpError> {
        self.make_request("POST", path, body).await
    }

    /// Make a PUT request with body
    pub async fn put(&self, path: &str, body: Option<&str>) -> Result<String, HttpError> {
        self.make_request("PUT", path, body).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> Result<String, HttpError> {
        self.make_request("DELETE", path, None).await
    }

    /// Internal method to make HTTP requests with retry logic
    async fn make_request(&self, method: &str, path: &str, body: Option<&str>) -> Result<String, HttpError> {
        let url = format!("{}{}", self.config.base_url, path);
        
        for attempt in 0..=self.config.max_retries {
            match self.execute_request(method, &url, body).await {
                Ok(response) => return Ok(response),
                Err(e) if attempt < self.config.max_retries => {
                    eprintln!("Request attempt {} failed: {}", attempt + 1, e);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        
        unreachable!("Should have returned from loop")
    }

    /// Execute a single HTTP request attempt
    async fn execute_request(&self, method: &str, url: &str, body: Option<&str>) -> Result<String, HttpError> {
        // Simulate HTTP request logic
        if url.contains("error") {
            return Err(HttpError::NetworkError("Simulated network error".to_string()));
        }
        
        if method == "POST" && body.is_none() {
            return Err(HttpError::ValidationError {
                field: "body".to_string(),
                reason: "POST requires a body".to_string(),
            });
        }
        
        // Simulate successful response
        Ok(format!("{} request to {} successful", method, url))
    }
}

/// Helper function to create a default client
pub fn create_default_client() -> HttpClient {
    HttpClient::new(HttpConfig::default())
}

/// Helper function to create a client with custom timeout
pub fn create_client_with_timeout(timeout_ms: u64) -> HttpClient {
    let mut config = HttpConfig::default();
    config.timeout_ms = timeout_ms;
    HttpClient::new(config)
}

/// Utility function for validating URLs
pub fn validate_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Utility function for building query strings
pub fn build_query_string(params: &HashMap<String, String>) -> String {
    if params.is_empty() {
        return String::new();
    }
    
    let pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    
    format!("?{}", pairs.join("&"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_client_creation() {
        let client = create_default_client();
        assert_eq!(client.config().timeout_ms, 5000);
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_url("http://example.com"));
        assert!(validate_url("https://example.com"));
        assert!(!validate_url("ftp://example.com"));
        assert!(!validate_url("example.com"));
    }

    #[test]
    fn test_query_string_building() {
        let mut params = HashMap::new();
        params.insert("key1".to_string(), "value1".to_string());
        params.insert("key2".to_string(), "value2".to_string());
        
        let query = build_query_string(&params);
        assert!(query.starts_with("?"));
        assert!(query.contains("key1=value1"));
        assert!(query.contains("key2=value2"));
    }
}
"#;

    // Small new source (simulating heavily modified working directory)
    // Most content deleted, only a small portion remains
    let new_code = r#"
use std::fmt;

#[derive(Debug)]
pub enum HttpError {
    NetworkError(String),
    ParseError(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            HttpError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}
"#;

    println!("Old source length: {} chars", old_code.len());
    println!("New source length: {} chars", new_code.len());

    // Set up parser
    let parser_impl = get_parser_for_language(ProgrammingLanguage::Rust);
    let mut ts_parser = Parser::new();
    ts_parser.set_language(parser_impl.get_language()).unwrap();

    // Parse both versions
    let old_tree = ts_parser.parse(old_code, None).unwrap();
    let new_tree = ts_parser.parse(new_code, None).unwrap();

    // Create source objects
    let old_source = SourceCode::new(old_code);
    let new_source = SourceCode::new(new_code);

    // Detect changes
    let strategies = ChangeDetectionStrategies::default_strategies();
    let ast_diff =
        diff_ast_trees_with_strategies(&old_tree, &new_tree, old_code, new_code, strategies);

    println!("Detected {} changes", ast_diff.changes.len());

    // This should not panic!
    // Convert to reviewable diffs - this is where the panic occurs
    let reviewable_diffs = expand_changes_to_reviewable_diffs(
        &ast_diff.changes,
        parser_impl.as_ref(),
        &old_source,
        &new_source,
        ProgrammingLanguage::Rust,
    );

    println!("Generated {} reviewable diffs", reviewable_diffs.len());

    // Convert to renderable diffs - this triggers the panic in extract_boundary_source
    for (i, reviewable_diff) in reviewable_diffs.iter().enumerate() {
        println!("Processing reviewable diff {}", i + 1);

        // This line should not panic with "index out of bounds"
        let _renderable: RenderableDiff = reviewable_diff.into();

        println!(
            "Successfully converted reviewable diff {} to renderable diff",
            i + 1
        );
    }

    // If we get here without panicking, the fix worked!
    println!("Test completed successfully - no index out of bounds panic!");
}
