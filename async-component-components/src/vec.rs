use std::{
    ops::RangeBounds,
    pin::Pin,
    task::{Context, Poll},
    vec, slice,
};

use async_component::{AsyncComponent, ComponentPollFlags, StateCell};

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
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<ComponentPollFlags> {
        let mut result = ComponentPollFlags::empty();

        if StateCell::refresh(&mut self.updated) {
            result |= ComponentPollFlags::STATE;
        }

        for component in &mut self.vec {
            if let Poll::Ready(flag) = Pin::new(component).poll_next(cx) {
                result |= flag;
            }
        }

        if result.is_empty() {
            Poll::Pending
        } else {
            Poll::Ready(result)
        }
    }
}
