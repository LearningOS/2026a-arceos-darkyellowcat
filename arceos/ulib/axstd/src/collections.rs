//! Collection types provided by axstd.

pub use alloc::collections::*;
use alloc::vec::Vec;

use core::hash::{BuildHasher, Hash, Hasher};

// FNV-1a 64-bit constants
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

// A simple FNV-1a hasher implementation
pub struct FnvHasher {
    state: u64,
}

// 初始 state 设置为 FNV_OFFSET_BASIS
impl Default for FnvHasher {
    fn default() -> Self {
        FnvHasher {
            state: FNV_OFFSET_BASIS,
        }
    }
}

impl Hasher for FnvHasher {
    fn write(&mut self, bytes: &[u8]) {
        let mut state = self.state;
        for &byte in bytes {
            state ^= byte as u64;
            state = state.wrapping_mul(FNV_PRIME);
        }
        self.state = state;
    }

    fn finish(&self) -> u64 {
        // 返回最终哈希值
        self.state
    }
}

// A default hash builder that uses FNV-1a hasher
#[derive(Clone, Copy, Default)]
pub struct DefaultHashBuilder;

impl DefaultHashBuilder {
    pub fn new() -> Self {
        DefaultHashBuilder
    }
}

impl BuildHasher for DefaultHashBuilder {
    type Hasher = FnvHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FnvHasher::default()
    }
}

// A simple HashMap implementation using BTreeMap for buckets
pub struct HashMap<K, V, S = DefaultHashBuilder> {
    buckets: BTreeMap<u64, Vec<(K, V)>>,
    len: usize,
    hasher: S,
}

impl<K, V, S> HashMap<K, V, S> {
    pub fn with_hasher(hasher: S) -> Self {
        HashMap {
            buckets: BTreeMap::new(),
            len: 0,
            hasher,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.buckets.values().flat_map(|bucket| {
            bucket.iter().map(|(k, v)| (k, v))
        })
    }
}

impl<K, V> HashMap<K, V, DefaultHashBuilder> {
    pub fn new() -> Self {
        HashMap::with_hasher(DefaultHashBuilder::new())
    }
}

impl<K, V, S: BuildHasher> HashMap<K, V, S> {
    fn hash_key<Q: Hash + ?Sized>(&self, key: &Q) -> u64 {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> HashMap<K, V, S> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let hash = self.hash_key(&key);

        let bucket = self.buckets.entry(hash).or_default();

        // 如果找到相同的 Key：
        // 使用 core::mem::replace() 替换 Value
        // 返回旧 Value
        // 如果没有找到相同的 Key：
        // 将 (key, value) 添加到 Vec
        // len 增加 1
        if let Some((_, old_value)) = bucket.iter_mut().find(|(k, _)| k == &key) {
            return Some(core::mem::replace(old_value, value));
        }

        bucket.push((key, value));
        self.len += 1;

        return None;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash_key(key);
        self.buckets.get(&hash).and_then(|bucket| {
            bucket.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        })
    }
}

