use std::{
    ops::RangeBounds,
    pin::Pin,
    slice::{self, Iter, IterMut},
    task::{Context, Poll},
    vec,
};

use async_component_core::{AsyncComponent, StateCell};

#[derive(Debug)]
pub struct VecComponent<T> {
    _state: StateCell<()>,

    vec: Vec<T>,
}

impl<T: AsyncComponent> VecComponent<T> {
    pub const fn new() -> Self {
        Self {
            _state: StateCell::new(()),
            vec: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            _state: StateCell::new(()),
            vec: Vec::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn remove(&mut self, index: usize) {
        self.vec.remove(index);
    }

    pub fn push(&mut self, component: T) {
        self.vec.push(component);
    }

    pub fn append(&mut self, other: &mut Vec<T>) {
        self.vec.append(other);
    }

    pub fn pop(&mut self) -> Option<()> {
        self.vec.pop()?;

        Some(())
    }

    pub fn drain(&mut self, range: impl RangeBounds<usize>) {
        self.vec.drain(range);
    }

    pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
        self.vec.retain(f);
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }
    
    pub fn iter(&self) -> Iter<T> {
        self.vec.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.vec.iter_mut()
    }
}

impl<T: AsyncComponent> Default for VecComponent<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: AsyncComponent> IntoIterator for &'a VecComponent<T> {
    type Item = &'a T;

    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: AsyncComponent> IntoIterator for &'a mut VecComponent<T> {
    type Item = &'a mut T;

    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: AsyncComponent> IntoIterator for VecComponent<T> {
    type Item = T;

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T: AsyncComponent> AsyncComponent for VecComponent<T> {
    fn poll_next_state(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut result = Poll::Pending;

        if StateCell::poll_state(Pin::new(&mut self._state), cx).is_ready() {
            result = Poll::Ready(());
        }

        for component in &mut self.vec {
            if Pin::new(component).poll_next_state(cx).is_ready() {
                if result.is_pending() {
                    result = Poll::Ready(());
                }
            }
        }

        result
    }

    fn poll_next_stream(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut result = Poll::Pending;

        for component in &mut self.vec {
            if Pin::new(component).poll_next_stream(cx).is_ready() {
                if result.is_pending() {
                    result = Poll::Ready(());
                }
            }
        }

        result
    }
}
