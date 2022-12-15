//! Specialized async Executor built on top of winit event loop for running [`AsyncComponent`]

pub mod signal;
#[cfg(target_arch = "wasm32")]
mod wasm;

use std::{
    sync::atomic::Ordering,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use async_component_core::{
    context::{ComponentStream, StateContext},
    AsyncComponent,
};
use futures::StreamExt;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use crate::WinitComponent;

use ref_extended::ref_extended;

use self::signal::WinitSignal;

/// Reserved zero sized user event struct used for waking winit eventloop
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ExecutorPollEvent;

/// Executor implemented on top of winit eventloop using user event.
///
/// See [`WinitSignal`] for more detail how it utilize winit user event.
#[derive(Debug)]
pub struct WinitExecutor {
    event_loop: Option<EventLoop<ExecutorPollEvent>>,

    state_signal: WinitSignal,
}

impl WinitExecutor {
    /// Create new [`WinitExecutor`]
    pub fn new(event_loop: EventLoop<ExecutorPollEvent>) -> Self {
        let state_signal = WinitSignal::new(event_loop.create_proxy());

        Self {
            event_loop: Some(event_loop),

            state_signal,
        }
    }

    fn poll_component(
        &'static self,
        component_stream: &mut ComponentStream<impl AsyncComponent>,
    ) -> Poll<()> {
        if self
            .state_signal
            .scheduled
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            if let Poll::Ready(Some(_)) =
                component_stream.poll_next_unpin(&mut Context::from_waker(&unsafe {
                    Waker::from_raw(create_raw_waker(&self.state_signal))
                }))
            {
                self.state_signal.scheduled.store(true, Ordering::Release);
            }

            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    /// Initializes the winit event loop and run component.
    ///
    /// See [`EventLoop`] for more detail about winit event loop
    pub fn run<C: AsyncComponent + WinitComponent + 'static>(
        mut self,
        func: impl FnOnce(&StateContext) -> C,
    ) -> ! {
        let event_loop = self.event_loop.take().unwrap();

        let mut stream = ComponentStream::new(func);

        let executor = self;
        ref_extended!(|&executor| event_loop.run(move |event, _, control_flow| {
            match event {
                Event::MainEventsCleared => {
                    stream
                        .component_mut()
                        .on_event(&mut Event::MainEventsCleared, control_flow);

                    if let ControlFlow::ExitWithCode(_) = control_flow {
                        return;
                    }

                    match executor.poll_component(&mut stream) {
                        Poll::Ready(_) => {
                            control_flow.set_poll();
                        }
                        Poll::Pending => {
                            control_flow.set_wait();
                        }
                    }
                }

                // Handled in Event::MainEventsCleared
                Event::UserEvent(_) => {}

                _ => {
                    stream
                        .component_mut()
                        .on_event(&mut event.map_nonuser_event().unwrap(), control_flow);
                }
            }
        }))
    }
}

fn create_raw_waker(signal: &'static WinitSignal) -> RawWaker {
    unsafe fn waker_clone(this: *const ()) -> RawWaker {
        create_raw_waker(&*(this as *const WinitSignal))
    }

    unsafe fn waker_wake(this: *const ()) {
        let this = &*(this as *const WinitSignal);
        this.wake_by_ref();
    }

    unsafe fn waker_wake_by_ref(this: *const ()) {
        let this = &*(this as *const WinitSignal);
        this.wake_by_ref();
    }

    unsafe fn waker_drop(_: *const ()) {}

    RawWaker::new(
        signal as *const _ as *const (),
        &RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop),
    )
}
