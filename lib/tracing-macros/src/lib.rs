use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, ReturnType, Token, Type, parse::Parse, parse::ParseStream, parse_macro_input,
};

/// Arguments for the `observe_error` macro.
///
/// Supported forms:
/// - `#[observe_error]` — inner mode (cap at WARN)
/// - `#[observe_error(allow_single_error_alert)]` — boundary mode (full severity)
/// - `#[observe_error(aggregate = MY_OVERRIDES)]` — inner mode with custom thresholds
/// - `#[observe_error(allow_single_error_alert, aggregate = MY_OVERRIDES)]` — boundary + custom thresholds
struct ObserveErrorArgs {
    allow_single_error_alert: bool,
    aggregate_overrides: Option<Ident>,
}

impl Parse for ObserveErrorArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut allow_single_error_alert = false;
        let mut aggregate_overrides = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            if ident == "allow_single_error_alert" {
                allow_single_error_alert = true;
            } else if ident == "aggregate" {
                let _: Token![=] = input.parse()?;
                aggregate_overrides = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unexpected argument: {ident}"),
                ));
            }
            if !input.is_empty() {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(ObserveErrorArgs {
            allow_single_error_alert,
            aggregate_overrides,
        })
    }
}

/// Observes errors from Result-returning functions with context-aware severity.
///
/// **Default (inner mode)**: Caps severity at WARN, emits aggregate ERROR after sustained failures.
/// **`allow_single_error_alert`**: Emits at full severity (ERROR allowed) for use-case boundaries.
///
/// # Examples
/// ```ignore
/// #[observe_error]  // inner layer — errors capped at WARN
/// #[instrument(name = "repo.find")]
/// async fn find(&self, id: Id) -> Result<T, Error> { ... }
///
/// #[observe_error(allow_single_error_alert)]  // boundary — full severity
/// #[instrument(name = "credit.create")]
/// pub async fn create(&self, ...) -> Result<T, Error> { ... }
/// ```
#[proc_macro_attribute]
pub fn observe_error(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ObserveErrorArgs);
    let function = parse_macro_input!(input as ItemFn);

    // Check if function returns a Result
    let returns_result = match &function.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => is_result_type(ty),
    };

    if !returns_result {
        return TokenStream::from(quote! { #function });
    }

    let fn_vis = &function.vis;
    let fn_sig = &function.sig;
    let fn_body = &function.block;
    let fn_attrs = &function.attrs;
    let is_async = function.sig.asyncness.is_some();
    let fn_name = function.sig.ident.to_string();

    let return_type = match &function.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    let execute_body = if is_async {
        quote! { async move #fn_body.await }
    } else {
        quote! { #fn_body }
    };

    // Build the aggregate threshold lookup
    let aggregate_threshold = if let Some(ref overrides_fn) = args.aggregate_overrides {
        quote! {
            let (__threshold, __window) = #overrides_fn(__variant_name);
        }
    } else {
        quote! {
            let __threshold: u64 = 10;
            let __window: u64 = 600;
        }
    };

    // Build aggregate alert check (common to both modes)
    let aggregate_check = quote! {
        let __variant_name = {
            use ::tracing_utils::ErrorSeverity;
            __e.variant_name()
        };
        #aggregate_threshold
        let __agg_key = format!("{}::{}::{}", module_path!(), #fn_name, __variant_name);
        if ::tracing_utils::rate_tracker::should_trigger_aggregate_alert(&__agg_key, __threshold, __window) {
            ::tracing::event!(
                ::tracing::Level::ERROR,
                error = &::tracing::field::display(__e),
                error.aggregate = true,
                error.variant = __variant_name,
            );
        }
    };

    // Build the primary error event emission
    let error_event = if args.allow_single_error_alert {
        // Boundary mode: emit at full severity
        quote! {
            use ::tracing_utils::ErrorSeverity;
            let __severity = __e.severity();
            match __severity {
                ::tracing::Level::ERROR => {
                    ::tracing::event!(::tracing::Level::ERROR, error = &::tracing::field::display(__e), error.boundary = true, error.use_case = #fn_name);
                }
                ::tracing::Level::WARN => {
                    ::tracing::event!(::tracing::Level::WARN, error = &::tracing::field::display(__e), error.boundary = true, error.use_case = #fn_name);
                }
                ::tracing::Level::INFO => {
                    ::tracing::event!(::tracing::Level::INFO, error = &::tracing::field::display(__e), error.boundary = true, error.use_case = #fn_name);
                }
                ::tracing::Level::DEBUG => {
                    ::tracing::event!(::tracing::Level::DEBUG, error = &::tracing::field::display(__e), error.boundary = true, error.use_case = #fn_name);
                }
                ::tracing::Level::TRACE => {
                    ::tracing::event!(::tracing::Level::TRACE, error = &::tracing::field::display(__e), error.boundary = true, error.use_case = #fn_name);
                }
            }
        }
    } else {
        // Inner mode: cap severity at WARN (ERROR→WARN)
        quote! {
            use ::tracing_utils::ErrorSeverity;
            let __severity = __e.severity();
            let __capped = if __severity == ::tracing::Level::ERROR {
                ::tracing::Level::WARN
            } else {
                __severity
            };
            match __capped {
                ::tracing::Level::WARN => {
                    ::tracing::event!(::tracing::Level::WARN, error = &::tracing::field::display(__e), error.layer = "inner");
                }
                ::tracing::Level::INFO => {
                    ::tracing::event!(::tracing::Level::INFO, error = &::tracing::field::display(__e), error.layer = "inner");
                }
                ::tracing::Level::DEBUG => {
                    ::tracing::event!(::tracing::Level::DEBUG, error = &::tracing::field::display(__e), error.layer = "inner");
                }
                ::tracing::Level::TRACE => {
                    ::tracing::event!(::tracing::Level::TRACE, error = &::tracing::field::display(__e), error.layer = "inner");
                }
                _ => {
                    // ERROR is capped to WARN above, this shouldn't happen
                    ::tracing::event!(::tracing::Level::WARN, error = &::tracing::field::display(__e), error.layer = "inner");
                }
            }
        }
    };

    let new_body = quote! {
        {
            let __result: #return_type = #execute_body;

            if let Err(ref __e) = __result {
                #error_event
                #aggregate_check
            }

            __result
        }
    };

    let result = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig #new_body
    };

    TokenStream::from(result)
}

#[proc_macro_attribute]
pub fn record_error_severity(_args: TokenStream, input: TokenStream) -> TokenStream {
    let function = parse_macro_input!(input as ItemFn);

    // Check if function returns a Result
    let returns_result = match &function.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => is_result_type(ty),
    };

    if !returns_result {
        // If not returning Result, just return the original function
        return TokenStream::from(quote! { #function });
    }

    // Extract function components
    let fn_vis = &function.vis;
    let fn_sig = &function.sig;
    let fn_body = &function.block;
    let fn_attrs = &function.attrs;
    let is_async = function.sig.asyncness.is_some();

    // Extract the return type from the function signature
    let return_type = match &function.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    // Create the execution part based on whether the function is async
    let execute_body = if is_async {
        quote! { async move #fn_body.await }
    } else {
        quote! { #fn_body }
    };

    // Create the new function body that wraps the original
    let new_body = quote! {
        {
            // Execute the original function body directly
            let __result: #return_type = #execute_body;

            // Record error severity if it's an Err
            if let Err(ref __e) = __result {
                use tracing_utils::ErrorSeverity;

                let __severity = __e.severity();

                // Emit event at appropriate level with "error" field for OpenTelemetry
                match __severity {
                    ::tracing::Level::ERROR => {
                        ::tracing::event!(::tracing::Level::ERROR, error = &::tracing::field::display(__e));
                    }
                    ::tracing::Level::WARN => {
                        ::tracing::event!(::tracing::Level::WARN, error = &::tracing::field::display(__e));
                    }
                    ::tracing::Level::INFO => {
                        ::tracing::event!(::tracing::Level::INFO, error = &::tracing::field::display(__e));
                    }
                    ::tracing::Level::DEBUG => {
                        ::tracing::event!(::tracing::Level::DEBUG, error = &::tracing::field::display(__e));
                    }
                    ::tracing::Level::TRACE => {
                        ::tracing::event!(::tracing::Level::TRACE, error = &::tracing::field::display(__e));
                    }
                }
            }

            __result
        }
    };

    // Generate the complete wrapped function
    let result = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig #new_body
    };

    TokenStream::from(result)
}

// Helper function to check if a type is Result<T, E>
fn is_result_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident == "Result"
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Generates an aggregate override function for use with `#[observe_error(aggregate = ...)]`.
///
/// # Syntax
/// ```ignore
/// aggregate_overrides!(my_overrides {
///     VariantA => (5, 300),
///     VariantB => disabled,
/// });
/// ```
///
/// Generates `fn my_overrides(variant: &str) -> (u64, u64)` where disabled maps to `(0, 0)`.
#[proc_macro]
pub fn aggregate_overrides(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as AggregateOverridesDef);

    let fn_name = &def.name;
    let mut arms = Vec::new();
    for entry in &def.entries {
        let variant_str = entry.variant.to_string();
        let tokens = if entry.disabled {
            quote! { #variant_str => (0, 0), }
        } else {
            let threshold = entry.threshold;
            let window = entry.window;
            quote! { #variant_str => (#threshold, #window), }
        };
        arms.push(tokens);
    }

    let result = quote! {
        fn #fn_name(variant: &str) -> (u64, u64) {
            match variant {
                #(#arms)*
                _ => (10, 600),
            }
        }
    };

    TokenStream::from(result)
}

struct AggregateOverridesDef {
    name: Ident,
    entries: Vec<OverrideEntry>,
}

struct OverrideEntry {
    variant: Ident,
    disabled: bool,
    threshold: u64,
    window: u64,
}

impl Parse for AggregateOverridesDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        syn::braced!(content in input);

        let mut entries = Vec::new();
        while !content.is_empty() {
            let variant: Ident = content.parse()?;
            let _: Token![=>] = content.parse()?;

            let lookahead = content.lookahead1();
            let entry = if lookahead.peek(Ident) {
                let ident: Ident = content.parse()?;
                if ident != "disabled" {
                    return Err(syn::Error::new(
                        ident.span(),
                        "expected `disabled` or `(threshold, window)`",
                    ));
                }
                OverrideEntry {
                    variant,
                    disabled: true,
                    threshold: 0,
                    window: 0,
                }
            } else {
                let tuple_content;
                syn::parenthesized!(tuple_content in content);
                let threshold: syn::LitInt = tuple_content.parse()?;
                let _: Token![,] = tuple_content.parse()?;
                let window: syn::LitInt = tuple_content.parse()?;
                OverrideEntry {
                    variant,
                    disabled: false,
                    threshold: threshold.base10_parse()?,
                    window: window.base10_parse()?,
                }
            };
            entries.push(entry);
            if !content.is_empty() {
                let _: Token![,] = content.parse()?;
            }
        }

        Ok(AggregateOverridesDef { name, entries })
    }
}
