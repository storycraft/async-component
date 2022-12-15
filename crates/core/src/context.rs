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

    pub fn signal(&self) {
        self.inner.updated.store(true, Ordering::Relaxed);
        self.inner.waker.wake();
    }

    pub fn waker(&self) -> &Waker {
        &self.waker
    }
}

#[derive(Debug)]
pub struct Inner {
    updated: AtomicBool,
    waker: AtomicWaker,
}

impl Wake for Inner {
    fn wake(self: Arc<Self>) {
        self.waker.wake()
    }

    fn wake_by_ref(self: &Arc<Self>) {
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