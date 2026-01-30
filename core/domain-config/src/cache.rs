use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{DomainConfig, DomainConfigKey};

#[derive(Clone, Default)]
pub struct DomainConfigCache {
    inner: Arc<RwLock<HashMap<DomainConfigKey, DomainConfig>>>,
}

impl DomainConfigCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, key: &DomainConfigKey) -> Option<DomainConfig> {
        let guard = self.inner.read().await;
        guard.get(key).cloned()
    }

    pub async fn insert(&self, key: DomainConfigKey, config: DomainConfig) {
        let mut guard = self.inner.write().await;
        guard.insert(key, config);
    }

    pub async fn invalidate(&self, key: &DomainConfigKey) {
        let mut guard = self.inner.write().await;
        guard.remove(key);
    }

    pub async fn clear(&self) {
        let mut guard = self.inner.write().await;
        guard.clear();
    }
}

#[cfg(test)]
mod tests {
    use es_entity::{IntoEvents as _, TryFromEvents as _};

    use crate::{ConfigType, DomainConfigId, Visibility, entity::NewDomainConfig};

    use super::*;

    fn create_test_config(key: &'static str) -> DomainConfig {
        let id = DomainConfigId::new();
        let events = NewDomainConfig::builder()
            .seed(
                id,
                DomainConfigKey::new(key),
                ConfigType::Bool,
                Visibility::Internal,
            )
            .build()
            .unwrap()
            .into_events();
        DomainConfig::try_from_events(events).unwrap()
    }

    #[tokio::test]
    async fn get_returns_none_for_missing_key() {
        let cache = DomainConfigCache::new();
        let key = DomainConfigKey::new("missing-key");

        let result = cache.get(&key).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn insert_then_get_returns_value() {
        let cache = DomainConfigCache::new();
        let key = DomainConfigKey::new("test-key");
        let config = create_test_config("test-key");

        cache.insert(key.clone(), config.clone()).await;
        let result = cache.get(&key).await;

        assert!(result.is_some());
        assert_eq!(result.unwrap().id, config.id);
    }

    #[tokio::test]
    async fn invalidate_removes_entry() {
        let cache = DomainConfigCache::new();
        let key = DomainConfigKey::new("test-key");
        let config = create_test_config("test-key");

        cache.insert(key.clone(), config).await;
        assert!(cache.get(&key).await.is_some());

        cache.invalidate(&key).await;
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn clear_removes_all_entries() {
        let cache = DomainConfigCache::new();
        let key1 = DomainConfigKey::new("key1");
        let key2 = DomainConfigKey::new("key2");
        let config1 = create_test_config("key1");
        let config2 = create_test_config("key2");

        cache.insert(key1.clone(), config1).await;
        cache.insert(key2.clone(), config2).await;
        assert!(cache.get(&key1).await.is_some());
        assert!(cache.get(&key2).await.is_some());

        cache.clear().await;
        assert!(cache.get(&key1).await.is_none());
        assert!(cache.get(&key2).await.is_none());
    }
}
