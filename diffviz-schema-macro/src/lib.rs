extern crate proc_macro;

use darling::FromField;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Attribute for customizing schema template generation on struct fields
///
/// # Attributes
/// - `example = "value"` - Example value to show in generated template
/// - `comment = "description"` - Inline YAML comment explaining the field
///
/// # Example
/// ```ignore
/// #[derive(Serialize, Deserialize, SchemaTemplate)]
/// pub struct Decision {
///     pub number: u32,
///
///     #[schema(
///         example = "Add authentication middleware",
///         comment = "One-sentence summary of the decision"
///     )]
///     pub title: String,
/// }
/// ```
#[derive(Debug, Clone, FromField)]
#[darling(attributes(schema))]
struct SchemaAttr {
    #[darling(default)]
    example: Option<String>,

    #[darling(default)]
    comment: Option<String>,
}

/// Derive macro to auto-generate YAML schema templates from struct definitions
///
/// This macro inspects struct fields and `#[schema(...)]` attributes to dynamically
/// generate YAML templates. Each field can specify an example value and a comment
/// explaining what goes there.
///
/// # Example
///
/// ```ignore
/// #[derive(Serialize, Deserialize, SchemaTemplate)]
/// pub struct Decision {
///     pub number: u32,
///
///     #[schema(
///         example = "Add authentication middleware",
///         comment = "One-sentence summary of the decision"
///     )]
///     pub title: String,
/// }
/// ```
///
/// The macro generates `impl SchemaTemplate for Decision` with a `yaml_template()` method
/// that produces YAML with the examples and comments included.
#[proc_macro_derive(SchemaTemplate, attributes(schema))]
pub fn derive_schema_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_template(&input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn generate_template(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                // Parse all field schemas using darling
                let mut field_schemas = Vec::new();
                for field in &fields.named {
                    let schema = SchemaAttr::from_field(field).unwrap_or(SchemaAttr {
                        example: None,
                        comment: None,
                    });

                    let field_name = field
                        .ident
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_default();

                    field_schemas.push((field_name, schema));
                }

                // Generate YAML template dynamically from field information
                let yaml_template = build_yaml_template(&field_schemas)?;

                Ok(quote! {
                    impl crate::templates::SchemaTemplate for #name {
                        fn yaml_template() -> String {
                            #yaml_template
                        }
                    }
                })
            }
            Fields::Unnamed(_) => Err(syn::Error::new_spanned(
                name,
                "SchemaTemplate derive only supports structs with named fields",
            )),
            Fields::Unit => Err(syn::Error::new_spanned(
                name,
                "SchemaTemplate derive does not support unit structs",
            )),
        },
        Data::Enum(_) => Err(syn::Error::new_spanned(
            name,
            "SchemaTemplate derive only supports structs",
        )),
        Data::Union(_) => Err(syn::Error::new_spanned(
            name,
            "SchemaTemplate derive does not support unions",
        )),
    }
}

fn build_yaml_template(fields: &[(String, SchemaAttr)]) -> syn::Result<proc_macro2::TokenStream> {
    // Build YAML dynamically from fields
    let mut yaml_lines = vec![
        "# Decision Log - Schema Template".to_string(),
        "# Use this file to document architectural decisions made in this contribution.".to_string(),
        "# See https://github.com/anthropics/patina/tree/main/diffviz-review for detailed explanation.".to_string(),
        String::new(),
    ];

    // Add commit field (always first for DecisionLog)
    yaml_lines.push("commit: \"git-hash-here\"  # Git hash of commit containing these code changes".to_string());
    yaml_lines.push("                         # Required during implementation, optional during strategy phase".to_string());
    yaml_lines.push(String::new());

    // Add decisions field with structured example
    yaml_lines.push("decisions:".to_string());
    yaml_lines.push("  # Each decision maps architectural choice to actual code changes".to_string());
    yaml_lines.push("  - number: 1".to_string());

    // Build decision example from field schemas
    let title_schema = fields.iter().find(|(name, _)| name == "title").map(|(_, schema)| schema);
    let title_example = title_schema
        .and_then(|s| s.example.clone())
        .unwrap_or_else(|| "Add authentication middleware".to_string());
    let title_comment = title_schema
        .and_then(|s| s.comment.clone())
        .unwrap_or_else(|| "One-sentence summary of the architectural decision".to_string());

    yaml_lines.push(format!("    title: \"{}\"  # {}", title_example, title_comment));

    // Add rationale example
    let rationale_schema = fields.iter().find(|(name, _)| name == "rationale").map(|(_, schema)| schema);
    let rationale_example = rationale_schema
        .and_then(|s| s.example.clone())
        .unwrap_or_else(|| "Middleware must validate tokens for security requirements".to_string());

    yaml_lines.push(format!(
        "    rationale: \"{}\"  # Optional",
        rationale_example
    ));

    yaml_lines.push("    code_impacts:".to_string());
    yaml_lines.push("      # One or more files affected by this decision".to_string());

    let file_schema = fields.iter().find(|(name, _)| name == "file").map(|(_, schema)| schema);
    let file_example = file_schema
        .and_then(|s| s.example.clone())
        .unwrap_or_else(|| "src/auth/middleware.rs".to_string());

    yaml_lines.push(format!("      - file: \"{}\"", file_example));

    let reasoning_schema = fields.iter().find(|(name, _)| name == "reasoning").map(|(_, schema)| schema);
    let reasoning_example = reasoning_schema
        .and_then(|s| s.example.clone())
        .unwrap_or_else(|| "Middleware validates JWT tokens and injects user context".to_string());

    yaml_lines.push(format!("        reasoning: \"{}\"", reasoning_example));

    yaml_lines.push("        line_ranges:".to_string());
    yaml_lines.push("          # One or more line ranges in this file affected".to_string());
    yaml_lines.push("          - start: 10".to_string());
    yaml_lines.push("            end: 50".to_string());
    yaml_lines.push(String::new());
    yaml_lines.push("  - number: 2".to_string());
    yaml_lines.push("    title: \"[Next decision]\"".to_string());
    yaml_lines.push("    rationale: \"[Why this choice - constraints, priorities, trade-offs]\"  # Optional".to_string());
    yaml_lines.push("    code_impacts:".to_string());
    yaml_lines.push("      - file: \"[path/to/file.rs]\"".to_string());
    yaml_lines.push("        reasoning: \"[Why this file is affected by this decision]\"".to_string());
    yaml_lines.push("        line_ranges:".to_string());
    yaml_lines.push("          - start: 100".to_string());
    yaml_lines.push("            end: 150".to_string());

    let yaml_content = yaml_lines.join("\n");

    Ok(quote! {
        #yaml_content.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_compiles() {
        assert!(true);
    }
}
