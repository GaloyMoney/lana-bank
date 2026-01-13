//! Macros for defining domain configurations.
//!
//! See the crate-level documentation for usage examples.

#[macro_export]
#[doc(hidden)]
macro_rules! __define_config_spec {
    (
        name: $name:ident,
        key: $key:literal,
        visibility: $visibility:path,
        kind: $kind:ty,
        value_ty: $value_ty:ty,
        $(default: $default:expr;)?
        $(validate: $validate:expr;)?
    ) => {
        impl $crate::ConfigSpec for $name {
            const KEY: $crate::DomainConfigKey = $crate::DomainConfigKey::new($key);
            const VISIBILITY: $crate::Visibility = $visibility;
            type Kind = $kind;

            $(fn default_value() -> Option<$value_ty> { ($default)() })?
            $(fn validate(value: &$value_ty) -> Result<(), $crate::DomainConfigError> {
                ($validate)(value)
            })?
        }

        $crate::inventory::submit! {
            $crate::registry::ConfigSpecEntry {
                key: $key,
                visibility: $visibility,
                config_type: <$kind as $crate::ValueKind>::TYPE,
                validate_json: <$name as $crate::ConfigSpec>::validate_json,
            }
        }
    };
}

/// Define an exposed configuration (modifiable via API/UI).
///
/// Creates a tuple struct wrapping a simple type with `Visibility::Exposed`.
/// Only supports simple types: `bool`, `String`, `i64`, `u64`, `Decimal`.
///
/// # Example
/// ```ignore
/// define_exposed_config! {
///     pub struct NotificationEmail(String);
///     spec {
///         key: "notification-email";
///         validate: |value: &String| {
///             if value.is_empty() {
///                 return Err(DomainConfigError::InvalidState("required".into()));
///             }
///             Ok(())
///         };
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_exposed_config {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($inner:ty);
        spec {
            key: $key:literal;
            $(default: $default:expr;)?
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Exposed,
            kind: $crate::Simple<$inner>,
            value_ty: $inner,
            $(default: $default;)?
            $(validate: $validate;)?
        }
    };
}

/// Define an internal configuration (programmatic access only).
///
/// Supports two forms:
///
/// **Simple form** - tuple struct with a simple type:
/// ```ignore
/// define_internal_config! {
///     pub struct MaxRetries(u64);
///     spec {
///         key: "max-retries";
///         default: || Some(3);
///     }
/// }
/// ```
///
/// **Complex form** - struct with multiple fields (requires `Serialize + Deserialize`):
/// ```ignore
/// define_internal_config! {
///     #[derive(Clone, Serialize, Deserialize)]
///     pub struct FiscalYearConfig {
///         pub revenue_account: String,
///         pub expense_account: String,
///     }
///     spec {
///         key: "fiscal-year";
///         validate: |value: &Self| { /* ... */ Ok(()) };
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_internal_config {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($inner:ty);
        spec {
            key: $key:literal;
            $(default: $default:expr;)?
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Internal,
            kind: $crate::Simple<$inner>,
            value_ty: $inner,
            $(default: $default;)?
            $(validate: $validate;)?
        }
    };
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident { $($body:tt)* }
        spec {
            key: $key:literal;
            $(default: $default:expr;)?
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name { $($body)* }

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Internal,
            kind: $crate::Complex<$name>,
            value_ty: $name,
            $(default: $default;)?
            $(validate: $validate;)?
        }
    };
}
