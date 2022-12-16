use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll, Wake, Waker},
};

use atomic_waker::AtomicWaker;
use futures_core::Stream;

use crate::AsyncComponent;

#[derive(Debug)]
pub struct ComponentStream<C> {
    inner: Arc<Inner>,
    component: C,
}

impl<C: AsyncComponent> ComponentStream<C> {
    /// Create new [`ComponentStream`]
    pub fn new(func: impl FnOnce(&StateContext) -> C) -> Self {
        let inner = Arc::new(Inner::default());

        let cx = StateContext::new(Waker::from(inner.clone()));
        let component = func(&cx);

        Self { inner, component }
    }

    pub fn component(&self) -> &C {
        &self.component
    }

    pub fn component_mut(&mut self) -> &mut C {
        &mut self.component
    }
}

impl<C: AsyncComponent> Stream for ComponentStream<C> {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<()>> {
        self.inner.waker.register(cx.waker());

        self.component.update_component();
        if self.inner.updated.swap(false, Ordering::Relaxed) {
            Poll::Ready(Some(()))
        } else {
            Poll::Pending
        }
    }
}

impl<C: AsyncComponent> Unpin for ComponentStream<C> {}

#[derive(Debug, Clone)]
pub struct StateContext(Waker);

impl StateContext {
    pub(crate) const fn new(waker: Waker) -> Self {
        StateContext(waker)
    }

    /// Signal context to wake
    pub fn signal(&self) {
        self.0.wake_by_ref();
    }

    /// Returns [`Context`] which can be used for polling future
    pub fn task_context<'a>(&'a self) -> Context<'a> {
        Context::from_waker(&self.0)
    }
}

#[derive(Debug)]
struct Inner {
    updated: AtomicBool,
    waker: AtomicWaker,
}

impl Wake for Inner {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.updated.store(true, Ordering::Relaxed);
        self.waker.wake()
    }
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            updated: AtomicBool::new(true),
            waker: Default::default(),
        }
    }
}
