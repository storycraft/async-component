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
    length_updated: StateCell<()>,
    vec: Vec<T>,
}

impl<T: AsyncComponent> VecComponent<T> {
    pub const fn new() -> Self {
        Self {
            length_updated: StateCell::new(()),
            vec: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            length_updated: StateCell::new(()),
            vec: Vec::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn remove(&mut self, index: usize) {
        self.vec.remove(index);
        StateCell::invalidate(&mut self.length_updated);
    }

    pub fn push(&mut self, component: T) {
        self.vec.push(component);
        StateCell::invalidate(&mut self.length_updated);
    }

    pub fn append(&mut self, other: &mut Vec<T>) {
        self.watch_len_changes(move |this| {
            this.vec.append(other);
        });
    }

    pub fn pop(&mut self) -> Option<()> {
        self.vec.pop()?;
        StateCell::invalidate(&mut self.length_updated);

        Some(())
    }

    pub fn drain(&mut self, range: impl RangeBounds<usize>) {
        self.watch_len_changes(move |this| {
            this.vec.drain(range);
        });
    }

    pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
        self.watch_len_changes(move |this| {
            this.vec.retain(f);
        });
    }

    pub fn clear(&mut self) {
        self.watch_len_changes(|this| {
            this.vec.clear();
        });
    }

    pub fn iter(&self) -> Iter<T> {
        self.vec.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.vec.iter_mut()
    }

    fn watch_len_changes(&mut self, func: impl FnOnce(&mut Self)) {
        let original_len = self.vec.len();
        func(self);
        let len = self.vec.len();

        if original_len != len {
            StateCell::invalidate(&mut self.length_updated);
        }
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
        let mut poll = Poll::Pending;

        for component in &mut self.vec {
            if Pin::new(component).poll_next_state(cx).is_ready() && poll.is_pending() {
                poll = Poll::Ready(());
            }
        }

        if StateCell::poll_state(Pin::new(&mut self.length_updated), cx).is_ready() && poll.is_pending() {
            poll = Poll::Ready(());
        }

        poll
    }

    fn poll_next_stream(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut poll = Poll::Pending;

        for component in &mut self.vec {
            if Pin::new(component).poll_next_stream(cx).is_ready() && poll.is_pending() {
                poll = Poll::Ready(());
            }
        }

        poll
    }
}
