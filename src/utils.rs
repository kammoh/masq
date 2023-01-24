use serde::de::{self, Deserializer, Unexpected};
use std::collections::hash_map;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct MapToSet<K, V>(HashMap<K, HashSet<V>>);

impl<K, V> MapToSet<K, V>
where
    K: Eq + std::hash::Hash,
    V: Eq + std::hash::Hash,
{
    pub fn append(&mut self, k: K, v: V) {
        self.0.entry(k).or_insert(HashSet::new()).insert(v);
    }
    pub fn get(&self, k: &K) -> Option<&HashSet<V>> {
        self.0.get(k)
    }
}

#[derive(Clone, Debug)]
pub struct MapToVec<K, V>(HashMap<K, Vec<V>>);

impl<K, V> MapToVec<K, V>
where
    K: Eq + std::hash::Hash,
{
    #[inline(always)]
    pub fn append(&mut self, k: K, v: V) {
        self.0.entry(k).or_insert(Vec::new()).push(v);
    }
    #[inline(always)]
    pub fn get(&self, k: &K) -> Option<&Vec<V>> {
        self.0.get(k)
    }
    #[inline(always)]
    pub fn iter(&self) -> hash_map::Iter<'_, K, Vec<V>> {
        self.0.iter()
    }
}

impl<K, V> Default for MapToVec<K, V> {
    #[inline(always)]
    fn default() -> Self {
        Self(Default::default())
    }
}

pub(crate) fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::Deserialize;
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}
