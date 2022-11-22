use std::collections::HashMap;
use std::hash::Hash;
use std::time::SystemTime;

#[derive(Debug, Clone)]
struct TTLCachedValue<V> {
    value: V,
    insert_time: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Cache<K: Hash + Eq, V> {
    cache: HashMap<K, TTLCachedValue<V>>,
    ttl: u64,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new() -> Cache<K, V> {
        Default::default()
    }

    pub fn set(&mut self, key: K, value: V) -> Option<V> {
        let cached_value = self.cache.insert(
            key,
            TTLCachedValue {
                value,
                insert_time: SystemTime::now(),
            },
        );
        match cached_value {
            None => None,
            Some(cached_value) => Some(cached_value.value),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let cached_value = self.cache.get(key);
        match cached_value {
            None => None,
            Some(TTLCachedValue { insert_time, value }) => {
                match insert_time.elapsed().unwrap().as_secs() > self.ttl {
                    false => Some(value),
                    true => None,
                }
            }
        }
    }
}

impl<K: Hash + Eq, V> Default for Cache<K, V> {
    #[inline]
    fn default() -> Cache<K, V> {
        let cache = HashMap::<K, TTLCachedValue<V>>::new();
        Cache {
            cache: cache,
            ttl: 30,
        }
    }
}
