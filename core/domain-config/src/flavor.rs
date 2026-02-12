use es_entity::Idempotent;

use crate::{
    ConfigSpec, DomainConfigError, EncryptionConfig, TypedDomainConfig, ValueKind,
    entity::DomainConfig,
};

/// Marker type for plaintext (unencrypted) domain configs.
/// This is the default flavor for all configs.
pub struct DomainConfigFlavorPlaintext;

/// Marker type for encrypted domain configs.
/// Configs with this flavor will have their values encrypted at rest.
pub struct DomainConfigFlavorEncrypted;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::DomainConfigFlavorPlaintext {}
    impl Sealed for super::DomainConfigFlavorEncrypted {}
}

/// Trait for domain config flavors (plaintext or encrypted).
pub trait ConfigFlavor: sealed::Sealed {
    const IS_ENCRYPTED: bool;
}

impl ConfigFlavor for DomainConfigFlavorPlaintext {
    const IS_ENCRYPTED: bool = false;
}

impl ConfigFlavor for DomainConfigFlavorEncrypted {
    const IS_ENCRYPTED: bool = true;
}

/// Trait for compile-time dispatch based on config flavor.
///
/// This enables unified method signatures in the public API while routing
/// to flavor-specific implementations internally. All methods that differ
/// between plaintext and encrypted configs are dispatched through this trait.
pub trait FlavorDispatch: sealed::Sealed {
    // === TypedDomainConfig operations ===

    fn try_new<C: ConfigSpec<Flavor = Self>>(
        entity: DomainConfig,
        config: &EncryptionConfig,
    ) -> Result<TypedDomainConfig<C>, DomainConfigError>;

    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        typed_config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value>;

    // === Entity operations ===

    fn update_value<C: ConfigSpec<Flavor = Self>>(
        entity: &mut DomainConfig,
        config: &EncryptionConfig,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError>;
}

impl FlavorDispatch for DomainConfigFlavorPlaintext {
    fn try_new<C: ConfigSpec<Flavor = Self>>(
        entity: DomainConfig,
        _config: &EncryptionConfig,
    ) -> Result<TypedDomainConfig<C>, DomainConfigError> {
        TypedDomainConfig::try_new_plain(entity)
    }

    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        typed_config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value> {
        typed_config.maybe_value_plain()
    }

    fn update_value<C: ConfigSpec<Flavor = Self>>(
        entity: &mut DomainConfig,
        _config: &EncryptionConfig,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        entity.update_value_plain::<C>(value)
    }
}

impl FlavorDispatch for DomainConfigFlavorEncrypted {
    fn try_new<C: ConfigSpec<Flavor = Self>>(
        entity: DomainConfig,
        config: &EncryptionConfig,
    ) -> Result<TypedDomainConfig<C>, DomainConfigError> {
        TypedDomainConfig::try_new_encrypted(entity, config.key)
    }

    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        typed_config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value> {
        typed_config.maybe_value_encrypted()
    }

    fn update_value<C: ConfigSpec<Flavor = Self>>(
        entity: &mut DomainConfig,
        config: &EncryptionConfig,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        entity.update_value_encrypted::<C>(&config.key, value)
    }
}
