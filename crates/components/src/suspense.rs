use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use async_component_core::{AsyncComponent, ComponentPollFlags};

pub struct SuspenseComponent<Fallback, Component>(SuspenseVariant<Fallback, Component>);

impl<F: AsyncComponent, C: AsyncComponent> SuspenseComponent<F, C> {
    pub fn new(fallback: F, fut: impl Future<Output = C> + 'static) -> Self {
        Self(SuspenseVariant::Loading {
            fallback,
            fut: Box::pin(fut),
        })
    }

    pub const fn is_ready(&self) -> bool {
        matches!(self.0, SuspenseVariant::Ready { .. })
    }

    pub const fn is_loading(&self) -> bool {
        matches!(self.0, SuspenseVariant::Loading { .. })
    }

    pub const fn get(&self) -> Result<&C, &F> {
        match self.0 {
            SuspenseVariant::Loading { ref fallback, .. } => Err(fallback),
            SuspenseVariant::Ready(ref component) => Ok(component),
        }
    }

    pub fn get_mut(&mut self) -> Result<&mut C, &mut F> {
        match self.0 {
            SuspenseVariant::Loading {
                ref mut fallback, ..
            } => Err(fallback),
            SuspenseVariant::Ready(ref mut component) => Ok(component),
        }
    }
}

impl<F: AsyncComponent, C: AsyncComponent> AsyncComponent for SuspenseComponent<F, C> {
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<ComponentPollFlags> {
        if let SuspenseVariant::Loading { ref mut fut, .. } = self.0 {
            if let Poll::Ready(component) = Pin::new(fut).poll(cx) {
                self.0 = SuspenseVariant::Ready(component);
            }
        }

        match self.0 {
            SuspenseVariant::Loading {
                ref mut fallback, ..
            } => Pin::new(fallback).poll_next(cx),

            SuspenseVariant::Ready(ref mut component) => Pin::new(component).poll_next(cx),
        }
    }
}

enum SuspenseVariant<Fallback, Component> {
    Loading {
        fallback: Fallback,
        fut: Pin<Box<dyn Future<Output = Component>>>,
    },
    Ready(Component),
}
