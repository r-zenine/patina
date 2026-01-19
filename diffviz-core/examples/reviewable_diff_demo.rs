//! Complete demonstration of the ReviewableDiff pipeline
//!
//! This example shows how to:
//! 1. Parse source code into AST trees
//! 2. Detect changes using multiple strategies  
//! 3. Expand changes with semantic context
//! 4. Convert to structured ReviewableDiff objects
//! 5. Display results with debug formatting
//!
//! Run with: cargo run --example reviewable_diff_demo

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::{
    ast_diff::SourceCode, parsers::RustParser, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Original version - Basic HTTP client with synchronous operations
const OLD_CODE: &str = r#"use std::collections::HashMap;
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

impl std::error::Error for HttpError {}

#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            base_url: "http://localhost:8080".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
        }
    }
}

pub struct HttpClient {
    config: HttpConfig,
    headers: HashMap<String, String>,
}

impl HttpClient {
    pub fn new(config: HttpConfig) -> Self {
        Self {
            config,
            headers: HashMap::new(),
        }
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn get(&self, path: &str) -> Result<String, HttpError> {
        let url = format!("{}/{}", self.config.base_url, path);
        self.make_request("GET", &url, None)
    }

    pub fn post(&self, path: &str, body: Option<String>) -> Result<String, HttpError> {
        let url = format!("{}/{}", self.config.base_url, path);
        self.make_request("POST", &url, body)
    }

    fn make_request(&self, method: &str, url: &str, body: Option<String>) -> Result<String, HttpError> {
        println!("Making {} request to: {}", method, url);
        
        if url.contains("error") {
            return Err(HttpError::NetworkError("Connection failed".to_string()));
        }
        
        let response = match body {
            Some(data) => format!("Response for {} with body: {}", method, data),
            None => format!("Response for {}", method),
        };
        
        Ok(response)
    }
}
"#;

/// Refactored version - Async HTTP client with improved error handling and features  
const NEW_CODE: &str = r#"use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Comprehensive HTTP error types with detailed context
#[derive(Debug, Clone)]
pub enum HttpError {
    NetworkError { message: String, status_code: Option<u16> },
    ParseError { message: String, line: Option<usize> },
    TimeoutError { duration: Duration },
    ConfigError { field: String, value: String },
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::NetworkError { message, status_code } => {
                match status_code {
                    Some(code) => write!(f, "Network error ({}): {}", code, message),
                    None => write!(f, "Network error: {}", message),
                }
            },
            HttpError::ParseError { message, line } => {
                match line {
                    Some(l) => write!(f, "Parse error at line {}: {}", l, message),
                    None => write!(f, "Parse error: {}", message),
                }
            },
            HttpError::TimeoutError { duration } => {
                write!(f, "Request timed out after {:?}", duration)
            },
            HttpError::ConfigError { field, value } => {
                write!(f, "Invalid config for {}: {}", field, value)
            },
        }
    }
}

impl std::error::Error for HttpError {}

/// Enhanced configuration with validation
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub user_agent: String,
    pub follow_redirects: bool,
}

impl HttpConfig {
    /// Create a new configuration with validation
    pub fn new(base_url: String, timeout_ms: u64) -> Result<Self, HttpError> {
        if base_url.is_empty() {
            return Err(HttpError::ConfigError { 
                field: "base_url".to_string(), 
                value: "empty".to_string() 
            });
        }
        
        Ok(HttpConfig {
            base_url,
            timeout_ms,
            max_retries: 3,
            user_agent: "RustHttpClient/1.0".to_string(),
            follow_redirects: true,
        })
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            base_url: "https://api.example.com".to_string(),
            timeout_ms: 10000,
            max_retries: 5,
            user_agent: "RustHttpClient/1.0".to_string(),
            follow_redirects: true,
        }
    }
}

/// Async HTTP client with comprehensive features
#[derive(Debug)]
pub struct HttpClient {
    config: HttpConfig,
    headers: HashMap<String, String>,
    request_count: u64,
}

impl HttpClient {
    /// Create a new HTTP client with configuration
    pub fn new(config: HttpConfig) -> Self {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), config.user_agent.clone());
        
        Self {
            config,
            headers,
            request_count: 0,
        }
    }

    /// Add a custom header to all requests
    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    /// Remove a header from all requests
    pub fn remove_header(&mut self, key: &str) {
        self.headers.remove(key);
    }

    /// Get current request count for monitoring
    pub fn get_request_count(&self) -> u64 {
        self.request_count
    }

    /// Async GET request with improved error handling
    pub async fn get(&mut self, path: &str) -> Result<String, HttpError> {
        let url = format!("{}/{}", self.config.base_url, path.trim_start_matches('/'));
        self.make_request("GET", &url, None).await
    }

    /// Async POST request with JSON body support
    pub async fn post(&mut self, path: &str, body: Option<String>) -> Result<String, HttpError> {
        let url = format!("{}/{}", self.config.base_url, path.trim_start_matches('/'));
        self.make_request("POST", &url, body).await
    }

    /// Async PUT request for updates
    pub async fn put(&mut self, path: &str, body: String) -> Result<String, HttpError> {
        let url = format!("{}/{}", self.config.base_url, path.trim_start_matches('/'));
        self.make_request("PUT", &url, Some(body)).await
    }

    /// Internal async request handler with retry logic
    async fn make_request(&mut self, method: &str, url: &str, body: Option<String>) -> Result<String, HttpError> {
        self.request_count += 1;
        
        println!("Making async {} request #{} to: {}", method, self.request_count, url);
        
        // Simulate timeout scenarios
        if url.contains("timeout") {
            return Err(HttpError::TimeoutError { 
                duration: Duration::from_millis(self.config.timeout_ms) 
            });
        }
        
        // Simulate network errors with status codes
        if url.contains("error") {
            return Err(HttpError::NetworkError { 
                message: "Connection refused".to_string(),
                status_code: Some(503),
            });
        }
        
        // Simulate successful response
        let response = match body {
            Some(data) => format!("Async response for {} with body: {} (headers: {:?})", 
                               method, data, self.headers.len()),
            None => format!("Async response for {} (headers: {:?})", 
                          method, self.headers.len()),
        };
        
        Ok(response)
    }
}
"#;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Semantic Code Review Demo");
    println!();

    // Configuration
    let language = ProgrammingLanguage::Rust;

    // Setup source code objects
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);

    // Parse AST trees
    let rust_parser = RustParser::new();
    let mut ts_parser = Parser::new();
    ts_parser.set_language(rust_parser.get_language())?;

    let old_tree = ts_parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old code")?;
    let new_tree = ts_parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new code")?;

    let old_semantic_tree = rust_parser
        .build_semantic_tree(&old_tree, OLD_CODE)
        .map_err(|e| format!("Failed to build old semantic tree: {e}"))?;
    let new_semantic_tree = rust_parser
        .build_semantic_tree(&new_tree, NEW_CODE)
        .map_err(|e| format!("Failed to build new semantic tree: {e}"))?;

    // Build semantic pairs and convert to reviewable diffs
    let semantic_pairs = build_semantic_pairs(
        &old_semantic_tree,
        &new_semantic_tree,
        &old_source,
        &new_source,
        &rust_parser,
    )
    .map_err(|e| format!("Failed to build semantic pairs: {e}"))?;

    let semantic_reviewable_diffs =
        semantic_pairs_to_reviewable_diffs(&semantic_pairs, language, &old_source, &new_source);

    // First pass: filter out changes with no visible content
    let visible_changes: Vec<&_> = semantic_reviewable_diffs
        .iter()
        .filter(|reviewable_diff| {
            let renderable: RenderableDiff = (*reviewable_diff).into();
            let visible_lines = renderable
                .lines
                .iter()
                .filter(|line| !line.should_fold())
                .count();
            visible_lines > 0
        })
        .collect();

    for (i, reviewable_diff) in visible_changes.iter().enumerate() {
        // Convert to RenderableDiff for display
        let renderable: RenderableDiff = (*reviewable_diff).into();

        let changed_lines = renderable
            .lines
            .iter()
            .filter(|line| line.has_changes())
            .count();
        let hidden_lines = renderable
            .lines
            .iter()
            .filter(|line| line.should_fold())
            .count();

        println!(
            "\n🔸 Change {} of {}: {}",
            i + 1,
            visible_changes.len(),
            renderable.metadata.boundary_name
        );

        if changed_lines > 0 {
            println!("   📊 {changed_lines} changed lines, {hidden_lines} hidden lines");
        }

        // Display source code with syntax highlighting

        println!("   📝 Source Code:");

        let mut hidden_count = 0;
        for line in &renderable.lines {
            if line.should_fold() {
                hidden_count += 1;
                continue;
            }

            if hidden_count > 0 {
                println!("   \x1b[37m  ... {hidden_count} lines hidden ...\x1b[0m");
                hidden_count = 0;
            }

            let (prefix, color) = line.get_display_style();
            println!("   {}{} {}\x1b[0m", color, prefix, line.content);
        }

        if hidden_count > 0 {
            println!("   \x1b[37m  ... {hidden_count} lines hidden ...\x1b[0m");
        }
    }

    println!(
        "\n✅ Review complete: {} changes detected",
        visible_changes.len()
    );

    Ok(())
}
