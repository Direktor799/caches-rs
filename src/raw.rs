// MIT License

// Copyright (c) 2016 Jerome Froelich

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use alloc::borrow::Borrow;
use alloc::boxed::Box;
use core::fmt;
use core::hash::{BuildHasher, Hash, Hasher};
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem;
use core::ptr;
use core::usize;

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;
#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;
use crate::{DefaultEvictCallback, OnEvictCallback, CacheError, DefaultHashBuilder, PutResult};

extern crate alloc;

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

#[cfg(feature = "nightly")]
#[doc(hidden)]
pub auto trait NotKeyRef {}

#[cfg(feature = "nightly")]
impl<K> !NotKeyRef for KeyRef<K> {}

#[cfg(feature = "nightly")]
impl<K, D> Borrow<D> for KeyRef<K>
    where
        K: Borrow<D>,
        D: NotKeyRef + ?Sized,
{
    fn borrow(&self) -> &D {
        unsafe { &*self.k }.borrow()
    }
}

#[cfg(not(feature = "nightly"))]
impl<K> Borrow<K> for KeyRef<K> {
    fn borrow(&self) -> &K {
        unsafe { &*self.k }
    }
}

// Struct used to hold a key value pair. Also contains references to previous and next entries
// so we can maintain the entries in a linked list ordered by their use.
struct EntryNode<K, V> {
    key: mem::MaybeUninit<K>,
    val: mem::MaybeUninit<V>,
    prev: *mut EntryNode<K, V>,
    next: *mut EntryNode<K, V>,
}

impl<K, V> EntryNode<K, V> {
    fn new(key: K, val: V) -> Self {
        EntryNode {
            key: mem::MaybeUninit::new(key),
            val: mem::MaybeUninit::new(val),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }

    fn new_sigil() -> Self {
        EntryNode {
            key: mem::MaybeUninit::uninit(),
            val: mem::MaybeUninit::uninit(),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}

#[cfg(feature = "hashbrown")]
pub type DefaultHasher = hashbrown::hash_map::DefaultHashBuilder;
#[cfg(not(feature = "hashbrown"))]
pub type DefaultHasher = std::collections::hash_map::RandomState;

fn check_size(size: usize) -> Result<(), CacheError> {
    if size == 0 {
        Err(CacheError::InvalidSize(0))
    } else {
        Ok(())
    }
}

/// An RawLRU Cache
pub struct RawLRU<K, V, E = DefaultEvictCallback,  S = DefaultHasher> {
    map: HashMap<KeyRef<K>, Box<EntryNode<K, V>>, S>,
    cap: usize,

    on_evict: Option<E>,
    
    // head and tail are sigil nodes to faciliate inserting entries
    head: *mut EntryNode<K, V>,
    tail: *mut EntryNode<K, V>,
}

impl<K: Hash + Eq, V> RawLRU<K, V> {
    /// Creates a new LRU Cache that holds at most `cap` items.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache: RawLRU<isize, &str> = RawLRU::new(10).unwrap();
    /// ```
    pub fn new(cap: usize) -> Result<Self, CacheError> {
        check_size(cap).map(|_| Self::construct(cap, HashMap::with_capacity(cap), None))
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> RawLRU<K, V, DefaultEvictCallback, S> {
    /// Creates a new LRU Cache that holds at most `cap` items and
    /// uses the provided hash builder to hash keys.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::{RawLRU, DefaultHashBuilder};
    ///
    /// let s = DefaultHashBuilder::default();
    /// let mut cache: RawLRU<isize, &str> = RawLRU::with_hasher(10, s).unwrap();
    /// ```
    pub fn with_hasher(cap: usize, hash_builder: S) -> Result<Self, CacheError> {
        check_size(cap).map(|_| Self::construct(cap, HashMap::with_capacity_and_hasher(cap, hash_builder), None))
    }
}

impl<K: Hash + Eq, V, E: OnEvictCallback> RawLRU<K, V, E, DefaultHashBuilder> {
    /// Creates a new LRU Cache that holds at most `cap` items and
    /// uses the provided hash builder to hash keys.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::{RawLRU, DefaultHashBuilder};
    ///
    /// let s = DefaultHashBuilder::default();
    /// let mut cache: RawLRU<isize, &str> = RawLRU::with_hasher(10, s).unwrap();
    /// ```
    pub fn with_on_evict_cb(cap: usize, cb: E) -> Result<Self, CacheError> {
        check_size(cap).map(|_| Self::construct(cap, HashMap::with_capacity(cap), Some(cb)))
    }
}

impl<K: Hash + Eq, V, E: OnEvictCallback, S: BuildHasher> RawLRU<K, V, E, S> {
    pub fn with_hasher_and_on_evict_cb(cap: usize, cb: E, hasher: S) -> Result<Self, CacheError> {
        check_size(cap).map(|_| Self::construct(cap, HashMap::with_capacity_and_hasher(cap, hasher), Some(cb)))
    }

    /// Creates a new LRU Cache with the given capacity.
    fn construct(cap: usize, map: HashMap<KeyRef<K>, Box<EntryNode<K, V>>, S>, cb: Option<E>) -> Self {
        // NB: The compiler warns that cache does not need to be marked as mutable if we
        // declare it as such since we only mutate it inside the unsafe block.
        let cache = Self {
            map,
            cap,
            on_evict: cb,
            head: Box::into_raw(Box::new(EntryNode::new_sigil())),
            tail: Box::into_raw(Box::new(EntryNode::new_sigil())),
        };

        unsafe {
            (*cache.head).next = cache.tail;
            (*cache.tail).prev = cache.head;
        }

        cache
    }

    /// Puts a key-value pair into cache, returns a [`PutResult`].
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::{RawLRU, PutResult};
    /// let mut cache = RawLRU::new(2);
    ///
    /// assert_eq!(PutResult::Put, cache.put(1, "a"));
    /// assert_eq!(PutResult::Put, cache.put(2, "b"));
    /// assert_eq!(PutResult::Update("b"), cache.put(2, "beta"));
    /// assert_eq!(PutResult::Evicted{ key: 1, value: "a"}, cache.put(3, "c"));
    ///
    /// assert_eq!(cache.get(&1), Some(&"a"));
    /// assert_eq!(cache.get(&2), Some(&"beta"));
    /// ```
    ///
    /// [`PutResult`]: struct.PutResult.html
    pub fn put(&mut self, mut k: K, mut v: V) -> PutResult<K, V> {
        let node_ptr = self.map.get_mut(&KeyRef { k: &k }).map(|node| {
            let node_ptr: *mut EntryNode<K, V> = &mut **node;
            node_ptr
        });

        match node_ptr {
            Some(node_ptr) => {
                // if the key is already in the cache just update its value and move it to the
                // front of the list
                unsafe { mem::swap(&mut v, &mut (*(*node_ptr).val.as_mut_ptr()) as &mut V) }
                self.detach(node_ptr);
                self.attach(node_ptr);
                PutResult::Update(v)
            }
            None => if self.len() >= self.cap() {
                    // if the cache is full, remove the last entry so we can use it for the new key
                    let old_key = KeyRef {
                        k: unsafe { &(*(*(*self.tail).prev).key.as_ptr()) },
                    };
                    let mut old_node = self.map.remove(&old_key).unwrap();
                    let old_node_ptr = old_node.as_mut();

                    // if the key and value with the least recent used entry
                    unsafe {
                        mem::swap(&mut v, &mut (*(*old_node_ptr).val.as_mut_ptr()) as &mut V);
                        mem::swap(&mut k, &mut (*(*old_node_ptr).key.as_mut_ptr()) as &mut K);
                    }

                    let node_ptr: *mut EntryNode<K, V> = &mut *old_node;
                    self.detach(node_ptr);
                    self.attach(node_ptr);

                    let keyref = unsafe { (*node_ptr).key.as_ptr() };
                    self.map.insert(KeyRef { k: keyref }, old_node);
                    self.cb(&k, &v);

                    PutResult::Evicted{
                        key: k,
                        value: v,
                    }
                } else {
                    // if the cache is not full allocate a new EntryNode
                    let mut node = Box::new(EntryNode::new(k, v));
                    let node_ptr: *mut EntryNode<K, V> = &mut *node;
                    self.attach(node_ptr);

                    let keyref = unsafe { (*node_ptr).key.as_ptr() };
                    self.map.insert(KeyRef { k: keyref }, node);
                    PutResult::Put
                }

        }
    }

    /// Returns a reference to the value of the key in the cache or `None` if it is not
    /// present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(2, "c");
    /// cache.put(3, "d");
    ///
    /// assert_eq!(cache.get(&1), None);
    /// assert_eq!(cache.get(&2), Some(&"c"));
    /// assert_eq!(cache.get(&3), Some(&"d"));
    /// ```
    pub fn get<Q>(&mut self, k: &Q) -> Option<&V>
        where
            KeyRef<K>: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(k) {
            let node_ptr: *mut EntryNode<K, V> = &mut **node;

            self.detach(node_ptr);
            self.attach(node_ptr);

            Some(unsafe { &(*(*node_ptr).val.as_ptr()) as &V })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value of the key in the cache or `None` if it
    /// is not present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put("apple", 8);
    /// cache.put("banana", 4);
    /// cache.put("banana", 6);
    /// cache.put("pear", 2);
    ///
    /// assert_eq!(cache.get_mut(&"apple"), None);
    /// assert_eq!(cache.get_mut(&"banana"), Some(&mut 6));
    /// assert_eq!(cache.get_mut(&"pear"), Some(&mut 2));
    /// ```
    pub fn get_lru(&mut self) -> Option<(&K, &V)>
    {
        if self.is_empty() {
            return None;
        }

        unsafe {
            let node = (*self.tail).prev;
            self.detach(node);
            self.attach(node);

            let val = &(*(*node).val.as_ptr()) as &V;
            let key = &(*(*node).key.as_ptr()) as &K;
            Some((key, val))
        }
    }

    /// Returns a mutable reference to the value of the key in the cache or `None` if it
    /// is not present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put("apple", 8);
    /// cache.put("banana", 4);
    /// cache.put("banana", 6);
    /// cache.put("pear", 2);
    ///
    /// assert_eq!(cache.get_mut(&"apple"), None);
    /// assert_eq!(cache.get_mut(&"banana"), Some(&mut 6));
    /// assert_eq!(cache.get_mut(&"pear"), Some(&mut 2));
    /// ```
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
        where
            KeyRef<K>: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(k) {
            let node_ptr: *mut EntryNode<K, V> = &mut **node;

            self.detach(node_ptr);
            self.attach(node_ptr);

            Some(unsafe { &mut (*(*node_ptr).val.as_mut_ptr()) as &mut V })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value of the key in the cache or `None` if it
    /// is not present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put("apple", 8);
    /// cache.put("banana", 4);
    /// cache.put("banana", 6);
    /// cache.put("pear", 2);
    ///
    /// assert_eq!(cache.get_mut(&"apple"), None);
    /// assert_eq!(cache.get_mut(&"banana"), Some(&mut 6));
    /// assert_eq!(cache.get_mut(&"pear"), Some(&mut 2));
    /// ```
    pub fn get_lru_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.is_empty() {
            return None;
        }

        unsafe {
            let node = (*self.tail).prev;
            self.detach(node);
            self.attach(node);
            let key = &(*(*node).key.as_ptr()) as &K;
            let val = &mut (*(*node).val.as_mut_ptr()) as &mut V;
            Some((key, val))
        }
    }

    /// Returns a reference to the value corresponding to the key in the cache or `None` if it is
    /// not present in the cache. Unlike `get`, `peek` does not update the LRU list so the key's
    /// position will be unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    ///
    /// assert_eq!(cache.peek(&1), Some(&"a"));
    /// assert_eq!(cache.peek(&2), Some(&"b"));
    /// ```
    pub fn peek<Q>(&self, k: &Q) -> Option<&V>
        where
            KeyRef<K>: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        self.map
            .get(k)
            .map(|node| unsafe { &(*(*node).val.as_ptr()) as &V })
    }

    /// Returns a mutable reference to the value corresponding to the key in the cache or `None`
    /// if it is not present in the cache. Unlike `get_mut`, `peek_mut` does not update the LRU
    /// list so the key's position will be unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    ///
    /// assert_eq!(cache.peek_mut(&1), Some(&mut "a"));
    /// assert_eq!(cache.peek_mut(&2), Some(&mut "b"));
    /// ```
    pub fn peek_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
        where
            KeyRef<K>: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        match self.map.get_mut(k) {
            None => None,
            Some(node) => Some(unsafe { &mut (*(*node).val.as_mut_ptr()) as &mut V }),
        }
    }

    /// Returns the value corresponding to the least recently used item or `None` if the
    /// cache is empty. Like `peek`, `peek_lru` does not update the LRU list so the item's
    /// position will be unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    ///
    /// assert_eq!(cache.peek_lru(), Some((&1, &"a")));
    /// ```
    pub fn peek_lru<'a>(&'_ self) -> Option<(&'a K, &'a V)> {
        if self.is_empty() {
            return None;
        }

        let (key, val);
        unsafe {
            let node = (*self.tail).prev;
            key = &(*(*node).key.as_ptr()) as &K;
            val = &(*(*node).val.as_ptr()) as &V;
        }

        Some((key, val))
    }

    /// Returns the value corresponding to the least recently used item or `None` if the
    /// cache is empty. Like `peek`, `peek_lru` does not update the LRU list so the item's
    /// position will be unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    ///
    /// assert_eq!(cache.peek_lru_mut(), Some((&1, &mut "a")));
    /// ```
    pub fn peek_lru_mut<'a>(&'_ mut self) -> Option<(&'a K, &'a mut V)> {
        if self.is_empty() {
            return None;
        }

        let (key, val);
        unsafe {
            let node = (*self.tail).prev;
            key = &(*(*node).key.as_ptr()) as &K;
            val = &mut (*(*node).val.as_mut_ptr()) as &mut V;
        }

        Some((key, val))
    }

    /// Returns a bool indicating whether the given key is in the cache. Does not update the
    /// LRU list.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2).unwrap();
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(3, "c");
    ///
    /// assert!(!cache.contains(&1));
    /// assert!(cache.contains(&2));
    /// assert!(cache.contains(&3));
    /// ```
    pub fn contains<Q>(&self, k: &Q) -> bool
        where
            KeyRef<K>: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(k)
    }

    /// Removes and returns the value corresponding to the key from the cache or
    /// `None` if it does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put(2, "a");
    ///
    /// assert_eq!(cache.remove(&1), None);
    /// assert_eq!(cache.remove(&2), Some("a"));
    /// assert_eq!(cache.remove(&2), None);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
        where
            KeyRef<K>: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        match self.map.remove(k) {
            None => None,
            Some(mut old_node) => {
                let node_ptr: *mut EntryNode<K, V> = &mut *old_node;
                self.detach(node_ptr);
                unsafe {
                    let val  = old_node.val.assume_init();
                    self.cb(&*old_node.key.as_ptr(), &val);
                    ptr::drop_in_place(old_node.key.as_mut_ptr());
                    Some(val)
                }
            }
        }
    }

    /// Removes and returns the key and value corresponding to the least recently
    /// used item or `None` if the cache is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    ///
    /// cache.put(2, "a");
    /// cache.put(3, "b");
    /// cache.put(4, "c");
    /// cache.get(&3);
    ///
    /// assert_eq!(cache.remove_lru(), Some((4, "c")));
    /// assert_eq!(cache.remove_lru(), Some((3, "b")));
    /// assert_eq!(cache.remove_lru(), None);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn remove_lru(&mut self) -> Option<(K, V)> {
        let node = self.remove_last()?;
        // N.B.: Can't destructure directly because of https://github.com/rust-lang/rust/issues/28536
        let node = *node;
        let EntryNode { key, val, .. } = node;
        unsafe {
            let key = key.assume_init();
            let val  = val.assume_init();
            self.cb(&key, &val);
            Some((key, val))
        }
    }

    /// Returns the number of key-value pairs that are currently in the the cache.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    /// assert_eq!(cache.len(), 0);
    ///
    /// cache.put(1, "a");
    /// assert_eq!(cache.len(), 1);
    ///
    /// cache.put(2, "b");
    /// assert_eq!(cache.len(), 2);
    ///
    /// cache.put(3, "c");
    /// assert_eq!(cache.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns a bool indicating whether the cache is empty or not.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache = RawLRU::new(2);
    /// assert!(cache.is_empty());
    ///
    /// cache.put(1, "a");
    /// assert!(!cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.len() == 0
    }

    /// Returns the maximum number of key-value pairs the cache can hold.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache: RawLRU<isize, &str> = RawLRU::new(2).unwrap();
    /// assert_eq!(cache.cap(), 2);
    /// ```
    pub fn cap(&self) -> usize {
        self.cap
    }

    /// Resizes the cache. If the new capacity is smaller than the size of the current
    /// cache any entries past the new capacity are discarded.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache: RawLRU<isize, &str> = RawLRU::new(2).unwrap();
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.resize(4);
    /// cache.put(3, "c");
    /// cache.put(4, "d");
    ///
    /// assert_eq!(cache.len(), 4);
    /// assert_eq!(cache.get(&1), Some(&"a"));
    /// assert_eq!(cache.get(&2), Some(&"b"));
    /// assert_eq!(cache.get(&3), Some(&"c"));
    /// assert_eq!(cache.get(&4), Some(&"d"));
    /// ```
    pub fn resize(&mut self, cap: usize) -> u64 {
        let mut evicted = 0u64;
        // return early if capacity doesn't change
        if cap == self.cap {
            return evicted;
        }

        while self.map.len() > cap {
            self.remove_lru();
            evicted += 1;
        }
        self.map.shrink_to_fit();

        self.cap = cap;
        evicted
    }

    /// Clears the contents of the cache.
    ///
    /// # Example
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    /// let mut cache: RawLRU<isize, &str> = RawLRU::new(2).unwrap();
    /// assert_eq!(cache.len(), 0);
    ///
    /// cache.put(1, "a");
    /// assert_eq!(cache.len(), 1);
    ///
    /// cache.put(2, "b");
    /// assert_eq!(cache.len(), 2);
    ///
    /// cache.purge();
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn purge(&mut self) {
        while self.remove_lru().is_some() {}
    }

    /// An iterator visiting all keys in most-recently used order. The iterator element type is
    /// `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    ///
    /// let mut cache = RawLRU::new(3);
    /// cache.put("a", 1);
    /// cache.put("b", 2);
    /// cache.put("c", 3);
    ///
    /// for (key, val) in cache.keys() {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
    pub fn keys<'a>(&'_ self) -> Keys<'a, K, V> {
        Keys {
            inner: Iter {
                len: self.len(),
                ptr: unsafe { (*self.tail).prev },
                end: unsafe { (*self.head).next },
                phantom: PhantomData,
            }
        }
    }

    /// An iterator visiting all keys in less-recently used order. The iterator element type is
    /// `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    ///
    /// let mut cache = RawLRU::new(3);
    /// cache.put("a", 1);
    /// cache.put("b", 2);
    /// cache.put("c", 3);
    ///
    /// for key in cache.reversed_keys() {
    ///     println!("key: {}", key);
    /// }
    /// ```
    pub fn reversed_keys<'a>(&'_ self) -> Keys<'a, K, V> {
        Keys {
            inner: Iter {
                len: self.len(),
                ptr: unsafe { (*self.tail).prev },
                end: unsafe { (*self.head).next },
                phantom: PhantomData,
            }
        }
    }

    /// An iterator visiting all entries in most-recently used order. The iterator element type is
    /// `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    ///
    /// let mut cache = RawLRU::new(3);
    /// cache.put("a", 1);
    /// cache.put("b", 2);
    /// cache.put("c", 3);
    ///
    /// for (key, val) in cache.iter() {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
    pub fn iter<'a>(&'_ self) -> Iter<'a, K, V> {
        Iter {
            len: self.len(),
            ptr: unsafe { (*self.head).next },
            end: unsafe { (*self.tail).prev },
            phantom: PhantomData,
        }
    }

    /// An iterator visiting all entries in less-recently used order. The iterator element type is
    /// `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    ///
    /// let mut cache = RawLRU::new(3);
    /// cache.put("a", 1);
    /// cache.put("b", 2);
    /// cache.put("c", 3);
    ///
    /// for (key, val) in cache.reversed_iter() {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
    pub fn reversed_iter<'a>(&'_ self) -> Iter<'a, K, V> {
        Iter {
            len: self.len(),
            ptr: unsafe { (*self.tail).prev },
            end: unsafe { (*self.head).next },
            phantom: PhantomData,
        }
    }

    /// An iterator visiting all entries in most-recently-used order, giving a mutable reference on
    /// V.  The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    ///
    /// struct HddBlock {
    ///     dirty: bool,
    ///     data: [u8; 512]
    /// }
    ///
    /// let mut cache = RawLRU::new(3);
    /// cache.put(0, HddBlock { dirty: false, data: [0x00; 512]});
    /// cache.put(1, HddBlock { dirty: true,  data: [0x55; 512]});
    /// cache.put(2, HddBlock { dirty: true,  data: [0x77; 512]});
    ///
    /// // write dirty blocks to disk.
    /// for (block_id, block) in cache.iter_mut() {
    ///     if block.dirty {
    ///         // write block to disk
    ///         block.dirty = false
    ///     }
    /// }
    /// ```
    pub fn iter_mut<'a>(&'_ mut self) -> IterMut<'a, K, V> {
        IterMut {
            len: self.len(),
            ptr: unsafe { (*self.head).next },
            end: unsafe { (*self.tail).prev },
            phantom: PhantomData,
        }
    }

    /// An iterator visiting all entries in less-recently-used order, giving a mutable reference on
    /// V.  The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashicorp_lru::RawLRU;
    ///
    /// struct HddBlock {
    ///     dirty: bool,
    ///     data: [u8; 512]
    /// }
    ///
    /// let mut cache = RawLRU::new(3);
    /// cache.put(0, HddBlock { dirty: false, data: [0x00; 512]});
    /// cache.put(1, HddBlock { dirty: true,  data: [0x55; 512]});
    /// cache.put(2, HddBlock { dirty: true,  data: [0x77; 512]});
    ///
    /// // write dirty blocks to disk.
    /// for (block_id, block) in cache.iter_mut() {
    ///     if block.dirty {
    ///         // write block to disk
    ///         block.dirty = false
    ///     }
    /// }
    /// ```
    pub fn reverse_iter_mut<'a>(&'_ mut self) -> IterMut<'a, K, V> {
        IterMut {
            len: self.len(),
            ptr: unsafe { (*self.tail).prev },
            end: unsafe { (*self.head).next },
            phantom: PhantomData,
        }
    }

    fn remove_last(&mut self) -> Option<Box<EntryNode<K, V>>> {
        let prev;
        unsafe { prev = (*self.tail).prev }
        if prev != self.head {
            let old_key = KeyRef {
                k: unsafe { &(*(*(*self.tail).prev).key.as_ptr()) },
            };
            let mut old_node = self.map.remove(&old_key).unwrap();
            let node_ptr: *mut EntryNode<K, V> = &mut *old_node;
            self.detach(node_ptr);
            Some(old_node)
        } else {
            None
        }
    }

    fn detach(&mut self, node: *mut EntryNode<K, V>) {

        unsafe {
            (*(*node).prev).next = (*node).next;
            (*(*node).next).prev = (*node).prev;
        }
    }

    fn attach(&mut self, node: *mut EntryNode<K, V>) {
        unsafe {
            (*node).next = (*self.head).next;
            (*node).prev = self.head;
            (*self.head).next = node;
            (*(*node).next).prev = node;
        }
    }

    #[inline]
    fn cb(&self, k: &K, v: &V) {
        if let Some(ref cb) = self.on_evict {
            cb.on_evict(k, v);
        }
    }
}

impl<K, V, E, S> Drop for RawLRU<K, V, E, S> {
    fn drop(&mut self) {
        self.map.values_mut().for_each(|e| unsafe {
            ptr::drop_in_place(e.key.as_mut_ptr());
            ptr::drop_in_place(e.val.as_mut_ptr());
        });
        // We rebox the head/tail, and because these are maybe-uninit
        // they do not have the absent k/v dropped.
        unsafe {
            let _head = *Box::from_raw(self.head);
            let _tail = *Box::from_raw(self.tail);
        }
    }
}

impl<'a, K: Hash + Eq, V, E: OnEvictCallback, S: BuildHasher> IntoIterator for &'a RawLRU<K, V, E, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<'a, K: Hash + Eq, V, E: OnEvictCallback, S: BuildHasher> IntoIterator for &'a mut RawLRU<K, V, E, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> IterMut<'a, K, V> {
        self.iter_mut()
    }
}

// The compiler does not automatically derive Send and Sync for RawLRU because it contains
// raw pointers. The raw pointers are safely encapsulated by RawLRU though so we can
// implement Send and Sync for it below.
unsafe impl<K: Send, V: Send, E: Send, S: Send> Send for RawLRU<K, V, E, S> {}
unsafe impl<K: Sync, V: Sync, E: Send, S: Sync> Sync for RawLRU<K, V, E, S> {}

impl<K: Hash + Eq, V> fmt::Debug for RawLRU<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawLRU")
            .field("len", &self.len())
            .field("cap", &self.cap())
            .finish()
    }
}

/// An iterator over the entries of a `RawLRU`.
///
/// This `struct` is created by the [`iter`] method on [`RawLRU`][`RawLRU`]. See its
/// documentation for more.
///
/// [`iter`]: struct.RawLRU.html#method.iter
/// [`RawLRU`]: struct.RawLRU.html
pub struct Iter<'a, K: 'a, V: 'a> {
    len: usize,

    ptr: *const EntryNode<K, V>,
    end: *const EntryNode<K, V>,

    phantom: PhantomData<&'a K>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*(*self.ptr).key.as_ptr()) as &K };
        let val = unsafe { &(*(*self.ptr).val.as_ptr()) as &V };

        self.len -= 1;
        self.ptr = unsafe { (*self.ptr).next };

        Some((key, val))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn count(self) -> usize {
        self.len
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*(*self.end).key.as_ptr()) as &K };
        let val = unsafe { &(*(*self.end).val.as_ptr()) as &V };

        self.len -= 1;
        self.end = unsafe { (*self.end).prev };

        Some((key, val))
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}
impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Iter<'a, K, V> {
        Iter {
            len: self.len,
            ptr: self.ptr,
            end: self.end,
            phantom: PhantomData,
        }
    }
}

// The compiler does not automatically derive Send and Sync for Iter because it contains
// raw pointers.
unsafe impl<'a, K: Send, V: Send> Send for Iter<'a, K, V> {}
unsafe impl<'a, K: Sync, V: Sync> Sync for Iter<'a, K, V> {}

/// An iterator over mutables entries of a `RawLRU`.
///
/// This `struct` is created by the [`iter_mut`] method on [`RawLRU`][`RawLRU`]. See its
/// documentation for more.
///
/// [`iter_mut`]: struct.RawLRU.html#method.iter_mut
/// [`RawLRU`]: struct.RawLRU.html
pub struct IterMut<'a, K: 'a, V: 'a> {
    len: usize,

    ptr: *mut EntryNode<K, V>,
    end: *mut EntryNode<K, V>,

    phantom: PhantomData<&'a K>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &mut (*(*self.ptr).key.as_mut_ptr()) as &mut K };
        let val = unsafe { &mut (*(*self.ptr).val.as_mut_ptr()) as &mut V };

        self.len -= 1;
        self.ptr = unsafe { (*self.ptr).next };

        Some((key, val))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn count(self) -> usize {
        self.len
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a mut V)> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &mut (*(*self.end).key.as_mut_ptr()) as &mut K };
        let val = unsafe { &mut (*(*self.end).val.as_mut_ptr()) as &mut V };

        self.len -= 1;
        self.end = unsafe { (*self.end).prev };

        Some((key, val))
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}
impl<'a, K, V> FusedIterator for IterMut<'a, K, V> {}

// The compiler does not automatically derive Send and Sync for Iter because it contains
// raw pointers.
unsafe impl<'a, K: Send, V: Send> Send for IterMut<'a, K, V> {}
unsafe impl<'a, K: Sync, V: Sync> Sync for IterMut<'a, K, V> {}

pub struct Keys<'a, K: 'a, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(| (k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.len, Some(self.inner.len))
    }

    fn count(self) -> usize {
        self.inner.len
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(k, _)| k)
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {}
impl<'a, K, V> FusedIterator for Keys<'a, K, V> {}

// The compiler does not automatically derive Send and Sync for Iter because it contains
// raw pointers.
unsafe impl<'a, K: Send, V: Send> Send for Keys<'a, K, V> {}
unsafe impl<'a, K: Sync, V: Sync> Sync for Keys<'a, K, V> {}

#[cfg(test)]
mod tests {
    use super::RawLRU;
    use core::fmt::Debug;
    use scoped_threadpool::Pool;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::{CacheError, PutResult};

    fn assert_opt_eq<V: PartialEq + Debug>(opt: Option<&V>, v: V) {
        assert!(opt.is_some());
        assert_eq!(opt.unwrap(), &v);
    }

    fn assert_opt_eq_mut<V: PartialEq + Debug>(opt: Option<&mut V>, v: V) {
        assert!(opt.is_some());
        assert_eq!(opt.unwrap(), &v);
    }

    fn assert_opt_eq_tuple<K: PartialEq + Debug, V: PartialEq + Debug>(
        opt: Option<(&K, &V)>,
        kv: (K, V),
    ) {
        assert!(opt.is_some());
        let res = opt.unwrap();
        assert_eq!(res.0, &kv.0);
        assert_eq!(res.1, &kv.1);
    }

    fn assert_opt_eq_mut_tuple<K: PartialEq + Debug, V: PartialEq + Debug>(
        opt: Option<(&K, &mut V)>,
        kv: (K, V),
    ) {
        assert!(opt.is_some());
        let res = opt.unwrap();
        assert_eq!(res.0, &kv.0);
        assert_eq!(res.1, &kv.1);
    }


    #[test]
    #[cfg(feature = "hashbrown")]
    fn test_with_hasher() {
        use hashbrown::hash_map::DefaultHashBuilder;

        let s = DefaultHashBuilder::default();
        let mut cache = RawLRU::with_hasher(16, s).unwrap();

        for i in 0..13370 {
            cache.put(i, ());
        }
        assert_eq!(cache.len(), 16);
    }

    #[test]
    fn test_put_and_get() {
        let mut cache = RawLRU::new(2).unwrap();
        assert!(cache.is_empty());

        assert_eq!(cache.put("apple", "red"), PutResult::Put);
        assert_eq!(cache.put("banana", "yellow"),PutResult::Put);

        assert_eq!(cache.cap(), 2);
        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());
        assert_opt_eq(cache.get(&"apple"), "red");
        assert_opt_eq(cache.get(&"banana"), "yellow");
    }

    #[test]
    fn test_put_and_get_mut() {
        let mut cache = RawLRU::new(2).unwrap();

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.cap(), 2);
        assert_eq!(cache.len(), 2);
        assert_opt_eq_mut(cache.get_mut(&"apple"), "red");
        assert_opt_eq_mut(cache.get_mut(&"banana"), "yellow");
    }

    #[test]
    fn test_get_mut_and_update() {
        let mut cache = RawLRU::new(2).unwrap();

        cache.put("apple", 1);
        cache.put("banana", 3);

        {
            let v = cache.get_mut(&"apple").unwrap();
            *v = 4;
        }

        assert_eq!(cache.cap(), 2);
        assert_eq!(cache.len(), 2);
        assert_opt_eq_mut(cache.get_mut(&"apple"), 4);
        assert_opt_eq_mut(cache.get_mut(&"banana"), 3);
    }

    #[test]
    fn test_put_update() {
        let mut cache = RawLRU::new(1).unwrap();

        assert_eq!(cache.put("apple", "red"), PutResult::Put);
        assert_eq!(cache.put("apple", "green"), PutResult::Update("red"));

        assert_eq!(cache.len(), 1);
        assert_opt_eq(cache.get(&"apple"), "green");
    }

    #[test]
    fn test_put_removes_oldest() {
        let mut cache = RawLRU::new(2).unwrap();

        assert_eq!(cache.put("apple", "red"), PutResult::Put);
        assert_eq!(cache.put("banana", "yellow"), PutResult::Put);
        assert_eq!(cache.put("pear", "green"), PutResult::Evicted { key: "apple", value: "red"});

        assert!(cache.get(&"apple").is_none());
        assert_opt_eq(cache.get(&"banana"), "yellow");
        assert_opt_eq(cache.get(&"pear"), "green");

        // Even though we inserted "apple" into the cache earlier it has since been removed from
        // the cache so there is no current value for `put` to return.
        assert_eq!(cache.put("apple", "green"), PutResult::Evicted {key: "banana", value: "yellow"});
        assert_eq!(cache.put("tomato", "red"), PutResult::Evicted {key: "pear", value: "green"});

        assert!(cache.get(&"pear").is_none());
        assert_opt_eq(cache.get(&"apple"), "green");
        assert_opt_eq(cache.get(&"tomato"), "red");
    }

    #[test]
    fn test_peek() {
        let mut cache = RawLRU::new(2).unwrap();

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_opt_eq(cache.peek(&"banana"), "yellow");
        assert_opt_eq(cache.peek(&"apple"), "red");

        cache.put("pear", "green");

        assert!(cache.peek(&"apple").is_none());
        assert_opt_eq(cache.peek(&"banana"), "yellow");
        assert_opt_eq(cache.peek(&"pear"), "green");
    }

    #[test]
    fn test_peek_mut() {
        let mut cache = RawLRU::new(2).unwrap();

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_opt_eq_mut(cache.peek_mut(&"banana"), "yellow");
        assert_opt_eq_mut(cache.peek_mut(&"apple"), "red");
        assert!(cache.peek_mut(&"pear").is_none());

        cache.put("pear", "green");

        assert!(cache.peek_mut(&"apple").is_none());
        assert_opt_eq_mut(cache.peek_mut(&"banana"), "yellow");
        assert_opt_eq_mut(cache.peek_mut(&"pear"), "green");

        {
            let v = cache.peek_mut(&"banana").unwrap();
            *v = "green";
        }

        assert_opt_eq_mut(cache.peek_mut(&"banana"), "green");
    }

    #[test]
    fn test_peek_lru() {
        let mut cache = RawLRU::new(2).unwrap();

        assert!(cache.peek_lru().is_none());

        cache.put("apple", "red");
        cache.put("banana", "yellow");
        assert_opt_eq_tuple(cache.peek_lru(), ("apple", "red"));

        cache.get(&"apple");
        assert_opt_eq_tuple(cache.peek_lru(), ("banana", "yellow"));

        cache.purge();
        assert!(cache.peek_lru().is_none());
    }

    #[test]
    fn test_contains() {
        let mut cache = RawLRU::new(2).unwrap();

        cache.put("apple", "red");
        cache.put("banana", "yellow");
        cache.put("pear", "green");

        assert!(!cache.contains(&"apple"));
        assert!(cache.contains(&"banana"));
        assert!(cache.contains(&"pear"));
    }

    #[test]
    fn test_remove() {
        let mut cache = RawLRU::new(2).unwrap();

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.len(), 2);
        assert_opt_eq(cache.get(&"apple"), "red");
        assert_opt_eq(cache.get(&"banana"), "yellow");

        let popped = cache.remove(&"apple");
        assert!(popped.is_some());
        assert_eq!(popped.unwrap(), "red");
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&"apple").is_none());
        assert_opt_eq(cache.get(&"banana"), "yellow");
    }

    #[test]
    fn test_remove_lru() {
        let mut cache = RawLRU::new(200).unwrap();

        for i in 0..75 {
            cache.put(i, "A");
        }
        for i in 0..75 {
            cache.put(i + 100, "B");
        }
        for i in 0..75 {
            cache.put(i + 200, "C");
        }
        assert_eq!(cache.len(), 200);

        for i in 0..75 {
            assert_opt_eq(cache.get(&(74 - i + 100)), "B");
        }
        assert_opt_eq(cache.get(&25), "A");

        for i in 26..75 {
            assert_eq!(cache.remove_lru(), Some((i, "A")));
        }
        for i in 0..75 {
            assert_eq!(cache.remove_lru(), Some((i + 200, "C")));
        }
        for i in 0..75 {
            assert_eq!(cache.remove_lru(), Some((74 - i + 100, "B")));
        }
        assert_eq!(cache.remove_lru(), Some((25, "A")));
        for _ in 0..50 {
            assert_eq!(cache.remove_lru(), None);
        }
    }

    #[test]
    fn test_clear() {
        let mut cache = RawLRU::new(2).unwrap();

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.len(), 2);
        assert_opt_eq(cache.get(&"apple"), "red");
        assert_opt_eq(cache.get(&"banana"), "yellow");

        cache.purge();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_resize_larger() {
        let mut cache = RawLRU::new(2).unwrap();

        cache.put(1, "a");
        cache.put(2, "b");
        cache.resize(4);
        cache.put(3, "c");
        cache.put(4, "d");

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_resize_smaller() {
        let mut cache = RawLRU::new(4).unwrap();

        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        cache.put(4, "d");

        cache.resize(2);

        assert_eq!(cache.len(), 2);
        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_send() {
        use std::thread;

        let mut cache = RawLRU::new(4).unwrap();
        cache.put(1, "a");

        let handle = thread::spawn(move || {
            assert_eq!(cache.get(&1), Some(&"a"));
        });

        assert!(handle.join().is_ok());
    }

    #[test]
    fn test_multiple_threads() {
        let mut pool = Pool::new(1);
        let mut cache = RawLRU::new(4).unwrap();
        cache.put(1, "a");

        let cache_ref = &cache;
        pool.scoped(|scoped| {
            scoped.execute(move || {
                assert_eq!(cache_ref.peek(&1), Some(&"a"));
            });
        });

        assert_eq!((cache_ref).peek(&1), Some(&"a"));
    }

    #[test]
    fn test_iter_forwards() {
        let mut cache = RawLRU::new(3).unwrap();
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_tuple(iter.next(), ("c", 3));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_tuple(iter.next(), ("b", 2));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_tuple(iter.next(), ("a", 1));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next(), None);
        }
        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_mut_tuple(iter.next(), ("c", 3));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_mut_tuple(iter.next(), ("b", 2));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_mut_tuple(iter.next(), ("a", 1));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn test_iter_backwards() {
        let mut cache = RawLRU::new(3).unwrap();
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_tuple(iter.next_back(), ("a", 1));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_tuple(iter.next_back(), ("b", 2));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_tuple(iter.next_back(), ("c", 3));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }

        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_mut_tuple(iter.next_back(), ("a", 1));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_mut_tuple(iter.next_back(), ("b", 2));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_mut_tuple(iter.next_back(), ("c", 3));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }
    }

    #[test]
    fn test_iter_forwards_and_backwards() {
        let mut cache = RawLRU::new(3).unwrap();
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_tuple(iter.next(), ("c", 3));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_tuple(iter.next_back(), ("a", 1));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_tuple(iter.next(), ("b", 2));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }
        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_mut_tuple(iter.next(), ("c", 3));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_mut_tuple(iter.next_back(), ("a", 1));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_mut_tuple(iter.next(), ("b", 2));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }
    }

    #[test]
    fn test_iter_multiple_threads() {
        let mut pool = Pool::new(1);
        let mut cache = RawLRU::new(3).unwrap();
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        let mut iter = cache.iter();
        assert_eq!(iter.len(), 3);
        assert_opt_eq_tuple(iter.next(), ("c", 3));

        {
            let iter_ref = &mut iter;
            pool.scoped(|scoped| {
                scoped.execute(move || {
                    assert_eq!(iter_ref.len(), 2);
                    assert_opt_eq_tuple(iter_ref.next(), ("b", 2));
                });
            });
        }

        assert_eq!(iter.len(), 1);
        assert_opt_eq_tuple(iter.next(), ("a", 1));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_clone() {
        let mut cache = RawLRU::new(3).unwrap();
        cache.put("a", 1);
        cache.put("b", 2);

        let mut iter = cache.iter();
        let mut iter_clone = iter.clone();

        assert_eq!(iter.len(), 2);
        assert_opt_eq_tuple(iter.next(), ("b", 2));
        assert_eq!(iter_clone.len(), 2);
        assert_opt_eq_tuple(iter_clone.next(), ("b", 2));

        assert_eq!(iter.len(), 1);
        assert_opt_eq_tuple(iter.next(), ("a", 1));
        assert_eq!(iter_clone.len(), 1);
        assert_opt_eq_tuple(iter_clone.next(), ("a", 1));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter_clone.len(), 0);
        assert_eq!(iter_clone.next(), None);
    }

    #[test]
    fn test_that_pop_actually_detaches_node() {
        let mut cache = RawLRU::new(5).unwrap();

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);
        cache.put("d", 4);
        cache.put("e", 5);

        assert_eq!(cache.remove(&"c"), Some(3));

        cache.put("f", 6);

        let mut iter = cache.iter();
        assert_opt_eq_tuple(iter.next(), ("f", 6));
        assert_opt_eq_tuple(iter.next(), ("e", 5));
        assert_opt_eq_tuple(iter.next(), ("d", 4));
        assert_opt_eq_tuple(iter.next(), ("b", 2));
        assert_opt_eq_tuple(iter.next(), ("a", 1));
        assert!(iter.next().is_none());
    }

    #[test]
    #[cfg(feature = "nightly")]
    fn test_get_with_borrow() {
        use alloc::string::String;

        let mut cache = RawLRU::new(2).unwrap();

        let key = String::from("apple");
        cache.put(key, "red");

        assert_opt_eq(cache.get("apple"), "red");
    }

    #[test]
    #[cfg(feature = "nightly")]
    fn test_get_mut_with_borrow() {
        use alloc::string::String;

        let mut cache = RawLRU::new(2).unwrap();

        let key = String::from("apple");
        cache.put(key, "red");

        assert_opt_eq_mut(cache.get_mut("apple"), "red");
    }

    #[test]
    fn test_no_memory_leaks() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = RawLRU::new(1).unwrap();
            for i in 0..n {
                cache.put(i, DropCounter {});
            }
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
    }

    #[test]
    fn test_no_memory_leaks_with_clear() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = RawLRU::new(1).unwrap();
            for i in 0..n {
                cache.put(i, DropCounter {});
            }
            cache.purge();
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
    }

    #[test]
    fn test_no_memory_leaks_with_resize() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = RawLRU::new(1).unwrap();
            for i in 0..n {
                cache.put(i, DropCounter {});
            }
            cache.resize(0);
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
    }

    #[test]
    fn test_no_memory_leaks_with_remove() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        #[derive(Hash, Eq)]
        struct KeyDropCounter(usize);

        impl PartialEq for KeyDropCounter {
            fn eq(&self, other: &Self) -> bool {
                self.0.eq(&other.0)
            }
        }

        impl Drop for KeyDropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = RawLRU::new(1).unwrap();

            for i in 0..n {
                cache.put(KeyDropCounter(i), i);
                cache.remove(&KeyDropCounter(i));
            }
        }

        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n * 2);
    }

    #[test]
    fn test_zero_cap_no_crash() {
        let cache = RawLRU::<u64, u64>::new(0);
        assert_eq!(cache.unwrap_err(), CacheError::InvalidSize(0))
    }
}
