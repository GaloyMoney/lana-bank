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
