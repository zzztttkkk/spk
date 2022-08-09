use std::{borrow::Borrow, hash::Hash};
use std::{collections::HashMap, mem::replace};

enum MultiMapData<K, V> {
    Small(Vec<K>, Vec<V>),
    Big(HashMap<K, V>),
}

impl<K, V> MultiMapData<K, V> {
    const SMALL_SIZE: usize = 32;
    const fn new() -> Self {
        Self::Small(vec![], vec![])
    }
    fn clear(&mut self) {
        match self {
            MultiMapData::Small(k, v) => {
                k.clear();
                v.clear();
            }
            MultiMapData::Big(map) => map.clear(),
        }
    }

    #[inline]
    const fn is_small(&self) -> bool {
        matches!(self, MultiMapData::Small(_, _))
    }

    #[inline]
    const fn is_big(&self) -> bool {
        !self.is_small()
    }

    #[inline]
    fn capacity(&self) -> usize {
        match self {
            MultiMapData::Small(keys, _) => keys.capacity(),
            MultiMapData::Big(map) => map.capacity(),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            MultiMapData::Small(keys, _) => keys.len(),
            MultiMapData::Big(map) => map.len(),
        }
    }
}

impl<K, V> MultiMapData<K, V>
where
    K: Hash + Eq,
{
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: ?Sized,
        Q: Hash + Eq,
        K: Borrow<Q>,
    {
        match self {
            MultiMapData::Small(keys, values) => keys
                .iter()
                .position(|k| k.borrow() == key)
                .map(|idx| &values[idx]),
            MultiMapData::Big(map) => map.get(key),
        }
    }

    fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: ?Sized,
        Q: Hash + Eq,
        K: Borrow<Q>,
    {
        match self {
            MultiMapData::Small(k, v) => k
                .iter()
                .position(|k| k.borrow() == key)
                .map(|idx| &mut v[idx]),
            MultiMapData::Big(map) => map.get_mut(key),
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.grow();
        match self {
            MultiMapData::Small(keys, values) => {
                if let Some(index) = keys.iter().position(|k| k == &key) {
                    Some(replace(values.get_mut(index).unwrap(), value))
                } else {
                    keys.push(key);
                    values.push(value);
                    None
                }
            }
            MultiMapData::Big(map) => map.insert(key, value),
        }
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: ?Sized,
        Q: Hash + Eq,
        K: Borrow<Q>,
    {
        match self {
            MultiMapData::Small(keys, values) => {
                if let Some(index) = keys.iter().position(|k| k.borrow() == key) {
                    keys.swap_remove(index);
                    Some(values.swap_remove(index))
                } else {
                    None
                }
            }
            MultiMapData::Big(map) => map.remove(key),
        }
    }

    fn grow(&mut self) {
        if let Self::Small(keys, values) = self {
            if keys.len() >= Self::SMALL_SIZE {
                let map = keys.drain(..).zip(values.drain(..)).collect();
                let _ = replace(self, Self::Big(map));
            }
        }
    }
}

pub struct MultiMap {
    data: MultiMapData<String, Vec<String>>,
}

impl MultiMap {
    pub fn new() -> Self {
        Self {
            data: MultiMapData::new(),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) ->usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn append(&mut self, k: &str, v: impl ToString) {
        let v = v.to_string();
        if let Some(values) = self.data.get_mut(k) {
            values.push(v)
        } else {
            self.data.insert(k.to_owned(), vec![v]);
        }
    }

    pub fn remove(&mut self, k: &str) {
        self.data.remove(k);
    }

    pub fn reset(&mut self, k: &str, v: impl ToString) {
        let v = v.to_string();
        if let Some(values) = self.data.get_mut(k) {
            values.clear();
            values.push(v);
        } else {
            self.data.insert(k.to_owned(), vec![v]);
        }
    }

    pub fn get(&self, k: &str) -> Option<&[String]> {
        self.data.get(k).map(|v| v.as_slice())
    }

    pub fn getone(&self, k: &str) -> Option<&String> {
        self.get(k)?.first()
    }
}

#[cfg(test)]
mod tests {
    use crate::h2tp::utils::multi_map::MultiMap;

    #[test]
    fn test_mm() {
        let mut mm = MultiMap::new();
        mm.append("a", "1");
        assert_eq!(mm.get("a"), Some(["1".to_owned()].as_slice()));
        mm.remove("a");
        assert!(mm.is_empty());
        assert_eq!(mm.get("a"), None);
        mm.append("a", "2");
        mm.append("a", "4");
        assert_eq!(mm.get("a"), Some(["2".to_owned(),"4".to_owned()].as_slice()));
        mm.clear();
        assert!(mm.is_empty());
    }
}
