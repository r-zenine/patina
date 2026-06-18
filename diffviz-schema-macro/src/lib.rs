extern crate proc_macro;

use darling::FromField;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, TypePath, parse_macro_input,
};

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
/// Generates templates by introspecting the struct definition and:
/// - Using field examples/comments from #[schema(...)] attributes
/// - Calling nested SchemaTemplate impls for Vec<T> fields
/// - No hardcoding - templates fully derived from data structure
#[proc_macro_derive(SchemaTemplate, attributes(schema))]
pub fn derive_schema_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_template(&input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[derive(Debug, Clone)]
struct FieldInfo {
    name: String,
    type_name: String,
    is_optional: bool,
    is_vec: bool,
    vec_inner_type: Option<TokenStream2>,
    example: Option<String>,
    comment: Option<String>,
}

fn generate_template(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                // Parse all field information
                let mut field_infos = Vec::new();
                for field in &fields.named {
                    let info = extract_field_info(field)?;
                    field_infos.push(info);
                }

                // Generate template code that builds YAML at runtime
                let template_code = build_template_code(&field_infos)?;

                Ok(quote! {
                    impl crate::templates::SchemaTemplate for #name {
                        fn yaml_template() -> String {
                            #template_code
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

fn extract_field_info(field: &syn::Field) -> syn::Result<FieldInfo> {
    let name = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?
        .to_string();

    // Parse #[schema(...)] attribute
    let mut schema_attr = None;
    for attr in &field.attrs {
        if attr.path().is_ident("schema") {
            let parsed: SchemaAttr =
                darling::FromField::from_field(field)
                    .ok()
                    .unwrap_or(SchemaAttr {
                        example: None,
                        comment: None,
                    });
            schema_attr = Some(parsed);
            break;
        }
    }

    // Extract example and comment
    let example = schema_attr.as_ref().and_then(|s| s.example.clone());
    let comment = schema_attr.as_ref().and_then(|s| s.comment.clone());

    // Analyze type
    let is_optional = is_option_type(&field.ty);
    let is_vec = is_vec_type(&field.ty);
    let vec_inner_type = if is_vec {
        extract_vec_inner_type(&field.ty)
    } else {
        None
    };
    let type_name = quote!(#field.ty).to_string();

    Ok(FieldInfo {
        name,
        type_name,
        is_optional,
        is_vec,
        vec_inner_type,
        example,
        comment,
    })
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

fn is_vec_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
    {
        return segment.ident == "Vec";
    }
    false
}

fn extract_vec_inner_type(ty: &Type) -> Option<TokenStream2> {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
        && let PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(GenericArgument::Type(inner)) = args.args.first()
    {
        let tokens = quote!(#inner);
        return Some(tokens);
    }
    None
}

fn get_default_example(field: &FieldInfo) -> String {
    match field.type_name.as_str() {
        t if t.contains("u32") || t.contains("usize") => "1".to_string(),
        t if t.contains("String") => "[placeholder]".to_string(),
        t if t.contains("Vec") => "[]".to_string(),
        _ => "[value]".to_string(),
    }
}

fn build_template_code(fields: &[FieldInfo]) -> syn::Result<TokenStream2> {
    // Preserve struct field order - don't sort
    let mut output_parts = Vec::new();
    output_parts.push(quote! {
        let mut __output = String::new();
    });

    // Generate code for each field in original order
    for field in fields {
        let field_name = &field.name;
        let example = field
            .example
            .clone()
            .unwrap_or_else(|| get_default_example(field));

        if field.is_vec {
            // For Vec fields, call the inner type's yaml_template() if available
            if let Some(inner_type) = &field.vec_inner_type {
                // Generate code that calls the inner type's template
                output_parts.push(quote! {
                    __output.push_str(#field_name);
                    __output.push_str(":\n");

                    // Call the inner type's SchemaTemplate if it implements it
                    let inner_template = <#inner_type as crate::templates::SchemaTemplate>::yaml_template();
                    let mut is_first = true;
                    for line in inner_template.lines() {
                        if is_first {
                            // First line gets the array indicator "- "
                            __output.push_str("  - ");
                            __output.push_str(line);
                            is_first = false;
                        } else {
                            // Subsequent lines get indented normally
                            __output.push_str("    ");
                            __output.push_str(line);
                        }
                        __output.push('\n');
                    }
                });
            } else {
                // Fallback: use the example provided
                output_parts.push(quote! {
                    __output.push_str(#field_name);
                    __output.push_str(": ");
                    __output.push_str(#example);
                    __output.push('\n');
                });
            }
        } else if field.is_optional {
            // Optional field
            let comment_str = field.comment.as_deref().unwrap_or("");
            if comment_str.is_empty() {
                output_parts.push(quote! {
                    __output.push_str(#field_name);
                    __output.push_str(": ");
                    __output.push_str(#example);
                    __output.push_str("  # Optional\n");
                });
            } else {
                output_parts.push(quote! {
                    __output.push_str(#field_name);
                    __output.push_str(": ");
                    __output.push_str(#example);
                    __output.push_str("  # Optional - ");
                    __output.push_str(#comment_str);
                    __output.push('\n');
                });
            }
        } else {
            // Required field
            let comment_str = field.comment.as_deref().unwrap_or("");
            if comment_str.is_empty() {
                output_parts.push(quote! {
                    __output.push_str(#field_name);
                    __output.push_str(": ");
                    __output.push_str(#example);
                    __output.push('\n');
                });
            } else {
                output_parts.push(quote! {
                    __output.push_str(#field_name);
                    __output.push_str(": ");
                    __output.push_str(#example);
                    __output.push_str("  # ");
                    __output.push_str(#comment_str);
                    __output.push('\n');
                });
            }
        }
    }

    output_parts.push(quote! {
        __output
    });

    Ok(quote! {
        {
            #(#output_parts)*
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_compiles() {}
}
