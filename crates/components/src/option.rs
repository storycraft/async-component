use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_component_core::{AsyncComponent, StateCell};

#[derive(Debug, Default)]
pub struct OptionComponent<T> {
    assigned: StateCell<()>,

    component: Option<T>,
}

impl<T: AsyncComponent> OptionComponent<T> {
    pub const fn new(component: Option<T>) -> Self {
        Self {
            assigned: StateCell::new(()),
            component,
        }
    }

    pub const fn is_none(&self) -> bool {
        self.component.is_none()
    }

    pub const fn is_some(&self) -> bool {
        self.component.is_some()
    }

    pub const fn get(&self) -> Option<&T> {
        self.component.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.component.as_mut()
    }

    pub fn take(&mut self) -> Option<()> {
        self.component.take()?;
        StateCell::invalidate(&mut self.assigned);

        Some(())
    }

    pub fn set(&mut self, component: Option<T>) {
        self.component = component;
        StateCell::invalidate(&mut self.assigned);
    }
}

impl<T: AsyncComponent> AsyncComponent for OptionComponent<T> {
    fn poll_next_state(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut poll = Poll::Pending;

        if let Some(ref mut component) = self.component {
            if Pin::new(component).poll_next_state(cx).is_ready() && poll.is_pending() {
                poll = Poll::Ready(());
            }
        }

        if StateCell::poll_state(Pin::new(&mut self.assigned), cx).is_ready() && poll.is_pending() {
            poll = Poll::Ready(());
        }

        poll
    }

    fn poll_next_stream(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if let Some(ref mut component) = self.component {
            Pin::new(component).poll_next_stream(cx)
        } else {
            Poll::Pending
        }
    }
}

impl<T: AsyncComponent> From<Option<T>> for OptionComponent<T> {
    fn from(opt: Option<T>) -> Self {
        Self::new(opt)
    }
}
