#![doc = "../readme.md"]

#[doc(hidden)]
#[path = "exports.rs"]
pub mod __private;

pub use async_component_macro::AsyncComponent;

use futures_core::{Future, Stream};

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

use bitflags::bitflags;

pub trait AsyncComponent: Unpin {
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<ComponentPollFlags>;
}

impl<T: ?Sized + AsyncComponent> AsyncComponent for Box<T> {
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<ComponentPollFlags> {
        T::poll_next(Pin::new(&mut *self), cx)
    }
}

impl<T: ?Sized + AsyncComponent> AsyncComponent for &mut T {
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<ComponentPollFlags> {
        Pin::new(&mut **self).poll_next(cx)
    }
}

pub trait AsyncComponentExt {
    fn next(&mut self) -> Next<Self>;

    fn into_stream(self) -> AsyncComponentStream<Self>;
}

impl<T: AsyncComponent> AsyncComponentExt for T {
    fn next(&mut self) -> Next<Self> {
        Next(self)
    }

    fn into_stream(self) -> AsyncComponentStream<Self> {
        AsyncComponentStream(self)
    }
}

#[derive(Debug)]
pub struct Next<'a, T: ?Sized>(&'a mut T);

impl<T: AsyncComponent> Future for Next<'_, T> {
    type Output = ComponentPollFlags;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_next(cx)
    }
}

#[derive(Debug, Clone)]
pub struct AsyncComponentStream<T: ?Sized>(T);

impl<T: AsyncComponent> Stream for AsyncComponentStream<T> {
    type Item = ComponentPollFlags;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx).map(Some)
    }
}

#[derive(Debug)]
pub struct StateCell<T> {
    status: StateStatus,
    inner: T,
}

impl<T> StateCell<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            status: StateStatus::Changed,
            inner,
        }
    }

    pub fn invalidate(this: &mut Self) {
        if let StateStatus::None = this.status {
            this.status = StateStatus::Changed;
        }
    }

    pub fn refresh(this: &mut Self) -> bool {
        match this.status {
            StateStatus::Changed => {
                this.status = StateStatus::None;
                true
            }

            _ => false,
        }
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

impl<T> From<T> for StateCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone)]
enum StateStatus {
    None,
    Changed,
}

impl Default for StateStatus {
    fn default() -> Self {
        Self::None
    }
}

bitflags! {
    pub struct ComponentPollFlags: u32 {
        const STATE = 0b00000001;
        const STREAM = 0b00000010;
    }
}
