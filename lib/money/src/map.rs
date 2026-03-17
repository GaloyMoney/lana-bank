use std::collections::{HashMap, HashSet, hash_map};
use std::ops::Index;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::code::CurrencyCode;

// ---------------------------------------------------------------------------
// CurrencySet — typed set of allowed currencies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CurrencySet {
    inner: HashSet<CurrencyCode>,
}

impl CurrencySet {
    pub fn new(currencies: impl IntoIterator<Item = CurrencyCode>) -> Self {
        Self {
            inner: currencies.into_iter().collect(),
        }
    }

    pub fn contains(&self, currency: &CurrencyCode) -> bool {
        self.inner.contains(currency)
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, CurrencyCode> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum CurrencyMapError {
    #[error("CurrencyMapError - CurrencyNotAllowed: {0}")]
    CurrencyNotAllowed(CurrencyCode),
}

// ---------------------------------------------------------------------------
// CurrencyMap<V>
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CurrencyMap<V> {
    inner: HashMap<CurrencyCode, V>,
}

impl<V> CurrencyMap<V> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, currency: CurrencyCode, value: V) -> Option<V> {
        self.inner.insert(currency, value)
    }

    pub fn get(&self, currency: &CurrencyCode) -> Option<&V> {
        self.inner.get(currency)
    }

    pub fn get_mut(&mut self, currency: &CurrencyCode) -> Option<&mut V> {
        self.inner.get_mut(currency)
    }

    pub fn remove(&mut self, currency: &CurrencyCode) -> Option<V> {
        self.inner.remove(currency)
    }

    pub fn contains_key(&self, currency: &CurrencyCode) -> bool {
        self.inner.contains_key(currency)
    }

    pub fn iter(&self) -> hash_map::Iter<'_, CurrencyCode, V> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> hash_map::IterMut<'_, CurrencyCode, V> {
        self.inner.iter_mut()
    }

    pub fn keys(&self) -> hash_map::Keys<'_, CurrencyCode, V> {
        self.inner.keys()
    }

    pub fn values(&self) -> hash_map::Values<'_, CurrencyCode, V> {
        self.inner.values()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// --- Default ---

impl<V> Default for CurrencyMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

// --- Index ---

impl<V> Index<&CurrencyCode> for CurrencyMap<V> {
    type Output = V;

    fn index(&self, currency: &CurrencyCode) -> &V {
        &self.inner[currency]
    }
}

// --- IntoIterator ---

impl<V> IntoIterator for CurrencyMap<V> {
    type Item = (CurrencyCode, V);
    type IntoIter = hash_map::IntoIter<CurrencyCode, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, V> IntoIterator for &'a CurrencyMap<V> {
    type Item = (&'a CurrencyCode, &'a V);
    type IntoIter = hash_map::Iter<'a, CurrencyCode, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

// --- FromIterator ---

impl<V> FromIterator<(CurrencyCode, V)> for CurrencyMap<V> {
    fn from_iter<I: IntoIterator<Item = (CurrencyCode, V)>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

// --- Extend ---

impl<V> Extend<(CurrencyCode, V)> for CurrencyMap<V> {
    fn extend<I: IntoIterator<Item = (CurrencyCode, V)>>(&mut self, iter: I) {
        self.inner.extend(iter);
    }
}

// --- From<HashMap> ---

impl<V> From<HashMap<CurrencyCode, V>> for CurrencyMap<V> {
    fn from(inner: HashMap<CurrencyCode, V>) -> Self {
        Self { inner }
    }
}

// ---------------------------------------------------------------------------
// RestrictedCurrencyMap<V>
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestrictedCurrencyMap<V> {
    allowed: CurrencySet,
    inner: CurrencyMap<V>,
}

impl<V> RestrictedCurrencyMap<V> {
    pub fn new(allowed: impl IntoIterator<Item = CurrencyCode>) -> Self {
        Self {
            allowed: CurrencySet::new(allowed),
            inner: CurrencyMap::new(),
        }
    }

    pub fn allowed_currencies(&self) -> &CurrencySet {
        &self.allowed
    }

    fn assert_allowed(&self, currency: &CurrencyCode) -> Result<(), CurrencyMapError> {
        if self.allowed.contains(currency) {
            Ok(())
        } else {
            Err(CurrencyMapError::CurrencyNotAllowed(*currency))
        }
    }

    pub fn insert(
        &mut self,
        currency: CurrencyCode,
        value: V,
    ) -> Result<Option<V>, CurrencyMapError> {
        self.assert_allowed(&currency)?;
        Ok(self.inner.insert(currency, value))
    }

    pub fn get(&self, currency: &CurrencyCode) -> Result<Option<&V>, CurrencyMapError> {
        self.assert_allowed(currency)?;
        Ok(self.inner.get(currency))
    }

    pub fn get_mut(&mut self, currency: &CurrencyCode) -> Result<Option<&mut V>, CurrencyMapError> {
        self.assert_allowed(currency)?;
        Ok(self.inner.get_mut(currency))
    }

    pub fn remove(&mut self, currency: &CurrencyCode) -> Result<Option<V>, CurrencyMapError> {
        self.assert_allowed(currency)?;
        Ok(self.inner.remove(currency))
    }

    pub fn contains_key(&self, currency: &CurrencyCode) -> Result<bool, CurrencyMapError> {
        self.assert_allowed(currency)?;
        Ok(self.inner.contains_key(currency))
    }

    pub fn iter(&self) -> hash_map::Iter<'_, CurrencyCode, V> {
        self.inner.iter()
    }

    pub fn keys(&self) -> hash_map::Keys<'_, CurrencyCode, V> {
        self.inner.keys()
    }

    pub fn values(&self) -> hash_map::Values<'_, CurrencyCode, V> {
        self.inner.values()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
