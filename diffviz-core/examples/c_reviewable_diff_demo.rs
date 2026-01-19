//! C-specific demonstration of the ReviewableDiff pipeline
//!
//! This example shows how to:
//! 1. Parse C source code into AST trees
//! 2. Detect changes using C-specific strategies
//! 3. Expand changes with semantic context including preprocessor directives
//! 4. Convert to structured ReviewableDiff objects
//! 5. Display results with debug formatting
//!
//! Note: This demo uses the C parser to showcase C-specific features including:
//! - Preprocessor directives (#include, #define, #pragma)
//! - Function declarations and definitions
//! - Struct and enum declarations
//! - Pointer and array types
//! - Static and inline keywords
//! - Header guards and conditional compilation
//!
//! Run with: cargo run --example c_reviewable_diff_demo

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::{
    ast_diff::SourceCode, parsers::CParser, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Original version - Traditional C library with basic error handling
const OLD_CODE: &str = r#"#ifndef HTTP_CLIENT_H
#define HTTP_CLIENT_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <curl/curl.h>

#define MAX_URL_LENGTH 256
#define MAX_RESPONSE_SIZE 4096
#define DEFAULT_TIMEOUT 30

// Response structure
struct http_response {
    char *data;
    size_t size;
};

// Client configuration
struct http_client {
    CURL *curl;
    long timeout;
    char user_agent[64];
};

// Error codes
enum http_error {
    HTTP_OK = 0,
    HTTP_ERROR_INIT = -1,
    HTTP_ERROR_REQUEST = -2,
    HTTP_ERROR_MEMORY = -3
};

// Function declarations
struct http_client* http_client_new(void);
void http_client_free(struct http_client *client);
int http_client_set_timeout(struct http_client *client, long timeout);
int http_client_get(struct http_client *client, const char *url, struct http_response *response);
void http_response_free(struct http_response *response);

// Implementation

struct http_client* http_client_new(void) {
    struct http_client *client = malloc(sizeof(struct http_client));
    if (!client) {
        return NULL;
    }
    
    client->curl = curl_easy_init();
    if (!client->curl) {
        free(client);
        return NULL;
    }
    
    client->timeout = DEFAULT_TIMEOUT;
    strcpy(client->user_agent, "HttpClient/1.0");
    
    return client;
}

void http_client_free(struct http_client *client) {
    if (client) {
        if (client->curl) {
            curl_easy_cleanup(client->curl);
        }
        free(client);
    }
}

int http_client_set_timeout(struct http_client *client, long timeout) {
    if (!client || timeout <= 0) {
        return HTTP_ERROR_REQUEST;
    }
    
    client->timeout = timeout;
    return HTTP_OK;
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
}

#endif // HTTP_CLIENT_H
"#;

/// Refactored version - Modern C library with advanced features and better error handling
const NEW_CODE: &str = r#"#ifndef HTTP_CLIENT_H
#define HTTP_CLIENT_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <stdint.h>
#include <curl/curl.h>
#include <pthread.h>

// Compiler-specific optimizations
#ifdef __GNUC__
    #define LIKELY(x)   __builtin_expect(!!(x), 1)
    #define UNLIKELY(x) __builtin_expect(!!(x), 0)
    #define FORCE_INLINE __attribute__((always_inline)) inline
#else
    #define LIKELY(x)   (x)
    #define UNLIKELY(x) (x)
    #define FORCE_INLINE inline
#endif

// Memory alignment and packing
#pragma pack(push, 1)

// Configuration constants
#define MAX_URL_LENGTH 512
#define MAX_RESPONSE_SIZE 8192
#define DEFAULT_TIMEOUT 60
#define MAX_REDIRECTS 5
#define MAX_USER_AGENT_LENGTH 128
#define MAX_HEADERS 16

// Thread safety macros
#define THREAD_SAFE __attribute__((warn_unused_result))
#define DEPRECATED __attribute__((deprecated))

// Response structure with enhanced metadata
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

// Header structure for custom headers
struct http_header {
    char name[64];
    char value[256];
};

// Advanced client configuration
struct http_client_config {
    long timeout;
    long connect_timeout;
    bool follow_redirects;
    long max_redirects;
    bool verify_ssl;
    bool enable_compression;
    bool enable_cookies;
    char user_agent[MAX_USER_AGENT_LENGTH];
    char proxy_url[MAX_URL_LENGTH];
};

// Client structure with thread safety
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

#pragma pack(pop)

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

// Callback function types
typedef size_t (*http_write_callback_t)(void *contents, size_t size, size_t nmemb, void *userp);
typedef int (*http_progress_callback_t)(void *clientp, double dltotal, double dlnow, double ultotal, double ulnow);

// Function declarations with enhanced signatures
THREAD_SAFE struct http_client* http_client_new(void);
THREAD_SAFE struct http_client* http_client_new_with_config(const struct http_client_config *config);
void http_client_free(struct http_client *client);

// Configuration functions
THREAD_SAFE int http_client_set_timeout(struct http_client *client, long timeout);
THREAD_SAFE int http_client_set_user_agent(struct http_client *client, const char *user_agent);
THREAD_SAFE int http_client_add_header(struct http_client *client, const char *name, const char *value);
THREAD_SAFE int http_client_clear_headers(struct http_client *client);
THREAD_SAFE int http_client_set_proxy(struct http_client *client, const char *proxy_url);

// Request functions
THREAD_SAFE int http_client_get(struct http_client *client, const char *url, struct http_response *response);
THREAD_SAFE int http_client_post(struct http_client *client, const char *url, const char *data, struct http_response *response);
THREAD_SAFE int http_client_put(struct http_client *client, const char *url, const char *data, struct http_response *response);
THREAD_SAFE int http_client_delete(struct http_client *client, const char *url, struct http_response *response);

// Advanced request functions
THREAD_SAFE int http_client_request_async(struct http_client *client, const char *method, const char *url, 
                                         const char *data, struct http_response *response,
                                         http_progress_callback_t progress_callback);

// Response functions
void http_response_init(struct http_response *response);
void http_response_free(struct http_response *response);
FORCE_INLINE bool http_response_is_success(const struct http_response *response);
FORCE_INLINE bool http_response_is_redirect(const struct http_response *response);
FORCE_INLINE bool http_response_is_error(const struct http_response *response);

// Utility functions
const char* http_error_string(enum http_error error);
THREAD_SAFE int http_client_get_stats(const struct http_client *client, uint32_t *request_count, double *total_time);

// Implementation with enhanced error handling

struct http_client* http_client_new(void) {
    struct http_client_config default_config = {
        .timeout = DEFAULT_TIMEOUT,
        .connect_timeout = 10,
        .follow_redirects = true,
        .max_redirects = MAX_REDIRECTS,
        .verify_ssl = true,
        .enable_compression = true,
        .enable_cookies = false,
        .user_agent = "HttpClient/2.0",
        .proxy_url = ""
    };
    
    return http_client_new_with_config(&default_config);
}

struct http_client* http_client_new_with_config(const struct http_client_config *config) {
    if (UNLIKELY(!config)) {
        return NULL;
    }
    
    struct http_client *client = calloc(1, sizeof(struct http_client));
    if (UNLIKELY(!client)) {
        return NULL;
    }
    
    // Initialize mutex for thread safety
    if (pthread_mutex_init(&client->mutex, NULL) != 0) {
        free(client);
        return NULL;
    }
    
    client->curl = curl_easy_init();
    if (UNLIKELY(!client->curl)) {
        pthread_mutex_destroy(&client->mutex);
        free(client);
        return NULL;
    }
    
    // Copy configuration
    client->config = *config;
    client->header_count = 0;
    client->initialized = true;
    client->request_count = 0;
    client->total_time = 0.0;
    
    // Set default curl options
    curl_easy_setopt(client->curl, CURLOPT_TIMEOUT, config->timeout);
    curl_easy_setopt(client->curl, CURLOPT_CONNECTTIMEOUT, config->connect_timeout);
    curl_easy_setopt(client->curl, CURLOPT_FOLLOWLOCATION, config->follow_redirects ? 1L : 0L);
    curl_easy_setopt(client->curl, CURLOPT_MAXREDIRS, config->max_redirects);
    curl_easy_setopt(client->curl, CURLOPT_SSL_VERIFYPEER, config->verify_ssl ? 1L : 0L);
    curl_easy_setopt(client->curl, CURLOPT_ACCEPT_ENCODING, config->enable_compression ? "gzip, deflate" : "");
    curl_easy_setopt(client->curl, CURLOPT_USERAGENT, config->user_agent);
    
    if (config->proxy_url[0] != '\0') {
        curl_easy_setopt(client->curl, CURLOPT_PROXY, config->proxy_url);
    }
    
    return client;
}

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

// Enhanced callback with compression support
static size_t write_callback_enhanced(void *contents, size_t size, size_t nmemb, struct http_response *response) {
    size_t total_size = size * nmemb;
    
    if (UNLIKELY(!response || !contents)) {
        return 0;
    }
    
    // Ensure capacity
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
    
    // Initialize response
    http_response_init(response);
    response->timestamp = (uint64_t)time(NULL);
    
    // Set URL and callbacks
    curl_easy_setopt(client->curl, CURLOPT_URL, url);
    curl_easy_setopt(client->curl, CURLOPT_WRITEFUNCTION, write_callback_enhanced);
    curl_easy_setopt(client->curl, CURLOPT_WRITEDATA, response);
    curl_easy_setopt(client->curl, CURLOPT_HTTPGET, 1L);
    
    // Perform request
    CURLcode res = curl_easy_perform(client->curl);
    
    // Get response info
    curl_easy_getinfo(client->curl, CURLINFO_RESPONSE_CODE, &response->status_code);
    curl_easy_getinfo(client->curl, CURLINFO_TOTAL_TIME, &response->total_time);
    
    char *content_type;
    if (curl_easy_getinfo(client->curl, CURLINFO_CONTENT_TYPE, &content_type) == CURLE_OK && content_type) {
        strncpy(response->content_type, content_type, sizeof(response->content_type) - 1);
        response->content_type[sizeof(response->content_type) - 1] = '\0';
    }
    
    // Update client stats
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

void http_response_init(struct http_response *response) {
    if (LIKELY(response)) {
        response->data = NULL;
        response->size = 0;
        response->capacity = 0;
        response->status_code = 0;
        response->total_time = 0.0;
        response->content_type[0] = '\0';
        response->is_compressed = false;
        response->timestamp = 0;
    }
}

void http_response_free(struct http_response *response) {
    if (LIKELY(response && response->data)) {
        free(response->data);
        response->data = NULL;
        response->size = 0;
        response->capacity = 0;
    }
}

FORCE_INLINE bool http_response_is_success(const struct http_response *response) {
    return response && (response->status_code >= 200 && response->status_code < 300);
}

const char* http_error_string(enum http_error error) {
    switch (error) {
        case HTTP_OK: return "Success";
        case HTTP_ERROR_INIT: return "Initialization error";
        case HTTP_ERROR_REQUEST: return "Invalid request";
        case HTTP_ERROR_MEMORY: return "Memory allocation error";
        case HTTP_ERROR_TIMEOUT: return "Request timeout";
        case HTTP_ERROR_SSL: return "SSL/TLS error";
        case HTTP_ERROR_REDIRECT: return "Too many redirects";
        case HTTP_ERROR_THREAD: return "Thread error";
        case HTTP_ERROR_INVALID_URL: return "Invalid URL";
        case HTTP_ERROR_NETWORK: return "Network error";
        default: return "Unknown error";
    }
}

#ifdef __cplusplus
}
#endif

#endif // HTTP_CLIENT_H
"#;

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
    let mut parser = Parser::new();
    parser.set_language(c_parser.get_language())?;

    let old_tree = parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old code")?;
    let new_tree = parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new code")?;

    // Try to build semantic trees (this will likely fail since C parser is a stub)
    match c_parser.build_semantic_tree(&old_tree, OLD_CODE) {
        Ok(old_semantic_tree) => {
            match c_parser.build_semantic_tree(&new_tree, NEW_CODE) {
                Ok(new_semantic_tree) => {
                    println!("✅ Successfully built semantic trees!");

                    // Build semantic pairs and convert to reviewable diffs
                    let semantic_pairs = build_semantic_pairs(
                        &old_semantic_tree,
                        &new_semantic_tree,
                        &old_source,
                        &new_source,
                        &c_parser,
                    )?;

                    let semantic_reviewable_diffs = semantic_pairs_to_reviewable_diffs(
                        &semantic_pairs,
                        language,
                        &old_source,
                        &new_source,
                    );

                    // Filter out changes with no visible content
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
                            println!(
                                "   📊 {changed_lines} changed lines, {hidden_lines} hidden lines"
                            );
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
                        "\n✅ C Review complete: {} changes detected",
                        visible_changes.len()
                    );
                }
                Err(e) => {
                    println!("❌ Failed to build new semantic tree: {e}");
                    demonstrate_c_parser_capabilities(
                        &c_parser, &old_tree, &new_tree, OLD_CODE, NEW_CODE,
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ C parser does not support semantic tree building: {e}");
            demonstrate_c_parser_capabilities(&c_parser, &old_tree, &new_tree, OLD_CODE, NEW_CODE);
        }
    }

    println!("\n🎯 C-specific constructs in this demo:");
    println!("   • Preprocessor directives (#include, #define, #pragma)");
    println!("   • Header guards (#ifndef/#define/#endif)");
    println!("   • Conditional compilation (#ifdef __GNUC__)");
    println!("   • Compiler attributes (__attribute__)");
    println!("   • Memory alignment (#pragma pack)");
    println!("   • Function pointers and callbacks");
    println!("   • Static and inline keywords");
    println!("   • Struct packing and alignment");
    println!("   • Thread safety with pthread_mutex_t");
    println!("   • Error handling with enums");
    println!("   • Pointer arithmetic and memory management");

    Ok(())
}

fn demonstrate_c_parser_capabilities(
    parser: &CParser,
    old_tree: &tree_sitter::Tree,
    new_tree: &tree_sitter::Tree,
    old_code: &str,
    new_code: &str,
) {
    println!("\n🔍 Demonstrating C Parser Capabilities:");

    // Test node classification capabilities
    println!("\n📋 Node Classification Test:");
    let test_nodes = [
        "function_definition",
        "function_declarator",
        "struct_specifier",
        "enum_specifier",
        "declaration",
        "preproc_include",
        "preproc_def",
        "preproc_ifdef",
        "preproc_pragma",
        "if_statement",
        "for_statement",
        "call_expression",
        "pointer_declarator",
        "array_declarator",
        "compound_statement",
        "unknown_node_type",
    ];

    for node_type in &test_nodes {
        let kind = parser.classify_node_kind(node_type);
        println!("   {node_type} → {kind:?}");
    }

    // Analyze old tree structure
    println!("\n🌳 Old Tree Analysis:");
    analyze_tree_structure(old_tree.root_node(), old_code, 0, 3);

    println!("\n🌳 New Tree Analysis (first 10 nodes):");
    analyze_tree_structure(new_tree.root_node(), new_code, 0, 2);
}

fn analyze_tree_structure(node: tree_sitter::Node, source: &str, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }

    let indent = "  ".repeat(depth);
    let text = node
        .utf8_text(source.as_bytes())
        .unwrap_or("")
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(50)
        .collect::<String>();

    println!(
        "{}{}({}) \"{}\"",
        indent,
        node.kind(),
        node.child_count(),
        text
    );

    // Show first few children
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).take(5).collect();
    for child in children {
        analyze_tree_structure(child, source, depth + 1, max_depth);
    }

    if node.child_count() > 5 {
        println!("{}  ... ({} more children)", indent, node.child_count() - 5);
    }
}
