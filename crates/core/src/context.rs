use std::{
    cell::RefCell,
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

thread_local! {
    static CONTEXT: RefCell<Option<StateContext>> = RefCell::new(None);
}

pub fn with_current_context<R>(func: impl FnOnce(&StateContext) -> R) -> R {
    CONTEXT.with(|cx| match *cx.borrow() {
        Some(ref cx) => func(cx),
        None => panic!("Called without state context"),
    })
}

#[derive(Debug)]
struct EnterContextGuard {}

impl Drop for EnterContextGuard {
    fn drop(&mut self) {
        CONTEXT.with(|cell| {
            *cell.borrow_mut() = None;
        })
    }
}

fn enter_guarded(cx: StateContext) -> EnterContextGuard {
    CONTEXT.with(|cell| {
        {
            let mut cell = cell.borrow_mut();

            if cell.is_some() {
                panic!("State context is already set");
            }

            *cell = Some(cx);
        }

        EnterContextGuard {}
    })
}

#[derive(Debug)]
pub struct ComponentStream<C> {
    inner: Arc<Inner>,
    component: C,
}

impl<C: AsyncComponent> ComponentStream<C> {
    /// Create new [`ComponentStream`]
    pub fn new(func: impl FnOnce() -> C) -> Self {
        let inner = Arc::new(Inner::default());

        let component = {
            let _guard = enter_guarded(StateContext::new(Waker::from(inner.clone())));

            func()
        };

        Self { inner, component }
    }

    /// Enter context scope with stream
    pub fn enter<'a>(&'a mut self) -> EnteredComponentStream<'a, C> {
        EnteredComponentStream {
            _guard: enter_guarded(StateContext::new(Waker::from(self.inner.clone()))),
            stream: self,
        }
    }
}

#[derive(Debug)]
pub struct EnteredComponentStream<'a, C> {
    _guard: EnterContextGuard,
    stream: &'a mut ComponentStream<C>,
}

impl<C> EnteredComponentStream<'_, C> {
    pub fn component(&self) -> &C {
        &self.stream.component
    }

    pub fn component_mut(&mut self) -> &mut C {
        &mut self.stream.component
    }
}

impl<C: AsyncComponent> Stream for EnteredComponentStream<'_, C> {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<()>> {
        self.stream.inner.waker.register(cx.waker());

        if self.stream.inner.updated.swap(false, Ordering::Relaxed) {
            self.stream.component.update_component();
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
