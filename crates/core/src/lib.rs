#![doc = "../README.md"]

#[doc(hidden)]
#[path = "exports.rs"]
pub mod __private;

use futures_core::Future;

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, Waker},
};

/// Core trait
pub trait AsyncComponent: Unpin {
    fn poll_next_state(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()>;

    fn poll_next_stream(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()>;
}

impl<T: ?Sized + AsyncComponent> AsyncComponent for Box<T> {
    fn poll_next_state(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        T::poll_next_state(Pin::new(&mut *self), cx)
    }

    fn poll_next_stream(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        T::poll_next_stream(Pin::new(&mut *self), cx)
    }
}

impl<T: ?Sized + AsyncComponent> AsyncComponent for &mut T {
    fn poll_next_state(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        T::poll_next_state(Pin::new(*self), cx)
    }

    fn poll_next_stream(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        T::poll_next_stream(Pin::new(*self), cx)
    }
}

pub trait AsyncComponentExt {
    fn next(&mut self) -> Next<Self>;

    fn next_state(&mut self) -> NextState<Self>;

    fn next_stream(&mut self) -> NextStream<Self>;
}

#[derive(Debug)]
pub struct Next<'a, T: ?Sized>(&'a mut T);

impl<T: AsyncComponent> Future for Next<'_, T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut result = Poll::Pending;

        if Pin::new(&mut *self.0).poll_next_stream(cx).is_ready() {
            result = Poll::Ready(());
        }

        if Pin::new(&mut *self.0).poll_next_state(cx).is_ready() {
            result = Poll::Ready(());
        }

        result
    }
}

impl<T: AsyncComponent> AsyncComponentExt for T {
    fn next(&mut self) -> Next<Self> {
        Next(self)
    }

    fn next_state(&mut self) -> NextState<Self> {
        NextState(self)
    }

    fn next_stream(&mut self) -> NextStream<Self> {
        NextStream(self)
    }
}

#[derive(Debug)]
pub struct NextState<'a, T: ?Sized>(&'a mut T);

impl<T: AsyncComponent> Future for NextState<'_, T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_next_state(cx)
    }
}

#[derive(Debug)]
pub struct NextStream<'a, T: ?Sized>(&'a mut T);

impl<T: AsyncComponent> Future for NextStream<'_, T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_next_stream(cx)
    }
}

pub type PhantomState = StateCell<()>;

/// Track change of value and notify the Executor.
/// This struct has no method and implements [`Deref`], [`DerefMut`].
/// When inner value is mutable dereferenced, it changes status and wake pending task.
/// This will also wake pending task when the cell is dropped.
#[derive(Debug)]
pub struct StateCell<T> {
    status: StateStatus,
    inner: T,
}

impl<T> StateCell<T> {
    /// Create new [`StateCell`]
    pub const fn new(inner: T) -> Self {
        Self {
            status: StateStatus::Changed,
            inner,
        }
    }

    /// Invalidate this [`StateCell`].
    /// It wakes task if there is any waker pending.
    pub fn invalidate(this: &mut Self) {
        match this.status {
            StateStatus::None => {
                this.status = StateStatus::Changed;
            }

            StateStatus::Pending(ref waker) => {
                waker.wake_by_ref();
                this.status = StateStatus::Changed;
            }

            StateStatus::Changed => {}
        }
    }

    /// Check if there are any changes or saves waker to wake task to notify when the value is changed.
    pub fn poll_state(mut this: Pin<&mut Self>, cx: &mut Context) -> Poll<()>
    where
        Self: Unpin,
    {
        match this.status {
            StateStatus::None => {
                this.status = StateStatus::Pending(cx.waker().clone());

                Poll::Pending
            }

            StateStatus::Pending(ref old_waker) => {
                if !old_waker.will_wake(cx.waker()) {
                    this.status = StateStatus::Pending(cx.waker().clone());
                }

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
        StateCell::invalidate(self);

        &mut self.inner
    }
}

impl<T> From<T> for StateCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Default> Default for StateCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Drop for StateCell<T> {
    fn drop(&mut self) {
        if let StateStatus::Pending(ref waker) = self.status {
            waker.wake_by_ref();
        }
    }
}

#[derive(Debug, Clone)]
enum StateStatus {
    None,
    Pending(Waker),
    Changed,
}

impl Default for StateStatus {
    fn default() -> Self {
        Self::None
    }
}
