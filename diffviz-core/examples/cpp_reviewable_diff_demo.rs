//! Complete C++ semantic code review demo
//!
//! This demonstrates the C++ parser's ability to detect and analyze
//! semantic changes in C++ code including classes, templates, attributes,
//! modern C++ features, and inheritance patterns.

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::{
    ast_diff::SourceCode, parsers::CppParser, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Original version - Basic C++ class with traditional patterns
const OLD_CODE: &str = r#"#include <iostream>
#include <vector>

// Basic data processor class
class DataProcessor {
private:
    std::vector<int> data;
    size_t capacity;
    
public:
    DataProcessor(size_t cap) : capacity(cap) {
        data.reserve(cap);
    }
    
    void addItem(int item) {
        if (data.size() < capacity) {
            data.push_back(item);
        }
    }
    
    int getCount() const {
        return static_cast<int>(data.size());
    }
    
    void process() {
        for (int& item : data) {
            item *= 2;
        }
    }
};

// Simple enum for processor states
enum ProcessorState {
    IDLE,
    PROCESSING,
    COMPLETE
};

// Helper function
void printStatus(ProcessorState state) {
    switch (state) {
        case IDLE:
            std::cout << "Idle\n";
            break;
        case PROCESSING:
            std::cout << "Processing\n";
            break;
        case COMPLETE:
            std::cout << "Complete\n";
            break;
    }
}

// Global configuration
struct Config {
    int maxItems;
    bool enableLogging;
};"#;

/// Enhanced version - Modern C++ with templates, attributes, and advanced features
const NEW_CODE: &str = r#"#include <iostream>
#include <vector>
#include <memory>
#include <concepts>
#include <algorithm>
#include <mutex>

// Modern C++20 concept for type constraints
template<typename T>
concept Processable = requires(T t) {
    t * T{2};
    { t + t } -> std::convertible_to<T>;
};

// Enhanced template class with attributes and modern C++ features
template<typename T, size_t N = 100>
class [[nodiscard]] DataProcessor {
private:
    std::vector<T> data;
    size_t capacity;
    std::unique_ptr<T[]> buffer;
    mutable std::mutex dataMutex;
    
public:
    [[nodiscard]] explicit DataProcessor(size_t cap) : capacity(cap) {
        data.reserve(cap);
        buffer = std::make_unique<T[]>(cap);
    }
    
    // Template method with perfect forwarding and concept constraints
    template<Processable U>
    void addItem(U&& item) noexcept(std::is_nothrow_move_constructible_v<U>) {
        std::lock_guard<std::mutex> lock(dataMutex);
        if (data.size() < capacity) {
            data.emplace_back(std::forward<U>(item));
        }
    }
    
    [[nodiscard]] constexpr auto getCount() const noexcept -> size_t {
        std::lock_guard<std::mutex> lock(dataMutex);
        return data.size();
    }
    
    // Enhanced processing with algorithm library
    template<typename Func>
    void process(Func&& func) requires std::invocable<Func, T&> {
        std::lock_guard<std::mutex> lock(dataMutex);
        std::for_each(data.begin(), data.end(), std::forward<Func>(func));
    }
    
    // Move semantics and rule of five
    DataProcessor(DataProcessor&& other) noexcept = default;
    DataProcessor& operator=(DataProcessor&& other) noexcept = default;
    
    virtual ~DataProcessor() = default;
    
    // Deleted copy operations for performance
    DataProcessor(const DataProcessor&) = delete;
    DataProcessor& operator=(const DataProcessor&) = delete;
};

// Modern scoped enum with explicit underlying type
enum class ProcessorState : uint8_t {
    IDLE = 0,
    PROCESSING = 1,
    COMPLETE = 2,
    ERROR = 3,
    SUSPENDED = 4
};

// Enhanced helper function with constexpr and string_view
[[nodiscard]] constexpr std::string_view getStatusString(ProcessorState state) noexcept {
    switch (state) {
        case ProcessorState::IDLE:
            return "Idle";
        case ProcessorState::PROCESSING:
            return "Processing";
        case ProcessorState::COMPLETE:
            return "Complete";
        case ProcessorState::ERROR:
            return "Error";
        case ProcessorState::SUSPENDED:
            return "Suspended";
        default:
            return "Unknown";
    }
}

// Template specialization for integer processing
template<>
class DataProcessor<int, 50> {
private:
    std::array<int, 50> data;
    size_t count = 0;
    
public:
    void addItem(int item) {
        if (count < 50) {
            data[count++] = item;
        }
    }
    
    [[nodiscard]] constexpr size_t getCount() const noexcept { 
        return count; 
    }
};

// Enhanced configuration with template and attributes  
template<typename T = int>
struct [[maybe_unused]] Config {
    T maxItems{100};
    bool enableLogging{true};
    std::chrono::milliseconds timeout{1000};
    
    [[nodiscard]] constexpr bool isValid() const noexcept {
        return maxItems > 0 && timeout.count() > 0;
    }
};"#;

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
    println!("⚙️  C++ Semantic Code Review Demo");
    println!();

    // Configuration
    let language = ProgrammingLanguage::Cpp;

    // Setup source code objects
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);

    // Parse AST trees
    let cpp_parser = CppParser::new();
    let mut ts_parser = Parser::new();
    ts_parser.set_language(cpp_parser.get_language())?;

    let old_tree = ts_parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old C++ code")?;
    let new_tree = ts_parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new C++ code")?;

    // Debug: Show AST structure
    println!("🌲 OLD AST structure:");
    analyze_tree_structure(old_tree.root_node(), OLD_CODE, 0, 2);
    println!("\n🌲 NEW AST structure:");
    analyze_tree_structure(new_tree.root_node(), NEW_CODE, 0, 2);

    let old_semantic_tree = cpp_parser
        .build_semantic_tree(&old_tree, OLD_CODE)
        .map_err(|e| format!("Failed to build old C++ semantic tree: {e}"))?;
    let new_semantic_tree = cpp_parser
        .build_semantic_tree(&new_tree, NEW_CODE)
        .map_err(|e| format!("Failed to build new C++ semantic tree: {e}"))?;

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
        &cpp_parser,
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
