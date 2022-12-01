use std::{
    ops::RangeBounds,
    pin::Pin,
    slice,
    task::{Context, Poll},
    vec,
};

use async_component_core::{AsyncComponent, StateCell};

#[derive(Debug)]
pub struct VecComponent<T> {
    updated: StateCell<()>,

    vec: Vec<T>,
}

impl<T: AsyncComponent> VecComponent<T> {
    pub const fn new() -> Self {
        Self {
            updated: StateCell::new(()),
            vec: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            updated: StateCell::new(()),
            vec: Vec::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn remove(&mut self, index: usize) {
        self.vec.remove(index);
        StateCell::invalidate(&mut self.updated);
    }

    pub fn push(&mut self, component: T) {
        self.vec.push(component);
        StateCell::invalidate(&mut self.updated);
    }

    pub fn append(&mut self, other: &mut Vec<T>) {
        self.vec.append(other);
        StateCell::invalidate(&mut self.updated);
    }

    pub fn pop(&mut self) -> Option<()> {
        self.vec.pop()?;
        StateCell::invalidate(&mut self.updated);

        Some(())
    }

    pub fn drain(&mut self, range: impl RangeBounds<usize>) {
        self.vec.drain(range);
        StateCell::invalidate(&mut self.updated);
    }

    pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
        self.vec.retain(f);
        StateCell::invalidate(&mut self.updated);
    }

    pub fn clear(&mut self) {
        self.vec.clear();
        StateCell::invalidate(&mut self.updated);
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
        self.vec.iter()
    }
}

impl<'a, T: AsyncComponent> IntoIterator for &'a mut VecComponent<T> {
    type Item = &'a mut T;

    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter_mut()
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

        if StateCell::poll_state(Pin::new(&mut self.updated), cx).is_ready() {
            result = Poll::Ready(());
        }

        for component in &mut self.vec {
            if Pin::new(component).poll_next_state(cx).is_ready() {
                result = Poll::Ready(());
            }
        }

        result
    }

    fn poll_next_stream(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut result = Poll::Pending;

        for component in &mut self.vec {
            if Pin::new(component).poll_next_stream(cx).is_ready() {
                result = Poll::Ready(());
            }
        }

        result
    }
}
