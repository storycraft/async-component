use std::{
    pin::Pin,
    task::{Context, Poll}, ops::{Deref, DerefMut},
};

use async_component_core::{AsyncComponent, StateCell};

#[derive(Debug)]
pub struct BoxedComponent<T: ?Sized> {
    inner: Box<T>,
    changed: StateCell<()>,
}

impl<T: ?Sized> BoxedComponent<T> {
    pub const fn new(inner: Box<T>) -> Self {
        Self {
            inner,
            changed: StateCell::new(()),
        }
    }

    pub fn set(&mut self, inner: Box<T>) {
        self.inner = inner;

        StateCell::invalidate(&mut self.changed);
    }
}

impl<T: ?Sized> Deref for BoxedComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<T: ?Sized> DerefMut for BoxedComponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

impl<T: ?Sized> From<Box<T>> for BoxedComponent<T> {
    fn from(inner: Box<T>) -> Self {
        Self::new(inner)
    }
}

impl<T: ?Sized + AsyncComponent> AsyncComponent for BoxedComponent<T> {
    fn poll_next_state(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut poll = Poll::Pending;

        if Pin::new(&mut *self.inner).poll_next_state(cx).is_ready() {
            poll = Poll::Ready(());
        }

        if StateCell::poll_state(Pin::new(&mut self.changed), cx).is_ready() && poll.is_pending()
        {
            poll = Poll::Ready(());
        }

        poll
    }

    fn poll_next_stream(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        Pin::new(&mut *self.inner).poll_next_stream(cx)
    }
}
