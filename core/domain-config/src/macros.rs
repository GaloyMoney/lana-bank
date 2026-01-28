#[macro_export]
#[doc(hidden)]
macro_rules! __define_config_spec {
    // With default: implements DefaultedConfig
    (
        name: $name:ident,
        key: $key:literal,
        visibility: $visibility:path,
        visibility_marker: $marker:path,
        kind: $kind:ty,
        value_ty: $value_ty:ty,
        default: $default:expr;
        $(validate: $validate:expr;)?
    ) => {
        impl $crate::ConfigSpec for $name {
            const KEY: $crate::DomainConfigKey = $crate::DomainConfigKey::new($key);
            const VISIBILITY: $crate::Visibility = $visibility;
            type Kind = $kind;

            fn default_value() -> Option<$value_ty> { ($default)() }
            $(fn validate(value: &$value_ty) -> Result<(), $crate::DomainConfigError> {
                ($validate)(value)
            })?
        }

        impl $marker for $name {}
        impl $crate::DefaultedConfig for $name {}

        $crate::inventory::submit! {
            $crate::registry::ConfigSpecEntry {
                key: $key,
                visibility: $visibility,
                config_type: <$kind as $crate::ValueKind>::TYPE,
                validate_json: <$name as $crate::ConfigSpec>::validate_json,
            }
        }
    };
    // Without default: does not implement DefaultedConfig
    (
        name: $name:ident,
        key: $key:literal,
        visibility: $visibility:path,
        visibility_marker: $marker:path,
        kind: $kind:ty,
        value_ty: $value_ty:ty,
        $(validate: $validate:expr;)?
    ) => {
        impl $crate::ConfigSpec for $name {
            const KEY: $crate::DomainConfigKey = $crate::DomainConfigKey::new($key);
            const VISIBILITY: $crate::Visibility = $visibility;
            type Kind = $kind;

            $(fn validate(value: &$value_ty) -> Result<(), $crate::DomainConfigError> {
                ($validate)(value)
            })?
        }

        impl $marker for $name {}

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

#[macro_export]
macro_rules! define_exposed_config {
    // With default
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($inner:ty);
        spec {
            key: $key:literal;
            default: $default:expr;
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Exposed,
            visibility_marker: $crate::ExposedConfig,
            kind: $crate::Simple<$inner>,
            value_ty: $inner,
            default: $default;
            $(validate: $validate;)?
        }
    };
    // Without default
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($inner:ty);
        spec {
            key: $key:literal;
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Exposed,
            visibility_marker: $crate::ExposedConfig,
            kind: $crate::Simple<$inner>,
            value_ty: $inner,
            $(validate: $validate;)?
        }
    };
}

#[macro_export]
macro_rules! define_internal_config {
    // Simple type with default
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($inner:ty);
        spec {
            key: $key:literal;
            default: $default:expr;
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Internal,
            visibility_marker: $crate::InternalConfig,
            kind: $crate::Simple<$inner>,
            value_ty: $inner,
            default: $default;
            $(validate: $validate;)?
        }
    };
    // Simple type without default
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($inner:ty);
        spec {
            key: $key:literal;
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Internal,
            visibility_marker: $crate::InternalConfig,
            kind: $crate::Simple<$inner>,
            value_ty: $inner,
            $(validate: $validate;)?
        }
    };
    // Complex type with default
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident { $($body:tt)* }
        spec {
            key: $key:literal;
            default: $default:expr;
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name { $($body)* }

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Internal,
            visibility_marker: $crate::InternalConfig,
            kind: $crate::Complex<$name>,
            value_ty: $name,
            default: $default;
            $(validate: $validate;)?
        }
    };
    // Complex type without default
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident { $($body:tt)* }
        spec {
            key: $key:literal;
            $(validate: $validate:expr;)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name { $($body)* }

        $crate::__define_config_spec! {
            name: $name,
            key: $key,
            visibility: $crate::Visibility::Internal,
            visibility_marker: $crate::InternalConfig,
            kind: $crate::Complex<$name>,
            value_ty: $name,
            $(validate: $validate;)?
        }
    };
}
