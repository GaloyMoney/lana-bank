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
        encrypted: $encrypted:expr,
        default: $default:expr;
        $(validate: $validate:expr;)?
    ) => {
        impl $crate::ConfigSpec for $name {
            const KEY: $crate::DomainConfigKey = $crate::DomainConfigKey::new($key);
            const VISIBILITY: $crate::Visibility = $visibility;
            const ENCRYPTED: bool = $encrypted;
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
                encrypted: $encrypted,
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
        encrypted: $encrypted:expr,
        $(validate: $validate:expr;)?
    ) => {
        impl $crate::ConfigSpec for $name {
            const KEY: $crate::DomainConfigKey = $crate::DomainConfigKey::new($key);
            const VISIBILITY: $crate::Visibility = $visibility;
            const ENCRYPTED: bool = $encrypted;
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
                encrypted: $encrypted,
                validate_json: <$name as $crate::ConfigSpec>::validate_json,
            }
        }
    };
}

/// Helper: peels off `encrypted: <lit>;` if present, defaults to `false`, then forwards.
#[macro_export]
#[doc(hidden)]
macro_rules! __with_encrypted {
    // encrypted: present
    (
        [$($prefix:tt)*]
        encrypted: $encrypted:literal; $($rest:tt)*
    ) => {
        $crate::__define_config_spec! {
            $($prefix)*
            encrypted: $encrypted,
            $($rest)*
        }
    };
    // encrypted: absent — default to false
    (
        [$($prefix:tt)*]
        $($rest:tt)*
    ) => {
        $crate::__define_config_spec! {
            $($prefix)*
            encrypted: false,
            $($rest)*
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
            $($spec_rest:tt)*
        }
    ) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        $crate::__with_encrypted! {
            [
                name: $name,
                key: $key,
                visibility: $crate::Visibility::Exposed,
                visibility_marker: $crate::ExposedConfig,
                kind: $crate::Simple<$inner>,
                value_ty: $inner,
            ]
            $($spec_rest)*
        }
    };
}

#[macro_export]
macro_rules! define_internal_config {
    // Simple type
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($inner:ty);
        spec {
            key: $key:literal;
            $($spec_rest:tt)*
        }
    ) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        $crate::__with_encrypted! {
            [
                name: $name,
                key: $key,
                visibility: $crate::Visibility::Internal,
                visibility_marker: $crate::InternalConfig,
                kind: $crate::Simple<$inner>,
                value_ty: $inner,
            ]
            $($spec_rest)*
        }
    };
    // Complex type
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident { $($body:tt)* }
        spec {
            key: $key:literal;
            $($spec_rest:tt)*
        }
    ) => {
        $(#[$meta])*
        $vis struct $name { $($body)* }

        $crate::__with_encrypted! {
            [
                name: $name,
                key: $key,
                visibility: $crate::Visibility::Internal,
                visibility_marker: $crate::InternalConfig,
                kind: $crate::Complex<$name>,
                value_ty: $name,
            ]
            $($spec_rest)*
        }
    };
}
