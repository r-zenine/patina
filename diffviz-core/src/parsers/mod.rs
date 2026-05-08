pub mod c;
pub mod cpp;
pub mod descriptor;
pub mod generic_builder;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;
pub mod typescript;

pub use c::CParser;
pub use cpp::CppParser;
pub use go::GoParser;
pub use java::JavaParser;
pub use javascript::JavaScriptParser;
pub use python::PythonParser;
pub use rust::RustParser;
pub use typescript::TypeScriptParser;

use crate::common::{LanguageParser, ProgrammingLanguage};

/// Returns the appropriate parser for a known language, or `None` for `Unknown`.
pub fn parser_for_language(language: ProgrammingLanguage) -> Option<Box<dyn LanguageParser>> {
    match language {
        ProgrammingLanguage::Rust => Some(Box::new(RustParser::new())),
        ProgrammingLanguage::Python => Some(Box::new(PythonParser::new())),
        ProgrammingLanguage::Go => Some(Box::new(GoParser::new())),
        ProgrammingLanguage::Java => Some(Box::new(JavaParser::new())),
        ProgrammingLanguage::TypeScript => Some(Box::new(TypeScriptParser::new())),
        ProgrammingLanguage::JavaScript => Some(Box::new(JavaScriptParser::new())),
        ProgrammingLanguage::C => Some(Box::new(CParser::new())),
        ProgrammingLanguage::Cpp => Some(Box::new(CppParser::new())),
        ProgrammingLanguage::Unknown => None,
    }
}
