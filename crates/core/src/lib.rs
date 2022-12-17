#![doc = include_str!("../README.md")]

#[doc(hidden)]
#[path = "exports.rs"]
pub mod __private;
pub mod context;

use context::current_context;
use futures_core::Stream;

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::Poll,
};

/// Core trait
pub trait AsyncComponent {
    fn update_component(&mut self);
}

/// State trait
///
/// Returns output if state is updated
pub trait State {
    type Output;

    fn update(this: &mut Self) -> Option<Self::Output>;
}

/// Track change of value and signal to [`StateContext`].
/// This struct has no method and implements [`Deref`], [`DerefMut`].
/// When inner value is mutable dereferenced, it is marked changed and send signal.
/// This will also send signal when the cell is constructed or dropped.
#[derive(Debug)]
pub struct StateCell<T> {
    changed: bool,
    inner: T,
}

impl<T> StateCell<T> {
    /// Create new [`StateCell`]
    pub fn new(inner: T) -> Self {
        current_context().signal();

        Self {
            changed: true,
            inner,
        }
    }

    /// Invalidate this [`StateCell`].
    /// Send signal to context.
    pub fn invalidate(this: &mut Self) {
        if !this.changed {
            this.changed = true;
        }

        current_context().signal();
    }
}

impl<T> Deref for StateCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for StateCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        StateCell::invalidate(self);

        &mut self.inner
    }
}

impl<T> State for StateCell<T> {
    type Output = ();

    fn update(this: &mut Self) -> Option<Self::Output> {
        if this.changed {
            this.changed = false;
            Some(())
        } else {
            None
        }
    }
}

impl<T> From<T> for StateCell<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T: Default> Default for StateCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Drop for StateCell<T> {
    fn drop(&mut self) {
        current_context().signal();
    }
}

/// State which polls inner stream
#[derive(Debug)]
pub struct StreamCell<T> {
    inner: T,
}

impl<T: Stream> StreamCell<T> {
    pub fn new(inner: T) -> Self {
        current_context().signal();
        Self { inner }
    }
}

impl<T: Stream + Unpin> State for StreamCell<T> {
    type Output = T::Item;

    fn update(this: &mut Self) -> Option<Self::Output> {
        match Pin::new(&mut this.inner).poll_next(&mut current_context().task_context()) {
            Poll::Ready(Some(output)) => Some(output),
            _ => None,
        }
    }
}

impl<T: Stream> From<T> for StreamCell<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T: Stream + Default> Default for StreamCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Drop for StreamCell<T> {
    fn drop(&mut self) {
        current_context().signal();
    }
}
