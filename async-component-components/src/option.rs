use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_component::{AsyncComponent, ComponentPollFlags, StateCell};

#[derive(Debug, Default)]
pub struct OptionComponent<T> {
    updated: StateCell<()>,

    component: Option<T>,
}

impl<T: AsyncComponent> OptionComponent<T> {
    pub const fn new(component: Option<T>) -> Self {
        Self {
            updated: StateCell::new(()),
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

        Some(())
    }

    pub fn set(&mut self, component: Option<T>) {
        self.component = component;
        StateCell::invalidate(&mut self.updated);
    }
}

impl<T: AsyncComponent> AsyncComponent for OptionComponent<T> {
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<ComponentPollFlags> {
        let mut result = ComponentPollFlags::empty();

        if StateCell::refresh(&mut self.updated) {
            result |= ComponentPollFlags::STATE;
        }

        if let Some(ref mut component) = self.component {
            if let Poll::Ready(flag) = Pin::new(component).poll_next(cx) {
                result |= flag;
            }
        }

        if result.is_empty() {
            Poll::Pending
        } else {
            Poll::Ready(result)
        }
    }
}
