//! TypeScript parser integration tests.
//!
//! These tests verify that the TypeScriptDescriptor-based parser correctly builds
//! semantic trees for TypeScript code, including files with interfaces, classes, and
//! enums. The original bug (modified TS files shown as "New file") lives in the
//! review layer and is tracked separately; these tests confirm the parser layer works.

#[cfg(test)]
mod typescript_file_classification_bug_tests {
    use diffviz_core::common::LanguageParser;
    use diffviz_core::common::ProgrammingLanguage;
    use diffviz_core::parsers::TypeScriptParser;

    #[test]
    fn test_typescript_modified_file_classification() {
        // Verify that the TypeScript descriptor-based parser can build a semantic tree
        // for a TypeScript file that was previously "modified" (has original + new version).
        let parser = TypeScriptParser::new();

        let modified_ts = r#"export interface User {
    id: string;
    name: string;
    email: string;
    role: UserRole;
    createdAt: Date;
}

export enum UserRole {
    ADMIN = 'admin',
    USER = 'user'
}"#;

        let tree = parser
            .try_parse(modified_ts)
            .expect("TypeScript parse must succeed");
        let semantic_tree = parser
            .build_semantic_tree(&tree, modified_ts)
            .expect("build_semantic_tree must succeed for TypeScript");

        // The root should have children (interface and enum declarations)
        assert!(
            !semantic_tree.root.children.is_empty(),
            "TypeScript semantic tree must contain semantic nodes"
        );
    }

    #[test]
    fn test_typescript_file_type_detection() {
        // Verify that multiple TypeScript files (simulating before/after of a modification)
        // both parse successfully with the descriptor-based parser.
        let parser = TypeScriptParser::new();

        let original_ts = r#"export interface IUser {
  id: string;
  email: string;
  username: string;
}"#;

        let modified_ts = r#"export interface IUser {
  id: string;
  email: string;
  username: string;
  preferences?: UserPreferences;
}

export interface UserPreferences {
  newsletter: boolean;
  darkMode: boolean;
}"#;

        // Both versions must produce valid semantic trees
        for source in &[original_ts, modified_ts] {
            let tree = parser.try_parse(source).expect("parse must succeed");
            let result = parser.build_semantic_tree(&tree, source);
            assert!(
                result.is_ok(),
                "build_semantic_tree must succeed for TypeScript source: {result:?}"
            );
        }
    }

    #[test]
    fn test_typescript_new_files_work_correctly() {
        let parser = TypeScriptParser::new();

        let new_ts = r#"export class Calculator {
    add(a: number, b: number): number {
        return a + b;
    }
}"#;

        let tree = parser.try_parse(new_ts).expect("parse must succeed");
        let semantic_tree = parser
            .build_semantic_tree(&tree, new_ts)
            .expect("build_semantic_tree must succeed");
        assert!(!semantic_tree.root.children.is_empty());
    }
}

/// Test data retained for external use.
pub mod test_data {
    pub const ORIGINAL_USER_TS: &str = r#"export interface IUser {
  id: string;
  email: string;
  username: string;
  firstName: string;
  lastName: string;
}"#;

    pub const MODIFIED_USER_TS: &str = r#"export interface IUser {
  id: string;
  email: string;
  username: string;
  firstName: string;
  lastName: string;
  preferences?: UserPreferences;
}

export interface UserPreferences {
  newsletter: boolean;
  darkMode: boolean;
}"#;

    pub const ORIGINAL_PRODUCT_SERVICE_TS: &str = r#"export class ProductService {
  static validateProductData(product: Partial<IProduct>): Result<boolean, string> {
    if (!product.name) {
      return { kind: 'error', error: 'Name required' };
    }
    return { kind: 'ok', value: true };
  }
}"#;

    pub const MODIFIED_PRODUCT_SERVICE_TS: &str = r#"export class ProductService {
  static validateProductData(product: Partial<IProduct>): Result<boolean, string> {
    if (!product.name) {
      return { kind: 'error', error: 'Name required' };
    }
    if (!product.description || product.description.length < 10) {
      return { kind: 'error', error: 'Description too short' };
    }
    if (product.categories && product.categories.length === 0) {
      return { kind: 'error', error: 'Need categories' };
    }
    return { kind: 'ok', value: true };
  }
}"#;
}
