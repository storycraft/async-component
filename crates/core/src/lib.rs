#![doc = include_str!("../README.md")]

#[doc(hidden)]
#[path = "exports.rs"]
pub mod __private;
pub mod context;

use context::StateContext;
use futures_core::Stream;

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

/// Core trait
pub trait AsyncComponent {
    fn update_component(&mut self) -> bool;
}

pub trait State {
    type Output;

    fn update(this: &mut Self) -> Option<Self::Output>;
}

/// Track change of value and notify the Executor.
/// This struct has no method and implements [`Deref`], [`DerefMut`].
/// When inner value is mutable dereferenced, it changes status and wake pending task.
/// This will also wake pending task when the cell is dropped.
#[derive(Debug)]
pub struct StateCell<T> {
    cx: StateContext,
    changed: bool,
    inner: T,
}

impl<T> StateCell<T> {
    /// Create new [`StateCell`]
    pub fn new(cx: StateContext, inner: T) -> Self {
        cx.signal();

        Self {
            cx,
            changed: true,
            inner,
        }
    }

    /// Invalidate this [`StateCell`].
    /// It wakes task if there is any waker pending.
    pub fn invalidate(this: &mut Self) {
        if !this.changed {
            this.changed = true;
        }

        this.cx.signal()
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

impl<T> Drop for StateCell<T> {
    fn drop(&mut self) {
        self.cx.signal();
    }
}

#[derive(Debug)]
pub struct StreamCell<T> {
    cx: StateContext,
    inner: T,
}

impl<T: Stream> StreamCell<T> {
    pub fn new(cx: StateContext, inner: T) -> Self {
        cx.signal();

        Self { cx, inner }
    }
}

impl<T: Stream + Unpin> State for StreamCell<T> {
    type Output = T::Item;

    fn update(this: &mut Self) -> Option<Self::Output> {
        match Pin::new(&mut this.inner).poll_next(&mut Context::from_waker(this.cx.waker())) {
            Poll::Ready(Some(output)) => Some(output),
            _ => None,
        }
    }
}

impl<T> Drop for StreamCell<T> {
    fn drop(&mut self) {
        self.cx.signal();
    }
}
