#![no_std]
#![cfg_attr(feature = "nightly", feature(negative_impls, auto_traits))]
#![feature(test)]
extern crate test;

#[deny(missing_docs)]

extern crate alloc;
#[cfg(feature = "hashbrown")]
extern crate hashbrown;

#[cfg(any(test, not(feature = "hashbrown")))]
extern crate std;




mod lru;
mod raw;

#[macro_use]
mod macros;
mod adaptive;
mod two_queue;


// pub use raw::{
//     IntoIter, Iter, IterMut, Keys, RawLRU, ReversedIter, ReversedIterMut, ReversedKeys,
//     ReversedValues, ReversedValuesMut, Values, ValuesMut,
// };

pub use raw::{
    Iter, IterMut, RawLRU
};

// pub use two_queue::{
//     TwoQueueCache,
//     DEFAULT_2Q_RECENT_RATIO,
//     DEFAULT_2Q_GHOST_RATIO,
// };

pub use lru::LRUCache;

use core::fmt::{Debug, Display, Formatter};

#[cfg(feature = "hashbrown")]
pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;

#[cfg(not(feature = "hashbrown"))]
pub type DefaultHashBuilder = std::collections::hash_map::DefaultHasher;

/// `DefaultEvictCallback` is a noop evict callback.
#[derive(Debug, Clone, Copy)]
pub struct DefaultEvictCallback;

impl OnEvictCallback for DefaultEvictCallback {
    fn on_evict<K, V>(&self, _: &K, _: &V) {}
}

pub trait OnEvictCallback {
    fn on_evict<K, V>(&self, key: &K, val: &V);
}

/// `PutResult` is returned when try to put a entry in cache
///
/// - **`PutResult::Put`** means that the key is not in cache previously, and the cache has enough
/// capacity, no evict happens.
///
/// - **`PutResult::Update`** means that the key already exists in the cache,
/// and this operation updates the key's value and the inner is the old value.
///
/// - **`PutResult::Evicted`** means that the the key is not in cache previously,
/// but the cache is full, so the evict happens. The inner is the evicted entry `(Key, Value)`.
pub enum PutResult<K, V> {
    /// `Put` means that the key is not in cache previously, and the cache has enough
    /// capacity, no evict happens.
    Put,

    /// `Update` means that the key already exists in the cache,
    /// and this operation updates the key's value and the inner is the old value
    Update(V),

    /// `Evicted` means that the the key is not in cache previously,
    /// but the cache is full, so the evict happens. The inner is the evicted entry `(Key, Value)`.
    Evicted{
        key: K,
        value: V,
    },
}

impl<K: PartialEq, V: PartialEq> PartialEq for PutResult<K, V> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            PutResult::Put => match other {
                PutResult::Put => true,
                _ => false,
            }
            PutResult::Update(old_val) => match other {
                PutResult::Update(v) => *v == *old_val,
                _ => false
            }
            PutResult::Evicted{key, value} => match other {
                PutResult::Evicted{ key: ok, value: ov} => *key ==*ok && *value == *ov,
                _ => false
            }
        }
    }
}

impl<K: Eq, V: Eq> Eq for PutResult<K, V> {}

impl<K: Debug, V: Debug> Debug for PutResult<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PutResult::Put => write!(f, "PutResult::Put"),
            PutResult::Update(old_val) => write!(f, "PutResult::Update({:?})", *old_val),
            PutResult::Evicted{key: k, value: v} => write!(f, "PutResult::Evicted {{key: {:?}, val: {:?}}}", *k, *v),
        }
    }
}

impl<K: Clone, V: Clone> Clone for PutResult<K, V> {
    fn clone(&self) -> Self {
        match self {
            PutResult::Put => PutResult::Put,
            PutResult::Update(v) => PutResult::Update(v.clone()),
            PutResult::Evicted{key: k, value: v} => PutResult::Evicted{ key: k.clone(), value: v.clone()},
        }
    }
}

impl<K: Copy, V: Copy> Copy for PutResult<K, V> {}


/// `CacheError` is the errors of this crate.
#[derive(Debug, PartialEq)]
pub enum CacheError {
    InvalidSize(usize),
    InvalidRecentRatio(f64),
    InvalidGhostRatio(f64),
}

impl Display for CacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            CacheError::InvalidSize(size) => write!(f, "invalid cache size {}", *size),
            CacheError::InvalidRecentRatio(r) => write!(f, "invalid recent ratio {}", *r),
            CacheError::InvalidGhostRatio(r) => write!(f, "invalid ghost ratio {}", *r)
        }
    }
}
