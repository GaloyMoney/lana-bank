use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type};

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

    // Create the new function body that wraps the original
    let new_body = if is_async {
        quote! {
            {
                async move {
                    let __result: #return_type = #fn_body;

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
                }.await
            }
        }
    } else {
        quote! {
            {
                let __result: #return_type = #fn_body;

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
        Type::TraitObject(_) => {
            // Handle trait object types like BoxStream wrapped in Result
            false
        }
        _ => false,
    }
}
