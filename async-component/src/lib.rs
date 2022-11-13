pub use async_component_macro::Component;
use pin_project::{pin_project, pinned_drop};

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, Waker},
};

use bitflags::bitflags;

#[derive(Debug)]
#[pin_project(PinnedDrop)]
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

    pub fn poll_changed(mut this: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
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

#[pinned_drop]
impl<T> PinnedDrop for StateCell<T> {
    fn drop(mut self: Pin<&mut Self>) {
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

bitflags! {
    pub struct ComponentPollFlags: u32 {
        const STATE = 0b00000001;
        const STREAM = 0b00000010;
    }
}
