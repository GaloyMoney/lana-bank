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
    let _fn_ident = &function.sig.ident;
    let _fn_inputs = &function.sig.inputs;
    let _fn_output = &function.sig.output;
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
                // Execute the original function body directly
                let __result: #return_type = async move #fn_body.await;

                // Record error severity if it's an Err
                if let Err(ref __e) = __result {
                    use tracing_utils::ErrorSeverity;

                    let __span = ::tracing::Span::current();
                    let __severity = __e.severity();

                    // Record error metadata on current span
                    __span.record("error", &::tracing::field::display(__e));
                    __span.record("error.level", __severity.as_str());
                    __span.record("error.message", &::tracing::field::display(__e));

                    // Emit event at appropriate level
                    match __severity {
                        ::tracing::Level::ERROR => {
                            ::tracing::event!(::tracing::Level::ERROR, error = %__e, "Operation failed");
                        }
                        ::tracing::Level::WARN => {
                            ::tracing::event!(::tracing::Level::WARN, error = %__e, "Operation warning");
                        }
                        ::tracing::Level::INFO => {
                            ::tracing::event!(::tracing::Level::INFO, error = %__e, "Operation notice");
                        }
                        ::tracing::Level::DEBUG => {
                            ::tracing::event!(::tracing::Level::DEBUG, error = %__e, "Operation debug");
                        }
                        ::tracing::Level::TRACE => {
                            ::tracing::event!(::tracing::Level::TRACE, error = %__e, "Operation trace");
                        }
                    }
                }

                __result
            }
        }
    } else {
        quote! {
            {
                // Execute the original function body directly
                let __result: #return_type = #fn_body;

                // Record error severity if it's an Err
                if let Err(ref __e) = __result {
                    use tracing_utils::ErrorSeverity;

                    let __span = ::tracing::Span::current();
                    let __severity = __e.severity();

                    // Record error metadata on current span
                    __span.record("error", &::tracing::field::display(__e));
                    __span.record("error.level", __severity.as_str());
                    __span.record("error.message", &::tracing::field::display(__e));

                    // Emit event at appropriate level
                    match __severity {
                        ::tracing::Level::ERROR => {
                            ::tracing::event!(::tracing::Level::ERROR, error = %__e, "Operation failed");
                        }
                        ::tracing::Level::WARN => {
                            ::tracing::event!(::tracing::Level::WARN, error = %__e, "Operation warning");
                        }
                        ::tracing::Level::INFO => {
                            ::tracing::event!(::tracing::Level::INFO, error = %__e, "Operation notice");
                        }
                        ::tracing::Level::DEBUG => {
                            ::tracing::event!(::tracing::Level::DEBUG, error = %__e, "Operation debug");
                        }
                        ::tracing::Level::TRACE => {
                            ::tracing::event!(::tracing::Level::TRACE, error = %__e, "Operation trace");
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
        _ => false,
    }
}
