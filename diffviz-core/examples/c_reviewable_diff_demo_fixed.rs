//! Complete C semantic code review demo
//!
//! This demonstrates the C parser's ability to detect and analyze
//! semantic changes in C code including functions, structs, enums,
//! and preprocessor directives.

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::{
    ast_diff::SourceCode, parsers::CParser, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Original version - Traditional C library with basic error handling
const OLD_CODE: &str = r#"#include <stdio.h>
#include <stdlib.h>
#include <curl/curl.h>

// Response structure
struct http_response {
    char *data;
    size_t size;
};

// Error codes
enum http_error {
    HTTP_OK = 0,
    HTTP_ERROR_INIT = -1,
    HTTP_ERROR_REQUEST = -2,
    HTTP_ERROR_MEMORY = -3
};

// Client configuration
struct http_client {
    CURL *curl;
    long timeout;
    char user_agent[64];
};

void http_client_free(struct http_client *client) {
    if (client) {
        if (client->curl) {
            curl_easy_cleanup(client->curl);
        }
        free(client);
    }
}

static size_t write_callback(void *contents, size_t size, size_t nmemb, struct http_response *response) {
    size_t total_size = size * nmemb;
    
    response->data = realloc(response->data, response->size + total_size + 1);
    if (!response->data) {
        return 0;
    }
    
    memcpy(response->data + response->size, contents, total_size);
    response->size += total_size;
    response->data[response->size] = '\0';
    
    return total_size;
}

int http_client_get(struct http_client *client, const char *url, struct http_response *response) {
    if (!client || !url || !response) {
        return HTTP_ERROR_REQUEST;
    }
    
    response->data = NULL;
    response->size = 0;
    
    curl_easy_setopt(client->curl, CURLOPT_URL, url);
    curl_easy_setopt(client->curl, CURLOPT_WRITEFUNCTION, write_callback);
    curl_easy_setopt(client->curl, CURLOPT_WRITEDATA, response);
    curl_easy_setopt(client->curl, CURLOPT_TIMEOUT, client->timeout);
    curl_easy_setopt(client->curl, CURLOPT_USERAGENT, client->user_agent);
    
    CURLcode res = curl_easy_perform(client->curl);
    if (res != CURLE_OK) {
        http_response_free(response);
        return HTTP_ERROR_REQUEST;
    }
    
    return HTTP_OK;
}

void http_response_free(struct http_response *response) {
    if (response && response->data) {
        free(response->data);
        response->data = NULL;
        response->size = 0;
    }
}"#;

/// Enhanced version - Modern C library with comprehensive error handling
const NEW_CODE: &str = r#"#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <curl/curl.h>
#include <pthread.h>

// Enhanced response structure with metadata
struct http_response {
    char *data;
    size_t size;
    size_t capacity;
    long status_code;
    double total_time;
    char content_type[64];
    bool is_compressed;
    uint64_t timestamp;
};

// Enhanced error codes with more specificity
enum http_error {
    HTTP_OK = 0,
    HTTP_ERROR_INIT = -1,
    HTTP_ERROR_REQUEST = -2,
    HTTP_ERROR_MEMORY = -3,
    HTTP_ERROR_TIMEOUT = -4,
    HTTP_ERROR_SSL = -5,
    HTTP_ERROR_REDIRECT = -6,
    HTTP_ERROR_THREAD = -7,
    HTTP_ERROR_INVALID_URL = -8,
    HTTP_ERROR_NETWORK = -9
};

// Thread-safe client configuration with advanced features
struct http_client {
    CURL *curl;
    struct http_client_config config;
    struct http_header headers[MAX_HEADERS];
    size_t header_count;
    pthread_mutex_t mutex;
    bool initialized;
    uint32_t request_count;
    double total_time;
};

void http_client_free(struct http_client *client) {
    if (UNLIKELY(!client)) {
        return;
    }
    
    pthread_mutex_lock(&client->mutex);
    
    if (client->curl) {
        curl_easy_cleanup(client->curl);
        client->curl = NULL;
    }
    
    client->initialized = false;
    
    pthread_mutex_unlock(&client->mutex);
    pthread_mutex_destroy(&client->mutex);
    free(client);
}

// Enhanced callback with compression and error handling
static size_t write_callback_enhanced(void *contents, size_t size, size_t nmemb, struct http_response *response) {
    size_t total_size = size * nmemb;
    
    if (UNLIKELY(!response || !contents)) {
        return 0;
    }
    
    // Dynamic capacity management
    if (response->size + total_size >= response->capacity) {
        size_t new_capacity = (response->capacity == 0) ? 1024 : response->capacity * 2;
        while (new_capacity < response->size + total_size + 1) {
            new_capacity *= 2;
        }
        
        char *new_data = realloc(response->data, new_capacity);
        if (UNLIKELY(!new_data)) {
            return 0;
        }
        
        response->data = new_data;
        response->capacity = new_capacity;
    }
    
    memcpy(response->data + response->size, contents, total_size);
    response->size += total_size;
    response->data[response->size] = '\0';
    
    return total_size;
}

int http_client_get(struct http_client *client, const char *url, struct http_response *response) {
    if (UNLIKELY(!client || !client->initialized || !url || !response)) {
        return HTTP_ERROR_REQUEST;
    }
    
    if (strlen(url) >= MAX_URL_LENGTH) {
        return HTTP_ERROR_INVALID_URL;
    }
    
    pthread_mutex_lock(&client->mutex);
    
    // Initialize response with metadata
    http_response_init(response);
    response->timestamp = (uint64_t)time(NULL);
    
    // Set URL and enhanced callbacks
    curl_easy_setopt(client->curl, CURLOPT_URL, url);
    curl_easy_setopt(client->curl, CURLOPT_WRITEFUNCTION, write_callback_enhanced);
    curl_easy_setopt(client->curl, CURLOPT_WRITEDATA, response);
    curl_easy_setopt(client->curl, CURLOPT_HTTPGET, 1L);
    
    // Perform request with comprehensive error handling
    CURLcode res = curl_easy_perform(client->curl);
    
    // Extract response metadata
    curl_easy_getinfo(client->curl, CURLINFO_RESPONSE_CODE, &response->status_code);
    curl_easy_getinfo(client->curl, CURLINFO_TOTAL_TIME, &response->total_time);
    
    char *content_type;
    if (curl_easy_getinfo(client->curl, CURLINFO_CONTENT_TYPE, &content_type) == CURLE_OK && content_type) {
        strncpy(response->content_type, content_type, sizeof(response->content_type) - 1);
        response->content_type[sizeof(response->content_type) - 1] = '\0';
    }
    
    // Update client statistics
    client->request_count++;
    client->total_time += response->total_time;
    
    pthread_mutex_unlock(&client->mutex);
    
    if (UNLIKELY(res != CURLE_OK)) {
        http_response_free(response);
        switch (res) {
            case CURLE_OPERATION_TIMEDOUT:
                return HTTP_ERROR_TIMEOUT;
            case CURLE_SSL_CONNECT_ERROR:
            case CURLE_SSL_CERTPROBLEM:
                return HTTP_ERROR_SSL;
            case CURLE_TOO_MANY_REDIRECTS:
                return HTTP_ERROR_REDIRECT;
            default:
                return HTTP_ERROR_NETWORK;
        }
    }
    
    return HTTP_OK;
}

void http_response_free(struct http_response *response) {
    if (LIKELY(response && response->data)) {
        free(response->data);
        response->data = NULL;
        response->size = 0;
        response->capacity = 0;
    }
}"#;

fn analyze_tree_structure(node: tree_sitter::Node, source: &str, indent: usize, max_depth: usize) {
    if indent > max_depth {
        return;
    }

    let indent_str = "  ".repeat(indent);
    let node_text = &source[node.start_byte()..node.end_byte().min(node.start_byte() + 50)];

    println!("{}├─ {} → {:?}", indent_str, node.kind(), node_text);

    if node.child_count() <= 3 && indent < max_depth {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            analyze_tree_structure(child, source, indent + 1, max_depth);
        }
    } else if node.child_count() > 3 {
        println!(
            "{}  └─ ... {} more children",
            indent_str,
            node.child_count()
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("⚙️  C Semantic Code Review Demo");
    println!();

    // Configuration
    let language = ProgrammingLanguage::C;

    // Setup source code objects
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);

    // Parse AST trees
    let c_parser = CParser::new();
    let mut ts_parser = Parser::new();
    ts_parser.set_language(c_parser.get_language())?;

    let old_tree = ts_parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old C code")?;
    let new_tree = ts_parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new C code")?;

    // Debug: Show AST structure
    println!("🌲 OLD AST structure:");
    analyze_tree_structure(old_tree.root_node(), OLD_CODE, 0, 2);
    println!("\n🌲 NEW AST structure:");
    analyze_tree_structure(new_tree.root_node(), NEW_CODE, 0, 2);

    let old_semantic_tree = c_parser
        .build_semantic_tree(&old_tree, OLD_CODE)
        .map_err(|e| format!("Failed to build old C semantic tree: {e}"))?;
    let new_semantic_tree = c_parser
        .build_semantic_tree(&new_tree, NEW_CODE)
        .map_err(|e| format!("Failed to build new C semantic tree: {e}"))?;

    println!("\n✅ Successfully built semantic trees!");

    // Debug: Show semantic tree structure
    println!(
        "📊 Old semantic tree has {} children",
        old_semantic_tree.root.children.len()
    );
    println!(
        "📊 New semantic tree has {} children",
        new_semantic_tree.root.children.len()
    );

    // Build semantic pairs and convert to reviewable diffs
    let semantic_pairs = build_semantic_pairs(
        &old_semantic_tree,
        &new_semantic_tree,
        &old_source,
        &new_source,
        &c_parser,
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
