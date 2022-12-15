use std::{
    collections::{hash_map::RandomState, HashMap},
    hash::Hash,
    ops::{Deref, DerefMut},
};

use async_component_core::AsyncComponent;

#[derive(Debug)]
pub struct HashMapComponent<K, V, S = RandomState>(pub HashMap<K, V, S>);

impl<K, V, S> Deref for HashMapComponent<K, V, S> {
    type Target = HashMap<K, V, S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, S> DerefMut for HashMapComponent<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K: Eq + Hash, V: AsyncComponent, S> AsyncComponent for HashMapComponent<K, V, S> {
    fn update_component(&mut self) -> bool {
        let mut updated = false;

        for value in self.0.values_mut() {
            if value.update_component() && !updated {
                updated = true;
            }
        }

        updated
    }
}
