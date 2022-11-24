use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use async_component::{AsyncComponent, ComponentPollFlags};

pub struct SuspenseComponent<Fallback, Component> {
    current: SuspenseVariant<Fallback, Component>,
}

impl<F: AsyncComponent, C: AsyncComponent> SuspenseComponent<F, C> {
    pub fn new(fallback: F, fut: impl Future<Output = C> + 'static) -> Self {
        Self {
            current: SuspenseVariant::Loading {
                fallback,
                fut: Box::pin(fut),
            },
        }
    }

    pub const fn is_ready(&self) -> bool {
        matches!(self.current, SuspenseVariant::Loaded { .. })
    }

    pub const fn is_loading(&self) -> bool {
        matches!(self.current, SuspenseVariant::Loading { .. })
    }

    pub const fn get(&self) -> Result<&C, &F> {
        match self.current {
            SuspenseVariant::Loading { ref fallback, .. } => Err(fallback),
            SuspenseVariant::Loaded(ref component) => Ok(component),
        }
    }

    pub fn get_mut(&mut self) -> Result<&mut C, &mut F> {
        match self.current {
            SuspenseVariant::Loading {
                ref mut fallback, ..
            } => Err(fallback),
            SuspenseVariant::Loaded(ref mut component) => Ok(component),
        }
    }
}

impl<F: AsyncComponent, C: AsyncComponent> AsyncComponent for SuspenseComponent<F, C> {
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<ComponentPollFlags> {
        if let SuspenseVariant::Loading { ref mut fut, .. } = self.current {
            if let Poll::Ready(component) = Pin::new(fut).poll(cx) {
                self.current = SuspenseVariant::Loaded(component);
            }
        }

        match self.current {
            SuspenseVariant::Loading {
                ref mut fallback, ..
            } => Pin::new(fallback).poll_next(cx),

            SuspenseVariant::Loaded(ref mut component) => Pin::new(component).poll_next(cx),
        }
    }
}

enum SuspenseVariant<Fallback, Component> {
    Loading {
        fallback: Fallback,
        fut: Pin<Box<dyn Future<Output = Component>>>,
    },
    Loaded(Component),
}
