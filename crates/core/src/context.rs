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
    cx: StateContext,
    component: C,
}

impl<C: AsyncComponent> ComponentStream<C> {
    
    /// Create new [`ComponentStream`]
    pub fn new(func: impl FnOnce(&StateContext) -> C) -> Self {
        let cx = StateContext::new();

        let component = func(&cx);
        Self { cx, component }
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
        self.cx.inner.waker.register(cx.waker());

        self.component.update_component();
        if self.cx.inner.updated.swap(false, Ordering::Relaxed) {
            Poll::Ready(Some(()))
        } else {
            Poll::Pending
        }
    }
}

impl<C: AsyncComponent> Unpin for ComponentStream<C> {}

#[derive(Debug, Clone)]
pub struct StateContext {
    inner: Arc<Inner>,
    waker: Waker,
}

impl StateContext {
    pub(crate) fn new() -> Self {
        let inner = Arc::new(Inner::default());
        let waker = Waker::from(inner.clone());

        StateContext { inner, waker }
    }

    /// Signal context to wake
    pub fn signal(&self) {
        self.waker.wake_by_ref();
    }

    /// Returns [`Context`] which can be used for polling future
    pub fn task_context<'a>(&'a self) -> Context<'a> {
        Context::from_waker(&self.waker)
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
