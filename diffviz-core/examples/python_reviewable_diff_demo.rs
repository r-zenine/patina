//! Python-specific demonstration of the ReviewableDiff pipeline
//!
//! This example shows how to:
//! 1. Parse Python source code into AST trees
//! 2. Detect changes using Python-specific strategies  
//! 3. Expand changes with semantic context including decorators
//! 4. Convert to structured ReviewableDiff objects
//! 5. Display results with debug formatting
//!
//! Run with: cargo run --example python_reviewable_diff_demo

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::{
    ast_diff::SourceCode, parsers::PythonParser, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Original version - Basic HTTP client with synchronous operations
const OLD_CODE: &str = r#"import requests
import json
from typing import Dict, Any

class APIError(Exception):
    def __init__(self, message):
        self.message = message
        super().__init__(self.message)

class HTTPClient:
    def __init__(self, base_url: str, timeout: int = 30):
        self.base_url = base_url
        self.timeout = timeout
        self.headers = {}
    
    def add_header(self, key: str, value: str):
        self.headers[key] = value
    
    def get(self, path: str) -> Dict[str, Any]:
        url = f"{self.base_url}/{path}"
        response = requests.get(url, headers=self.headers, timeout=self.timeout)
        
        if response.status_code != 200:
            raise APIError(f"Request failed with status {response.status_code}")
        
        return response.json()
    
    def post(self, path: str, data: Dict[str, Any]) -> Dict[str, Any]:
        url = f"{self.base_url}/{path}"
        response = requests.post(
            url, 
            headers=self.headers, 
            json=data, 
            timeout=self.timeout
        )
        
        if response.status_code not in [200, 201]:
            raise APIError(f"Request failed with status {response.status_code}")
        
        return response.json()

def create_client(api_key: str) -> HTTPClient:
    client = HTTPClient("https://api.example.com")
    client.add_header("Authorization", f"Bearer {api_key}")
    client.add_header("Content-Type", "application/json")
    return client
"#;

/// Refactored version - Async HTTP client with enhanced features and decorators
const NEW_CODE: &str = r#"import asyncio
import aiohttp
import json
import logging
from typing import Dict, Any, Optional
from dataclasses import dataclass
from abc import ABC, abstractmethod

logger = logging.getLogger(__name__)

@dataclass
class APIResponse:
    status_code: int
    data: Any
    headers: Dict[str, str]

class APIError(Exception):
    def __init__(self, message: str, status_code: Optional[int] = None):
        self.message = message
        self.status_code = status_code
        super().__init__(self.message)
    
    def __str__(self) -> str:
        if self.status_code:
            return f"API Error ({self.status_code}): {self.message}"
        return f"API Error: {self.message}"

class RateLimitError(APIError):
    def __init__(self, retry_after: int):
        super().__init__(f"Rate limit exceeded. Retry after {retry_after} seconds", 429)
        self.retry_after = retry_after

class HTTPClientInterface(ABC):
    @abstractmethod
    async def get(self, path: str) -> APIResponse:
        pass
    
    @abstractmethod
    async def post(self, path: str, data: Dict[str, Any]) -> APIResponse:
        pass

def retry(max_attempts: int = 3, delay: float = 1.0):
    """Decorator for retrying failed requests"""
    def decorator(func):
        async def wrapper(*args, **kwargs):
            last_exception = None
            for attempt in range(max_attempts):
                try:
                    return await func(*args, **kwargs)
                except (aiohttp.ClientError, asyncio.TimeoutError) as e:
                    last_exception = e
                    if attempt < max_attempts - 1:
                        await asyncio.sleep(delay * (2 ** attempt))
                        logger.warning(f"Attempt {attempt + 1} failed: {e}. Retrying...")
                    else:
                        logger.error(f"All {max_attempts} attempts failed")
            raise last_exception
        return wrapper
    return decorator

def log_requests(func):
    """Decorator for logging HTTP requests"""
    async def wrapper(self, *args, **kwargs):
        start_time = asyncio.get_event_loop().time()
        try:
            result = await func(self, *args, **kwargs)
            duration = asyncio.get_event_loop().time() - start_time
            logger.info(f"{func.__name__} completed in {duration:.2f}s")
            return result
        except Exception as e:
            duration = asyncio.get_event_loop().time() - start_time
            logger.error(f"{func.__name__} failed after {duration:.2f}s: {e}")
            raise
    return wrapper

class AsyncHTTPClient(HTTPClientInterface):
    def __init__(self, base_url: str, timeout: int = 30, max_connections: int = 100):
        self.base_url = base_url.rstrip("/")
        self.timeout = aiohttp.ClientTimeout(total=timeout)
        self.headers = {"User-Agent": "AsyncHTTPClient/1.0"}
        self._session = None
        self._connector = aiohttp.TCPConnector(limit=max_connections)
        self._request_count = 0
    
    async def __aenter__(self):
        self._session = aiohttp.ClientSession(
            connector=self._connector,
            timeout=self.timeout,
            headers=self.headers
        )
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._session:
            await self._session.close()
    
    def add_header(self, key: str, value: str):
        self.headers[key] = value
        if self._session:
            self._session.headers[key] = value
    
    def remove_header(self, key: str):
        """Remove a header from future requests"""
        self.headers.pop(key, None)
        if self._session and key in self._session.headers:
            del self._session.headers[key]
    
    @property
    def request_count(self) -> int:
        """Get the number of requests made by this client"""
        return self._request_count
    
    @retry(max_attempts=3, delay=1.0)
    @log_requests
    async def get(self, path: str, params: Optional[Dict[str, Any]] = None) -> APIResponse:
        return await self._make_request("GET", path, params=params)
    
    @retry(max_attempts=3, delay=1.0) 
    @log_requests
    async def post(self, path: str, data: Dict[str, Any]) -> APIResponse:
        return await self._make_request("POST", path, json=data)
    
    @retry(max_attempts=2, delay=0.5)
    @log_requests
    async def put(self, path: str, data: Dict[str, Any]) -> APIResponse:
        return await self._make_request("PUT", path, json=data)
    
    async def _make_request(self, method: str, path: str, **kwargs) -> APIResponse:
        if not self._session:
            raise APIError("Client session not initialized. Use 'async with' context.")
        
        self._request_count += 1
        url = f"{self.base_url}/{path.lstrip('/')}"
        
        try:
            async with self._session.request(method, url, **kwargs) as response:
                # Handle rate limiting
                if response.status == 429:
                    retry_after = int(response.headers.get("Retry-After", 60))
                    raise RateLimitError(retry_after)
                
                # Handle other errors
                if response.status >= 400:
                    error_text = await response.text()
                    raise APIError(
                        f"Request failed: {error_text}", 
                        status_code=response.status
                    )
                
                # Parse response
                content_type = response.headers.get("Content-Type", "")
                if "application/json" in content_type:
                    data = await response.json()
                else:
                    data = await response.text()
                
                return APIResponse(
                    status_code=response.status,
                    data=data,
                    headers=dict(response.headers)
                )
        
        except aiohttp.ClientError as e:
            raise APIError(f"Network error: {str(e)}")

async def create_client(api_key: str) -> AsyncHTTPClient:
    """Factory function to create configured HTTP client"""
    client = AsyncHTTPClient(
        base_url="https://api.example.com/v2",
        timeout=45,
        max_connections=50
    )
    client.add_header("Authorization", f"Bearer {api_key}")
    client.add_header("Content-Type", "application/json")
    client.add_header("Accept", "application/json")
    return client
"#;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🐍 Python Semantic Code Review Demo");
    println!();

    // Configuration
    let language = ProgrammingLanguage::Python;

    // Setup source code objects
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);

    // Parse AST trees
    let python_parser = PythonParser::new();
    let mut ts_parser = Parser::new();
    ts_parser.set_language(python_parser.get_language())?;

    let old_tree = ts_parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old code")?;
    let new_tree = ts_parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new code")?;

    let old_semantic_tree = python_parser
        .build_semantic_tree(&old_tree, OLD_CODE)
        .map_err(|e| format!("Failed to build old semantic tree: {e}"))?;
    let new_semantic_tree = python_parser
        .build_semantic_tree(&new_tree, NEW_CODE)
        .map_err(|e| format!("Failed to build new semantic tree: {e}"))?;

    // Build semantic pairs and convert to reviewable diffs
    let semantic_pairs = build_semantic_pairs(
        &old_semantic_tree,
        &new_semantic_tree,
        &old_source,
        &new_source,
        &python_parser,
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
        "\n✅ Python Review complete: {} changes detected",
        visible_changes.len()
    );

    println!("\n🎯 Python-specific features demonstrated:");
    println!("   • Decorator detection (@dataclass, @retry, @log_requests)");
    println!("   • Async function analysis (async def, await)");
    println!("   • Class inheritance tracking (ABC, Exception)");
    println!("   • Import statement parsing (from typing import, etc.)");
    println!("   • Method vs function distinction");
    println!("   • Magic method detection (__init__, __str__, __aenter__)");

    Ok(())
}
