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
///
/// This trait is sealed - only `DomainConfigFlavorPlaintext` and
/// `DomainConfigFlavorEncrypted` implement it.
pub trait ConfigFlavor: sealed::Sealed {
    const IS_ENCRYPTED: bool;

    #[doc(hidden)]
    fn try_new<C: ConfigSpec<Flavor = Self>>(
        entity: DomainConfig,
        config: &EncryptionConfig,
    ) -> Result<TypedDomainConfig<C>, DomainConfigError>;

    #[doc(hidden)]
    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        typed_config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value>;

    #[doc(hidden)]
    fn update_value<C: ConfigSpec<Flavor = Self>>(
        entity: &mut DomainConfig,
        config: &EncryptionConfig,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError>;
}

impl ConfigFlavor for DomainConfigFlavorPlaintext {
    const IS_ENCRYPTED: bool = false;

    fn try_new<C: ConfigSpec<Flavor = Self>>(
        entity: DomainConfig,
        _config: &EncryptionConfig,
    ) -> Result<TypedDomainConfig<C>, DomainConfigError> {
        TypedDomainConfig::try_new_plain(entity)
    }

    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        typed_config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value> {
        typed_config
            .entity
            .current_value_plain::<C>()
            .or_else(C::default_value)
    }

    fn update_value<C: ConfigSpec<Flavor = Self>>(
        entity: &mut DomainConfig,
        _config: &EncryptionConfig,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        entity.update_value_plain::<C>(value)
    }
}

impl ConfigFlavor for DomainConfigFlavorEncrypted {
    const IS_ENCRYPTED: bool = true;

    fn try_new<C: ConfigSpec<Flavor = Self>>(
        entity: DomainConfig,
        config: &EncryptionConfig,
    ) -> Result<TypedDomainConfig<C>, DomainConfigError> {
        TypedDomainConfig::try_new_encrypted(entity, config.key, config.key_id.clone())
    }

    fn maybe_value<C: ConfigSpec<Flavor = Self>>(
        typed_config: &TypedDomainConfig<C>,
    ) -> Option<<C::Kind as ValueKind>::Value> {
        let key = typed_config.encryption_key.as_ref()?;
        let key_id = typed_config.encryption_key_id.as_ref()?;
        typed_config
            .entity
            .current_value_encrypted::<C>(key, key_id)
            .or_else(C::default_value)
    }

    fn update_value<C: ConfigSpec<Flavor = Self>>(
        entity: &mut DomainConfig,
        config: &EncryptionConfig,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<Idempotent<()>, DomainConfigError> {
        entity.update_value_encrypted::<C>(&config.key, &config.key_id, value)
    }
}
