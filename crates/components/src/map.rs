use std::{
    collections::{
        hash_map::{Entry, IntoIter, Iter, IterMut, Keys, RandomState, Values, ValuesMut},
        HashMap,
    },
    hash::{BuildHasher, Hash},
    pin::Pin,
    task::{Context, Poll},
};

use async_component_core::{AsyncComponent, StateCell};

#[derive(Debug)]
pub struct HashMapComponent<K, V, S = RandomState> {
    updated: StateCell<()>,
    map: HashMap<K, V, S>,
}

impl<K: Eq + Hash, V> HashMapComponent<K, V, RandomState> {
    pub fn new() -> Self {
        Self {
            updated: StateCell::new(()),
            map: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            updated: StateCell::new(()),
            map: HashMap::with_capacity(capacity),
        }
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> HashMapComponent<K, V, S> {
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            updated: StateCell::new(()),
            map: HashMap::with_hasher(hash_builder),
        }
    }
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            updated: StateCell::new(()),
            map: HashMap::with_capacity_and_hasher(capacity, hash_builder),
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
        StateCell::invalidate(&mut self.updated);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<()> {
        self.map.remove(key)?;

        StateCell::invalidate(&mut self.updated);
        Some(())
    }

    pub fn keys(&self) -> Keys<K, V> {
        self.map.keys()
    }

    pub fn values(&self) -> Values<K, V> {
        self.map.values()
    }

    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        self.map.values_mut()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    pub fn clear(&mut self) {
        self.map.clear();
        StateCell::invalidate(&mut self.updated);
    }

    pub fn retain(&mut self, f: impl FnMut(&K, &mut V) -> bool) {
        self.map.retain(f);
        StateCell::invalidate(&mut self.updated);
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        StateCell::invalidate(&mut self.updated);
        self.map.entry(key)
    }

    pub fn iter(&self) -> Iter<K, V> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        self.map.iter_mut()
    }
}

impl<'a, K: Eq + Hash, V, S: BuildHasher> IntoIterator for &'a HashMapComponent<K, V, S> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a, K: Eq + Hash, V, S: BuildHasher> IntoIterator for &'a mut HashMapComponent<K, V, S> {
    type Item = (&'a K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> IntoIterator for HashMapComponent<K, V, S> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<K: Eq + Hash + Unpin, V: Unpin + AsyncComponent, S: Unpin> AsyncComponent
    for HashMapComponent<K, V, S>
{
    fn poll_next_state(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut poll = Poll::Pending;

        for component in self.map.values_mut() {
            if Pin::new(component).poll_next_state(cx).is_ready() && poll.is_pending() {
                poll = Poll::Ready(());
            }
        }

        if StateCell::poll_state(Pin::new(&mut self.updated), cx).is_ready() && poll.is_pending() {
            poll = Poll::Ready(());
        }

        poll
    }

    fn poll_next_stream(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut poll = Poll::Pending;

        for component in self.map.values_mut() {
            if Pin::new(component).poll_next_stream(cx).is_ready() && poll.is_pending() {
                poll = Poll::Ready(());
            }
        }

        poll
    }
}
