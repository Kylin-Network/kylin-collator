// Copyright (c) 2015-2021 Parity Technologies

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

//! A memory-based LRU cache.

use lru::LruCache;

use std::hash::Hash;

const INITIAL_CAPACITY: usize = 4;

/// An indicator of the resident in memory of a value.
pub trait ResidentSize {
    /// Return the resident size of the value. Users of the trait will depend
    /// on this value to remain stable unless the value is mutated.
    fn resident_size(&self) -> usize;
}

/// An LRU-cache which operates on memory used.
pub struct MemoryLruCache<K, V> {
    inner: LruCache<K, V>,
    cur_size: usize,
    max_size: usize,
}

impl<K: Eq + Hash, V: ResidentSize> MemoryLruCache<K, V> {
    /// Create a new cache with a maximum cumulative size of values.
    pub fn new(max_size: usize) -> Self {
        MemoryLruCache {
            inner: LruCache::new(INITIAL_CAPACITY),
            max_size: max_size,
            cur_size: 0,
        }
    }

    /// Insert an item.
    pub fn insert(&mut self, key: K, val: V) {
        let cap = self.inner.cap();

        // grow the cache as necessary; it operates on amount of items
        // but we're working based on memory usage.
        if self.inner.len() == cap && self.cur_size < self.max_size {
            self.inner.resize(cap * 2);
        }

        self.cur_size += val.resident_size();

        // account for any element displaced from the cache.
        if let Some(lru) = self.inner.put(key, val) {
            self.cur_size -= lru.resident_size();
        }

        self.readjust_down();
    }

    /// Get a reference to an item in the cache. It is a logic error for its
    /// heap size to be altered while borrowed.
    pub fn get(&mut self, key: &K) -> Option<&V> {
       self.inner.get(key)
    }

    /// Execute a closure with the value under the provided key.
    pub fn with_mut<U>(&mut self, key: &K, with: impl FnOnce(Option<&mut V>) -> U) -> U {
        let mut val = self.inner.get_mut(key);
        let prev_size = val.as_ref().map_or(0, |v| v.resident_size());

        let res = with(val.as_mut().map(|v: &mut &mut V| &mut **v));

        let new_size = val.as_ref().map_or(0, |v| v.resident_size());

        self.cur_size -= prev_size;
        self.cur_size += new_size;

        self.readjust_down();

        res
    }

    /// Currently-used size of values in bytes.
    pub fn current_size(&self) -> usize {
        self.cur_size
    }

    fn readjust_down(&mut self) {
        // remove elements until we are below the memory target.
        while self.cur_size > self.max_size {
            match self.inner.pop_lru() {
                Some((_, v)) => self.cur_size -= v.resident_size(),
                _ => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ResidentSize for Vec<u8> {
        fn resident_size(&self) -> usize {
            self.len()
        }
    }

    #[test]
    fn it_works() {
        let mut cache = MemoryLruCache::new(256);
        let val1 = vec![0u8; 100];
        let size1 = val1.resident_size();
        cache.insert("hello", val1);

        assert_eq!(cache.current_size(), size1);

        let val2 = vec![0u8; 210];
        let size2 = val2.resident_size();
        cache.insert("world", val2);

        assert!(cache.get(&"hello").is_none());
        assert!(cache.get(&"world").is_some());

        assert_eq!(cache.current_size(), size2);
    }
}
