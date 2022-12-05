//! Specialized async Executor built on top of winit event loop for running [`AsyncComponent`]

pub mod signal;
#[cfg(target_arch = "wasm32")]
mod wasm;

use std::{
    pin::Pin,
    sync::{atomic::Ordering, Arc},
    task::{Context, Poll, Waker},
};

use async_component_core::AsyncComponent;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use crate::WinitComponent;

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

    state_signal: Arc<WinitSignal>,
    state_waker: Waker,

    stream_signal: Arc<WinitSignal>,
    stream_waker: Waker,
}

impl WinitExecutor {
    /// Create new [`WinitExecutor`]
    pub fn new(event_loop: EventLoop<ExecutorPollEvent>) -> Self {
        let stream_signal = Arc::new(WinitSignal::new(event_loop.create_proxy()));
        let stream_waker = Waker::from(stream_signal.clone());

        let state_signal = Arc::new(WinitSignal::new(event_loop.create_proxy()));
        let state_waker = Waker::from(state_signal.clone());

        Self {
            event_loop: Some(event_loop),

            state_signal,
            state_waker,

            stream_signal,
            stream_waker,
        }
    }

    fn poll_stream(&mut self, component: &mut impl AsyncComponent) {
        if let Ok(_) = self.stream_signal.scheduled.compare_exchange(
            true,
            false,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            while let Poll::Ready(_) = Pin::new(&mut *component)
                .poll_next_stream(&mut Context::from_waker(&self.stream_waker))
            {}
        }
    }

    fn poll_state(&mut self, component: &mut impl AsyncComponent) -> Poll<()> {
        if let Ok(_) = self.state_signal.scheduled.compare_exchange(
            true,
            false,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            if Pin::new(component)
                .poll_next_state(&mut Context::from_waker(&self.state_waker))
                .is_ready()
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
    pub fn run(mut self, mut component: impl AsyncComponent + WinitComponent + 'static) -> ! {
        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => {
                component.on_event(&mut Event::MainEventsCleared, control_flow);

                if let ControlFlow::ExitWithCode(_) = control_flow {
                    return;
                }

                self.poll_stream(&mut component);

                match self.poll_state(&mut component) {
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
                component.on_event(&mut event.map_nonuser_event().unwrap(), control_flow);
            }
        });
    }
}
