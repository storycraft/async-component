#![doc = "../readme.md"]

pub use async_component_macro::Component;

#[doc(hidden)]
#[path = "exports.rs"]
pub mod __private;

use std::ops::{Deref, DerefMut};

use bitflags::bitflags;

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

    pub fn refresh(mut this: &mut Self) -> bool {
        match this.status {
            StateStatus::None => false,

            StateStatus::Changed => {
                this.status = StateStatus::None;

                true
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
