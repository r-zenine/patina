//! Go-specific demonstration of the ReviewableDiff pipeline
//!
//! This example shows how to:
//! 1. Parse Go source code into AST trees
//! 2. Detect changes using Go-specific strategies
//! 3. Expand changes with semantic context including build tags
//! 4. Convert to structured ReviewableDiff objects
//! 5. Display results with debug formatting
//!
//! Note: This demo uses the Go parser with enhanced features including:
//! - Build tag and comment detection (// +build, //go:build)
//! - Method receiver analysis (pointer vs value receivers)
//! - Interface implementation tracking
//! - Goroutine and channel pattern detection
//! - Package visibility detection (capitalized vs lowercase)
//! - Error handling pattern recognition
//!
//! Run with: cargo run --example go_reviewable_diff_demo

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::{
    ast_diff::SourceCode, parsers::GoParser, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Original version - Traditional HTTP client with basic error handling
const OLD_CODE: &str = r#"// +build !integration
// +build linux darwin

package client

import (
    "context"
    "encoding/json"
    "fmt"
    "io/ioutil"
    "net/http"
    "time"
)

// User represents a system user
type User struct {
    ID       int    `json:"id"`
    Name     string `json:"name"`
    Email    string `json:"email"`
    Active   bool   `json:"active"`
}

// UserRepository handles user data operations
type UserRepository interface {
    GetUser(ctx context.Context, id int) (*User, error)
    CreateUser(ctx context.Context, user *User) error
    UpdateUser(ctx context.Context, user *User) error
    DeleteUser(ctx context.Context, id int) error
}

// HTTPClient provides HTTP functionality for user operations
type HTTPClient struct {
    client  *http.Client
    baseURL string
    timeout time.Duration
}

// NewHTTPClient creates a new HTTP client instance
func NewHTTPClient(baseURL string, timeout time.Duration) *HTTPClient {
    return &HTTPClient{
        client: &http.Client{
            Timeout: timeout,
        },
        baseURL: baseURL,
        timeout: timeout,
    }
}

// GetUser retrieves a user by ID
func (c *HTTPClient) GetUser(ctx context.Context, id int) (*User, error) {
    url := fmt.Sprintf("%s/users/%d", c.baseURL, id)
    
    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil {
        return nil, err
    }

    resp, err := c.client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    if resp.StatusCode != http.StatusOK {
        return nil, fmt.Errorf("HTTP error: %d", resp.StatusCode)
    }

    body, err := ioutil.ReadAll(resp.Body)
    if err != nil {
        return nil, err
    }

    var user User
    if err := json.Unmarshal(body, &user); err != nil {
        return nil, err
    }

    return &user, nil
}

// CreateUser creates a new user
func (c *HTTPClient) CreateUser(ctx context.Context, user *User) error {
    url := fmt.Sprintf("%s/users", c.baseURL)
    
    data, err := json.Marshal(user)
    if err != nil {
        return err
    }

    req, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewBuffer(data))
    if err != nil {
        return err
    }
    req.Header.Set("Content-Type", "application/json")

    resp, err := c.client.Do(req)
    if err != nil {
        return err
    }
    defer resp.Body.Close()

    if resp.StatusCode != http.StatusCreated {
        return fmt.Errorf("HTTP error: %d", resp.StatusCode)
    }

    return nil
}

// processUsers handles a batch of user operations
func processUsers(repo UserRepository, userIDs []int) {
    for _, id := range userIDs {
        ctx := context.Background()
        
        user, err := repo.GetUser(ctx, id)
        if err != nil {
            fmt.Printf("Error getting user %d: %v\n", id, err)
            continue
        }

        if user.Active {
            fmt.Printf("User %s is active\n", user.Name)
        } else {
            fmt.Printf("User %s is inactive\n", user.Name)
        }
    }
}

// validateUser performs basic user validation
func validateUser(user *User) error {
    if user == nil {
        return fmt.Errorf("user cannot be nil")
    }
    if user.Name == "" {
        return fmt.Errorf("user name cannot be empty")
    }
    if user.Email == "" {
        return fmt.Errorf("user email cannot be empty")
    }
    return nil
}
"#;

/// Refactored version - Modern Go client with advanced patterns
const NEW_CODE: &str = r#"//go:build !integration && (linux || darwin)
// +build !integration
// +build linux darwin

package client

import (
    "context"
    "encoding/json"
    "errors"
    "fmt"
    "io"
    "net/http"
    "sync"
    "time"

    "github.com/go-playground/validator/v10"
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/trace"
)

var (
    // ErrUserNotFound indicates that the requested user was not found
    ErrUserNotFound = errors.New("user not found")
    
    // ErrInvalidUser indicates that the user data is invalid
    ErrInvalidUser = errors.New("invalid user data")
    
    // ErrTimeout indicates that the operation timed out
    ErrTimeout = errors.New("operation timeout")
    
    // ErrRateLimited indicates that the request was rate limited
    ErrRateLimited = errors.New("rate limited")
)

// User represents a system user with validation tags
type User struct {
    ID       int    `json:"id" validate:"min=1"`
    Name     string `json:"name" validate:"required,min=2,max=100"`
    Email    string `json:"email" validate:"required,email"`
    Active   bool   `json:"active"`
    Role     string `json:"role" validate:"oneof=admin user guest"`
    Metadata map[string]interface{} `json:"metadata,omitempty"`
}

// UserRepository defines the interface for user data operations
type UserRepository interface {
    GetUser(ctx context.Context, id int) (*User, error)
    GetUsers(ctx context.Context, ids []int) ([]*User, error)
    CreateUser(ctx context.Context, user *User) (*User, error)
    UpdateUser(ctx context.Context, user *User) (*User, error)
    DeleteUser(ctx context.Context, id int) error
    SearchUsers(ctx context.Context, query string) ([]*User, error)
}

// UserCache provides caching capabilities for user data
type UserCache interface {
    Get(ctx context.Context, key string) (*User, error)
    Set(ctx context.Context, key string, user *User, ttl time.Duration) error
    Delete(ctx context.Context, key string) error
    Clear(ctx context.Context) error
}

// HTTPClientConfig holds configuration for the HTTP client
type HTTPClientConfig struct {
    BaseURL        string        `validate:"required,url"`
    Timeout        time.Duration `validate:"min=1s,max=5m"`
    MaxRetries     int           `validate:"min=0,max=10"`
    RetryDelay     time.Duration `validate:"min=100ms,max=30s"`
    MaxConcurrency int           `validate:"min=1,max=100"`
    EnableTracing  bool
    EnableMetrics  bool
}

// HTTPClient provides HTTP functionality with advanced features
type HTTPClient struct {
    client        *http.Client
    config        *HTTPClientConfig
    cache         UserCache
    validator     *validator.Validate
    tracer        trace.Tracer
    
    // Connection pooling and rate limiting
    semaphore     chan struct{}
    requestCount  int64
    mu            sync.RWMutex
    
    // Circuit breaker state
    failures      int
    lastFailure   time.Time
    isOpen        bool
}

// NewHTTPClient creates a new HTTP client with advanced configuration
func NewHTTPClient(config *HTTPClientConfig, cache UserCache) (*HTTPClient, error) {
    if err := validateConfig(config); err != nil {
        return nil, fmt.Errorf("invalid config: %w", err)
    }

    client := &HTTPClient{
        client: &http.Client{
            Timeout: config.Timeout,
            Transport: &http.Transport{
                MaxIdleConns:        10,
                MaxIdleConnsPerHost: 5,
                IdleConnTimeout:     30 * time.Second,
            },
        },
        config:    config,
        cache:     cache,
        validator: validator.New(),
        semaphore: make(chan struct{}, config.MaxConcurrency),
    }

    if config.EnableTracing {
        client.tracer = otel.Tracer("http-client")
    }

    return client, nil
}

// GetUser retrieves a user by ID with caching and circuit breaking
func (c *HTTPClient) GetUser(ctx context.Context, id int) (*User, error) {
    if id <= 0 {
        return nil, fmt.Errorf("invalid user ID: %d", id)
    }

    // Start tracing span
    ctx, span := c.startSpan(ctx, "GetUser")
    defer span.End()

    // Check cache first
    cacheKey := fmt.Sprintf("user:%d", id)
    if user, err := c.cache.Get(ctx, cacheKey); err == nil {
        span.AddEvent("cache_hit")
        return user, nil
    }

    // Check circuit breaker
    if c.isCircuitOpen() {
        return nil, ErrTimeout
    }

    // Rate limiting
    select {
    case c.semaphore <- struct{}{}:
        defer func() { <-c.semaphore }()
    case <-ctx.Done():
        return nil, ctx.Err()
    }

    user, err := c.fetchUserWithRetry(ctx, id)
    if err != nil {
        c.recordFailure()
        return nil, err
    }

    // Cache successful result
    _ = c.cache.Set(ctx, cacheKey, user, 5*time.Minute)
    c.recordSuccess()
    
    return user, nil
}

// GetUsers retrieves multiple users concurrently
func (c *HTTPClient) GetUsers(ctx context.Context, ids []int) ([]*User, error) {
    if len(ids) == 0 {
        return []*User{}, nil
    }

    ctx, span := c.startSpan(ctx, "GetUsers")
    defer span.End()

    type result struct {
        user *User
        err  error
    }

    results := make(chan result, len(ids))
    var wg sync.WaitGroup

    // Process users concurrently
    for _, id := range ids {
        wg.Add(1)
        go func(userID int) {
            defer wg.Done()
            
            user, err := c.GetUser(ctx, userID)
            select {
            case results <- result{user: user, err: err}:
            case <-ctx.Done():
            }
        }(id)
    }

    // Wait for all goroutines
    go func() {
        wg.Wait()
        close(results)
    }()

    // Collect results
    users := make([]*User, 0, len(ids))
    var errs []error

    for result := range results {
        if result.err != nil {
            errs = append(errs, result.err)
        } else if result.user != nil {
            users = append(users, result.user)
        }
    }

    if len(errs) > 0 && len(users) == 0 {
        return nil, fmt.Errorf("failed to fetch any users: %v", errs)
    }

    return users, nil
}

// CreateUser creates a new user with validation
func (c *HTTPClient) CreateUser(ctx context.Context, user *User) (*User, error) {
    if err := c.validateUser(user); err != nil {
        return nil, fmt.Errorf("validation failed: %w", err)
    }

    ctx, span := c.startSpan(ctx, "CreateUser")
    defer span.End()

    url := fmt.Sprintf("%s/users", c.config.BaseURL)
    
    data, err := json.Marshal(user)
    if err != nil {
        return nil, fmt.Errorf("failed to marshal user: %w", err)
    }

    resp, err := c.doRequestWithRetry(ctx, "POST", url, data)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    if resp.StatusCode != http.StatusCreated {
        return nil, c.handleHTTPError(resp)
    }

    var createdUser User
    if err := json.NewDecoder(resp.Body).Decode(&createdUser); err != nil {
        return nil, fmt.Errorf("failed to decode response: %w", err)
    }

    return &createdUser, nil
}

// fetchUserWithRetry implements retry logic for user fetching
func (c *HTTPClient) fetchUserWithRetry(ctx context.Context, id int) (*User, error) {
    var lastErr error
    
    for attempt := 0; attempt <= c.config.MaxRetries; attempt++ {
        if attempt > 0 {
            select {
            case <-time.After(c.config.RetryDelay):
            case <-ctx.Done():
                return nil, ctx.Err()
            }
        }

        user, err := c.fetchUser(ctx, id)
        if err == nil {
            return user, nil
        }

        lastErr = err
        if !c.isRetryableError(err) {
            break
        }
    }

    return nil, lastErr
}

// fetchUser performs the actual HTTP request
func (c *HTTPClient) fetchUser(ctx context.Context, id int) (*User, error) {
    url := fmt.Sprintf("%s/users/%d", c.config.BaseURL, id)
    
    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil {
        return nil, fmt.Errorf("failed to create request: %w", err)
    }

    resp, err := c.client.Do(req)
    if err != nil {
        return nil, fmt.Errorf("request failed: %w", err)
    }
    defer resp.Body.Close()

    if resp.StatusCode == http.StatusNotFound {
        return nil, ErrUserNotFound
    }

    if resp.StatusCode != http.StatusOK {
        return nil, c.handleHTTPError(resp)
    }

    var user User
    if err := json.NewDecoder(resp.Body).Decode(&user); err != nil {
        return nil, fmt.Errorf("failed to decode user: %w", err)
    }

    return &user, nil
}

// processUsersAsync handles batch operations with worker pool pattern
func processUsersAsync(ctx context.Context, repo UserRepository, userIDs []int, workerCount int) <-chan *User {
    userChan := make(chan *User, len(userIDs))
    idChan := make(chan int, len(userIDs))
    
    // Send all IDs to channel
    go func() {
        defer close(idChan)
        for _, id := range userIDs {
            select {
            case idChan <- id:
            case <-ctx.Done():
                return
            }
        }
    }()

    // Start workers
    var wg sync.WaitGroup
    for i := 0; i < workerCount; i++ {
        wg.Add(1)
        go func() {
            defer wg.Done()
            for id := range idChan {
                if user, err := repo.GetUser(ctx, id); err == nil {
                    select {
                    case userChan <- user:
                    case <-ctx.Done():
                        return
                    }
                }
            }
        }()
    }

    // Close output channel when all workers finish
    go func() {
        wg.Wait()
        close(userChan)
    }()

    return userChan
}

// validateUser performs comprehensive user validation
func (c *HTTPClient) validateUser(user *User) error {
    if user == nil {
        return ErrInvalidUser
    }

    if err := c.validator.Struct(user); err != nil {
        return fmt.Errorf("validation failed: %w", err)
    }

    return nil
}

// Circuit breaker methods
func (c *HTTPClient) isCircuitOpen() bool {
    c.mu.RLock()
    defer c.mu.RUnlock()
    
    if !c.isOpen {
        return false
    }
    
    // Reset after timeout
    if time.Since(c.lastFailure) > 30*time.Second {
        c.mu.RUnlock()
        c.mu.Lock()
        c.isOpen = false
        c.failures = 0
        c.mu.Unlock()
        c.mu.RLock()
    }
    
    return c.isOpen
}

// recordFailure updates circuit breaker state on failure
func (c *HTTPClient) recordFailure() {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    c.failures++
    c.lastFailure = time.Now()
    
    if c.failures >= 5 {
        c.isOpen = true
    }
}

// recordSuccess resets circuit breaker state on success
func (c *HTTPClient) recordSuccess() {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    c.failures = 0
    c.isOpen = false
}

// Helper methods
func (c *HTTPClient) startSpan(ctx context.Context, operation string) (context.Context, trace.Span) {
    if c.tracer != nil {
        return c.tracer.Start(ctx, operation)
    }
    return ctx, trace.SpanFromContext(ctx)
}

func (c *HTTPClient) isRetryableError(err error) bool {
    return errors.Is(err, ErrTimeout) || errors.Is(err, ErrRateLimited)
}

func (c *HTTPClient) handleHTTPError(resp *http.Response) error {
    switch resp.StatusCode {
    case http.StatusNotFound:
        return ErrUserNotFound
    case http.StatusTooManyRequests:
        return ErrRateLimited
    case http.StatusRequestTimeout:
        return ErrTimeout
    default:
        body, _ := io.ReadAll(resp.Body)
        return fmt.Errorf("HTTP error %d: %s", resp.StatusCode, string(body))
    }
}

func validateConfig(config *HTTPClientConfig) error {
    validate := validator.New()
    return validate.Struct(config)
}
"#;

fn main() -> Result<(), Box<dyn Error>> {
    println!("⚡ Go Semantic Code Review Demo");
    println!();

    // Configuration
    let language = ProgrammingLanguage::Go;

    // Setup source code objects
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);

    // Parse AST trees
    let go_parser = GoParser::new();
    let mut parser = Parser::new();
    parser.set_language(go_parser.get_language())?;

    let old_tree = parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old code")?;
    let new_tree = parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new code")?;

    let old_semantic_tree = go_parser
        .build_semantic_tree(&old_tree, OLD_CODE)
        .map_err(|e| format!("Failed to build old semantic tree: {e}"))?;
    let new_semantic_tree = go_parser
        .build_semantic_tree(&new_tree, NEW_CODE)
        .map_err(|e| format!("Failed to build new semantic tree: {e}"))?;

    // Build semantic pairs and convert to reviewable diffs
    let semantic_pairs = build_semantic_pairs(
        &old_semantic_tree,
        &new_semantic_tree,
        &old_source,
        &new_source,
        &go_parser,
    )
    .map_err(|e| format!("Failed to build semantic pairs: {e}"))?;

    let semantic_reviewable_diffs =
        semantic_pairs_to_reviewable_diffs(&semantic_pairs, language, &old_source, &new_source);

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
        "\n✅ Go Review complete: {} changes detected",
        visible_changes.len()
    );

    println!("\n🎯 Go-specific features demonstrated:");
    println!("   • Build tag detection (//go:build, // +build)");
    println!("   • Method receiver analysis (pointer vs value receivers)");
    println!("   • Interface implementation tracking");
    println!("   • Goroutine and channel pattern detection");
    println!("   • Package visibility detection (capitalized vs lowercase)");
    println!("   • Error handling pattern recognition");
    println!("   • Struct tag analysis (`json:\"field\" validate:\"required\"`)");
    println!("   • Context-aware request handling");

    Ok(())
}
