//! <div align="center">
//! <h1>HashiCorp-LRU</h1>
//! </div>
//! <div align="center">
//!
//! [<img alt="github" src="https://img.shields.io/badge/GITHUB-hashicorp--lru-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-hashicorp--lru-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/hashicorp-lru)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/hashicorp-lru?logo=rust&style=for-the-badge" height="22">][crates-url]
//!
//! [<img alt="Build" src="https://img.shields.io/badge/Build-passing-brightgreen?style=for-the-badge&logo=Github-Actions" height="22">][CI-url]
//! [<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/hashicorp-lru?style=for-the-badge&token=65Q9QTR99U&logo=codecov" height="22">][codecov-url]
//!
//! [<img alt="license-apache" src="https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=for-the-badge&logo=Apache" height="22">][license-apache-url]
//! [<img alt="license-mit" src="https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMzZweCIgdmlld0JveD0iMCAwIDI0IDI0IiB3aWR0aD0iMzZweCIgZmlsbD0iI2Y1YzA3NiI+PHBhdGggZD0iTTAgMGgyNHYyNEgwVjB6IiBmaWxsPSJub25lIi8+PHBhdGggZD0iTTEwLjA4IDEwLjg2Yy4wNS0uMzMuMTYtLjYyLjMtLjg3cy4zNC0uNDYuNTktLjYyYy4yNC0uMTUuNTQtLjIyLjkxLS4yMy4yMy4wMS40NC4wNS42My4xMy4yLjA5LjM4LjIxLjUyLjM2cy4yNS4zMy4zNC41My4xMy40Mi4xNC42NGgxLjc5Yy0uMDItLjQ3LS4xMS0uOS0uMjgtMS4yOXMtLjQtLjczLS43LTEuMDEtLjY2LS41LTEuMDgtLjY2LS44OC0uMjMtMS4zOS0uMjNjLS42NSAwLTEuMjIuMTEtMS43LjM0cy0uODguNTMtMS4yLjkyLS41Ni44NC0uNzEgMS4zNlM4IDExLjI5IDggMTEuODd2LjI3YzAgLjU4LjA4IDEuMTIuMjMgMS42NHMuMzkuOTcuNzEgMS4zNS43Mi42OSAxLjIuOTFjLjQ4LjIyIDEuMDUuMzQgMS43LjM0LjQ3IDAgLjkxLS4wOCAxLjMyLS4yM3MuNzctLjM2IDEuMDgtLjYzLjU2LS41OC43NC0uOTQuMjktLjc0LjMtMS4xNWgtMS43OWMtLjAxLjIxLS4wNi40LS4xNS41OHMtLjIxLjMzLS4zNi40Ni0uMzIuMjMtLjUyLjNjLS4xOS4wNy0uMzkuMDktLjYuMS0uMzYtLjAxLS42Ni0uMDgtLjg5LS4yMy0uMjUtLjE2LS40NS0uMzctLjU5LS42MnMtLjI1LS41NS0uMy0uODgtLjA4LS42Ny0uMDgtMXYtLjI3YzAtLjM1LjAzLS42OC4wOC0xLjAxek0xMiAyQzYuNDggMiAyIDYuNDggMiAxMnM0LjQ4IDEwIDEwIDEwIDEwLTQuNDggMTAtMTBTMTcuNTIgMiAxMiAyem0wIDE4Yy00LjQxIDAtOC0zLjU5LTgtOHMzLjU5LTggOC04IDggMy41OSA4IDgtMy41OSA4LTggOHoiLz48L3N2Zz4=" height="22">][license-mit-url]
//! 
//! This is a Rust implementation for [HashiCorp's golang-lru](https://github.com/hashicorp/golang-lru).
//!
//! This crate contains three LRU based cache, [`LRUCache`], [`TwoQueueCache`] and [`AdaptiveCache`].
//!
//! See [Introduction](#introduction), [Trade-Off](#trade-off) and [Usages](#usages) for more details.
//! </div>
//!
//! ## Introduction
//! LRU based caches are `O(1)` for read, write and delete.
//!
//! - [`LRUCache`] or [`RawLRU`] is a fixed size LRU cache.
//!
//!
//! - [`AdaptiveCache`] is a fixed size Adaptive Replacement Cache (ARC).
//! ARC is an enhancement over the standard LRU cache in that tracks both
//! frequency and recency of use. This avoids a burst in access to new
//! entries from evicting the frequently used older entries.
//!
//!
//! - [`TwoQueueCache`] is a fixed size 2Q cache. 2Q is an enhancement
//! over the standard LRU cache in that it tracks both frequently
//! and recently used entries separately.
//!
//! ## Trade-Off
//! In theory, [`AdaptiveCache`] and [`TwoQueueCache`] add some additional
//! tracking overhead to a [`LRUCache`] cache, computationally it is roughly **2x** the cost,
//! and the extra memory overhead is linear with the size of the cache.
//! [`AdaptiveCache`] and [`TwoQueueCache`] have similar computationally cost,
//! which has been patented by IBM, but the [`TwoQueueCache`] (2Q) need to set reasonable parameters.
//!
//! However, the implementation for the [`RawLRU`] uses [`Box`] and a
//! raw pointer for each entry to break the limitation of the
//! Rust language (It does not use [`Rc`], because [`Rc`] is slower).
//! Thus, in practice, [`TwoQueueCache`] is **2.5x** computationally
//! slower than [`LRUCache`] and [`AdaptiveCache`] is **3x** computationally slower than [`LRUCache`],
//! because [`TwoQueueCache`] and [`AdaptiveCache`] has to do more box and re-box
//! than [`LRUCache`], even though I try my best to avoid boxing and re-boxing and
//! use [`mem::swap`] to avoid memory allocating and deallocating.
//!
//! Hence, it is better to understand what is the situation is,
//! (your project wants a higher hit ratio or faster
//! computationally performance), and then choose the reasonable
//! Cache in your project.
//!
//! (For more performance details, you can clone the project and
//! run `cargo bench`. The source code for benchmark is in the
//! [benches](https://github.com/al8n/hashicorp-lru/tree/main/benches),
//! I am also looking forward to anyone's help for writing more
//! reasonable test cases for benchmark).
//!
//! ## Usages
//!
//!
//! ## Acknowledgments
//! - The implementation of `RawLRU` is highly inspired by
//! [Jerome Froelich's LRU implementation](https://github.com/jeromefroe/lru-rs)
//! and [`std::collections`] library of Rust.
//!
//! - Thanks for [HashiCorp's golang-lru](https://github.com/hashicorp/golang-lru)
//! providing the amazing Go implementation.
//!
//! [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
//! [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
//! [`mem::swap`]: https://doc.rust-lang.org/stable/std/mem/fn.swap.html
//! [`std::collections`]: https://doc.rust-lang.org/stable/std/collections/
//! [`RawLRU`]: struct.RawLRU.html
//! [`LRUCache`]: type.LRUCache.html
//! [`TwoQueueCache`]: struct.TwoQueueCache.html
//! [`AdaptiveCache`]: struct.AdaptiveCache.html
//! [Github-url]: https://github.com/al8n/hashicorp-lru/
//! [CI-url]: https://github.com/al8n/hashicorp-lru/actions/workflows/ci.yml
//! [codecov-url]: https://codecov.io/gh/al8n/hashicorp-lru
//! [license-apache-url]: https://opensource.org/licenses/Apache-2.0
//! [license-mit-url]: https://opensource.org/licenses/Apache-2.0
//! [crates-url]: https://crates.io/crates/hashicorp-lru
#![no_std]
#![cfg_attr(feature = "nightly", feature(negative_impls, auto_traits))]
#![deny(missing_docs)]

extern crate alloc;
#[cfg(feature = "hashbrown")]
extern crate hashbrown;

#[cfg(any(test, not(feature = "hashbrown")))]
extern crate std;

mod adaptive;
mod lru;
mod raw;
mod two_queue;

#[macro_use]
mod macros;

pub use adaptive::{AdaptiveCache, AdaptiveCacheBuilder};
pub use lru::LRUCache;
pub use raw::{
    KeysLRUIter, KeysMRUIter, LRUIter, LRUIterMut, MRUIter, MRUIterMut, RawLRU, ValuesLRUIter,
    ValuesLRUIterMut, ValuesMRUIter, ValuesMRUIterMut,
};
pub use two_queue::{
    TwoQueueCache, TwoQueueCacheBuilder, DEFAULT_2Q_GHOST_RATIO, DEFAULT_2Q_RECENT_RATIO,
};

use crate::raw::EntryNode;
use core::borrow::Borrow;
use core::fmt::{Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::mem;

cfg_hashbrown!(
    /// Re-export for DefaultHashBuilder
    pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;
);

cfg_not_hashbrown!(
    /// Re-export for DefaultHashBuilder
    pub type DefaultHashBuilder = std::collections::hash_map::DefaultHasher;
);

// Struct used to hold a reference to a key
#[doc(hidden)]
pub struct KeyRef<K> {
    k: *const K,
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.k).hash(state) }
    }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &KeyRef<K>) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

cfg_nightly_hidden_doc!(
    pub auto trait NotKeyRef {}
    impl<K> !NotKeyRef for KeyRef<K> {}
    impl<K, D> Borrow<D> for KeyRef<K>
    where
        K: Borrow<D>,
        D: NotKeyRef + ?Sized,
    {
        fn borrow(&self) -> &D {
            unsafe { &*self.k }.borrow()
        }
    }
);

cfg_not_nightly!(
    impl<K> Borrow<K> for KeyRef<K> {
        fn borrow(&self) -> &K {
            unsafe { &*self.k }
        }
    }
);

/// `DefaultEvictCallback` is a noop evict callback.
#[derive(Debug, Clone, Copy)]
pub struct DefaultEvictCallback;

impl OnEvictCallback for DefaultEvictCallback {
    fn on_evict<K, V>(&self, _: &K, _: &V) {}
}

/// `OnEvictCallback` is used to apply a callback for evicted entry.
///
/// # Example
/// ```
/// use hashicorp_lru::{RawLRU, OnEvictCallback};
/// use std::sync::atomic::{AtomicU64, Ordering};
/// use std::sync::Arc;
///
/// struct EvictedCounter {
///     ctr: Arc<AtomicU64>,
/// }
///
/// impl EvictedCounter {
///     pub fn new(ctr: Arc<AtomicU64>) -> Self {
///         Self {
///             ctr,
///         }
///     }
/// }
///
/// impl OnEvictCallback for EvictedCounter {
///     fn on_evict<K, V>(&self, _: &K, _: &V) {
///         self.ctr.fetch_add(1, Ordering::SeqCst);
///     }
/// }
///
/// let counter = Arc::new(AtomicU64::new(0));
///
/// let mut cache: RawLRU<u64, u64, EvictedCounter> = RawLRU::with_on_evict_cb(1, EvictedCounter::new(counter.clone())).unwrap();
///
/// cache.put(1, 1);
/// cache.put(2, 2);
/// cache.put(3, 3);
///
/// assert_eq!(counter.load(Ordering::SeqCst), 2);
/// ```
pub trait OnEvictCallback {
    /// `on_evict` is a callback function will be invoked if an entry is evicted.
    fn on_evict<K, V>(&self, key: &K, val: &V);
}

/// `PutResult` is returned when try to put a entry in cache.
///
/// - **`PutResult::Put`** means that the key is not in cache previously, and the cache has enough
/// capacity, no evict happens.
///
/// - **`PutResult::Update`** means that the key already exists in the cache,
/// and this operation updates the key's value and the inner is the old value.
///
/// - **`PutResult::Evicted`** means that the the key is not in cache previously,
/// but the cache is full, so the evict happens. The inner is the evicted entry `(Key, Value)`.
///
/// - **`PutResult::EvictedAndUpdate`** is only possible to be returned by [`TwoQueueCache`] and [`AdaptiveCache`]. For more information, please see the related examples of [`TwoQueueCache`] and [`AdaptiveCache`]
///
/// [`TwoQueueCache`]: struct.TwoQueueCache.html
/// [`AdaptiveCache`]: struct.AdaptiveCache.html
pub enum PutResult<K, V> {
    /// `Put` means that the key is not in cache previously, and the cache has enough
    /// capacity, no evict happens.
    Put,

    /// `Update` means that the key already exists in the cache,
    /// and this operation updates the key's value and the inner is the old value
    Update(V),

    /// `Evicted` means that the the key is not in cache previously,
    /// but the cache is full, so the evict happens. The inner is the evicted entry `(Key, Value)`.
    Evicted {
        /// The key for the evicted entry.
        key: K,
        /// The value for the evicted entry.
        value: V,
    },

    /// `EvictedAndUpdate` is only possible to be returned by [`TwoQueueCache`].
    ///
    /// For more information, please see the related examples of [`TwoQueueCache`].
    ///
    /// [`TwoQueueCache`]: struct.TwoQueueCache.html
    EvictedAndUpdate {
        /// The evicted entry.
        evicted: (K, V),
        /// The old value for the updated entry.
        update: V,
    },
}

impl<K: PartialEq, V: PartialEq> PartialEq for PutResult<K, V> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            PutResult::Put => match other {
                PutResult::Put => true,
                _ => false,
            },
            PutResult::Update(old_val) => match other {
                PutResult::Update(v) => *v == *old_val,
                _ => false,
            },
            PutResult::Evicted { key, value } => match other {
                PutResult::Evicted { key: ok, value: ov } => *key == *ok && *value == *ov,
                _ => false,
            },
            PutResult::EvictedAndUpdate { evicted, update } => match other {
                PutResult::EvictedAndUpdate {
                    evicted: other_evicted,
                    update: other_update,
                } => {
                    (*evicted).0 == (*other_evicted).0
                        && (*evicted).1 == (*other_evicted).1
                        && *update == *other_update
                }
                _ => false,
            },
        }
    }
}

impl<K: Eq, V: Eq> Eq for PutResult<K, V> {}

impl<K: Debug, V: Debug> Debug for PutResult<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PutResult::Put => write!(f, "PutResult::Put"),
            PutResult::Update(old_val) => write!(f, "PutResult::Update({:?})", *old_val),
            PutResult::Evicted { key: k, value: v } => {
                write!(f, "PutResult::Evicted {{key: {:?}, val: {:?}}}", *k, *v)
            }
            PutResult::EvictedAndUpdate { evicted, update } => write!(f, "PutResult::EvictedAndUpdate {{ evicted: {{key: {:?}, value: {:?}}}, update: {:?} }}", (*evicted).0, (*evicted).1, *update),
        }
    }
}

impl<K: Clone, V: Clone> Clone for PutResult<K, V> {
    fn clone(&self) -> Self {
        match self {
            PutResult::Put => PutResult::Put,
            PutResult::Update(v) => PutResult::Update(v.clone()),
            PutResult::Evicted { key: k, value: v } => PutResult::Evicted {
                key: k.clone(),
                value: v.clone(),
            },
            PutResult::EvictedAndUpdate { evicted, update } => PutResult::EvictedAndUpdate {
                evicted: evicted.clone(),
                update: update.clone(),
            },
        }
    }
}

impl<K: Copy, V: Copy> Copy for PutResult<K, V> {}

/// `CacheError` is the errors of this crate.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum CacheError {
    /// Invalid cache size
    InvalidSize(usize),
    /// Invalid recent ratio for [`TwoQueueCache`]
    ///
    /// [`TwoQueueCache`]: struct.TwoQueueCache.html
    InvalidRecentRatio(f64),
    /// Invalid ghost ratio for [`TwoQueueCache`]
    ///
    /// [`TwoQueueCache`]: struct.TwoQueueCache.html
    InvalidGhostRatio(f64),
}

impl Display for CacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            CacheError::InvalidSize(size) => write!(f, "invalid cache size {}", *size),
            CacheError::InvalidRecentRatio(r) => write!(f, "invalid recent ratio {}", *r),
            CacheError::InvalidGhostRatio(r) => write!(f, "invalid ghost ratio {}", *r),
        }
    }
}

unsafe fn swap_value<K, V>(v: &mut V, ent: &mut EntryNode<K, V>) {
    mem::swap(v, &mut (*(*ent).val.as_mut_ptr()) as &mut V);
}
