use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Lit, Meta};

/// Derive macro for generating Avro schemas from tagged enums with struct variants.
///
/// This macro generates an `AvroSchema` implementation for enums that use
/// `#[serde(tag = "type")]` serialization. It creates a flattened record schema
/// where all variant fields are nullable (since each event only has its own fields).
///
/// # Attributes
///
/// - `#[avro(namespace = "com.example")]` - Sets the Avro namespace for the schema
///
/// # Example
///
/// ```ignore
/// use avro_derive::AvroEventSchema;
///
/// #[derive(AvroEventSchema)]
/// #[avro(namespace = "lana.core.deposit")]
/// #[serde(tag = "type")]
/// pub enum CoreDepositEvent {
///     DepositAccountCreated {
///         id: DepositAccountId,
///         account_holder_id: DepositAccountHolderId,
///     },
///     DepositInitialized {
///         id: DepositId,
///         deposit_account_id: DepositAccountId,
///         amount: UsdCents,
///     },
/// }
/// ```
#[proc_macro_derive(AvroEventSchema, attributes(avro))]
pub fn derive_avro_event_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let name_str = name.to_string();

    // Extract namespace from #[avro(namespace = "...")] attribute
    let namespace = extract_namespace(&input.attrs).unwrap_or_default();
    let full_name = if namespace.is_empty() {
        name_str.clone()
    } else {
        format!("{}.{}", namespace, name_str)
    };
    let full_enum_name = format!("{}Type", full_name);

    // Get enum variants
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new_spanned(input, "AvroEventSchema only works on enums")
                .to_compile_error()
                .into();
        }
    };

    // Collect variant names for the enum schema
    let variant_names: Vec<String> = variants.iter().map(|v| v.ident.to_string()).collect();

    // Collect all unique fields across all variants with their types
    let mut all_fields: Vec<(String, syn::Type)> = Vec::new();
    let mut seen_fields: std::collections::HashSet<String> = std::collections::HashSet::new();

    for variant in variants.iter() {
        if let Fields::Named(fields) = &variant.fields {
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap().to_string();
                if !seen_fields.contains(&field_name) {
                    seen_fields.insert(field_name.clone());
                    all_fields.push((field_name, field.ty.clone()));
                }
            }
        }
    }

    // Generate field construction code
    let field_constructions: Vec<_> = all_fields
        .iter()
        .enumerate()
        .map(|(idx, (field_name, field_type))| {
            let position = idx + 1; // +1 because type field is at position 0
            quote! {
                {
                    let inner_schema = <#field_type as ::apache_avro::AvroSchema>::get_schema();
                    let nullable = ::apache_avro::schema::UnionSchema::new(
                        vec![::apache_avro::Schema::Null, inner_schema]
                    ).expect("valid union schema");

                    ::apache_avro::schema::RecordField {
                        name: #field_name.to_string(),
                        doc: None,
                        aliases: None,
                        default: Some(::serde_json::Value::Null),
                        schema: ::apache_avro::Schema::Union(nullable),
                        order: ::apache_avro::schema::RecordFieldOrder::Ascending,
                        position: #position,
                        custom_attributes: ::std::collections::BTreeMap::new(),
                    }
                }
            }
        })
        .collect();

    // Generate the implementation
    let expanded = quote! {
        impl ::apache_avro::AvroSchema for #name {
            fn get_schema() -> ::apache_avro::Schema {
                use ::std::collections::BTreeMap;

                // Build enum schema for event type discriminator
                let event_type_enum = ::apache_avro::Schema::Enum(::apache_avro::schema::EnumSchema {
                    name: ::apache_avro::schema::Name::new(#full_enum_name).expect("valid name"),
                    aliases: None,
                    doc: None,
                    symbols: vec![#(#variant_names.to_string()),*],
                    default: None,
                    attributes: BTreeMap::new(),
                });

                // Type discriminator field (required, not nullable)
                let type_field = ::apache_avro::schema::RecordField {
                    name: "type".to_string(),
                    doc: None,
                    aliases: None,
                    default: None,
                    schema: event_type_enum,
                    order: ::apache_avro::schema::RecordFieldOrder::Ascending,
                    position: 0,
                    custom_attributes: BTreeMap::new(),
                };

                // All variant fields (nullable)
                let mut fields = vec![type_field];
                #(fields.push(#field_constructions);)*

                // Build lookup map
                let lookup: BTreeMap<String, usize> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (f.name.clone(), i))
                    .collect();

                // Build the record schema
                ::apache_avro::Schema::Record(::apache_avro::schema::RecordSchema {
                    name: ::apache_avro::schema::Name::new(#full_name).expect("valid name"),
                    aliases: None,
                    doc: None,
                    fields,
                    lookup,
                    attributes: BTreeMap::new(),
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_namespace(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("avro") {
            if let Meta::List(meta_list) = &attr.meta {
                // Parse the tokens inside avro(...)
                let nested: syn::Result<Meta> = meta_list.parse_args();
                if let Ok(Meta::NameValue(nv)) = nested {
                    if nv.path.is_ident("namespace") {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                return Some(lit_str.value());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
