pub use async_component_macro::Component;

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, Waker},
};

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

    pub fn set_changed(this: &mut Self) {
        match this.status {
            StateStatus::None => this.status = StateStatus::Changed,

            StateStatus::Pending(ref waker) => {
                waker.wake_by_ref();

                this.status = StateStatus::Changed;
            }

            StateStatus::Changed => {}
        }
    }

    pub fn poll_changed(mut this: Pin<&mut Self>, cx: &mut Context) -> Poll<()>
    where
        Self: Unpin,
    {
        match this.status {
            StateStatus::Pending(_) | StateStatus::None => {
                this.status = StateStatus::Pending(cx.waker().clone());

                Poll::Pending
            }

            StateStatus::Changed => {
                this.status = StateStatus::None;
                Poll::Ready(())
            }
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
        StateCell::set_changed(self);

        &mut self.inner
    }
}

impl<T> From<T> for StateCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Drop for StateCell<T> {
    fn drop(&mut self) {
        if let StateStatus::Pending(ref waker) = self.status {
            waker.wake_by_ref();
            self.status = StateStatus::Changed;
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateStatus {
    None,
    Pending(Waker),
    Changed,
}

impl Default for StateStatus {
    fn default() -> Self {
        Self::None
    }
}
