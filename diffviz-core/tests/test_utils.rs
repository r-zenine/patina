//! Test utilities for diffviz-core tests
//!
//! This module provides common test infrastructure and helper functions
//! used across the test suite.

use diffviz_core::common::LanguageParser;
use diffviz_core::common::ProgrammingLanguage;
use diffviz_core::parsers::{
    CParser, CppParser, CssParser, GoParser, JavaParser, JavaScriptParser, JsonParser,
    PythonParser, RustParser, TomlParser, TypeScriptParser,
};

/// Get the appropriate language parser for a given programming language
///
/// This helper function returns a boxed LanguageParser implementation
/// for the specified programming language.
pub fn get_parser_for_language(language: ProgrammingLanguage) -> Box<dyn LanguageParser> {
    match language {
        ProgrammingLanguage::Rust => Box::new(RustParser::new()),
        ProgrammingLanguage::Go => Box::new(GoParser::new()),
        ProgrammingLanguage::Python => Box::new(PythonParser::new()),
        ProgrammingLanguage::TypeScript => Box::new(TypeScriptParser::new()),
        ProgrammingLanguage::Java => Box::new(JavaParser::new()),
        ProgrammingLanguage::C => Box::new(CParser::new()),
        ProgrammingLanguage::Cpp => Box::new(CppParser::new()),
        ProgrammingLanguage::Json => Box::new(JsonParser::new()),
        ProgrammingLanguage::JavaScript => Box::new(JavaScriptParser::new()),
        ProgrammingLanguage::Css => Box::new(CssParser::new()),
        ProgrammingLanguage::Toml => Box::new(TomlParser::new()),
        ProgrammingLanguage::Unknown => Box::new(RustParser::new()), // Default to Rust for unknown languages
    }
}
