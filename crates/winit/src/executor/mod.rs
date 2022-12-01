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

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ExecutorStreamEvent;

#[derive(Debug)]
pub struct WinitExecutor<T> {
    event_loop: Option<EventLoop<ExecutorStreamEvent>>,
    component: T,

    state_signal: Arc<WinitSignal>,
    state_waker: Waker,

    stream_signal: Arc<WinitSignal>,
    stream_waker: Waker,
}

impl<T: AsyncComponent + WinitComponent> WinitExecutor<T> {
    pub fn new(event_loop: EventLoop<ExecutorStreamEvent>, component: T) -> Self {
        let stream_signal = Arc::new(WinitSignal::new(event_loop.create_proxy()));
        let stream_waker = Waker::from(stream_signal.clone());

        let state_signal = Arc::new(WinitSignal::new(event_loop.create_proxy()));
        let state_waker = Waker::from(stream_signal.clone());

        Self {
            event_loop: Some(event_loop),
            component,

            state_signal,
            state_waker,

            stream_signal,
            stream_waker,
        }
    }

    fn poll_stream(&mut self) {
        if let Ok(_) = self.stream_signal.scheduled.compare_exchange(
            true,
            false,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            while let Poll::Ready(_) = Pin::new(&mut self.component)
                .poll_next_stream(&mut Context::from_waker(&self.stream_waker))
            {}
        }
    }

    fn poll_state(&mut self) -> Poll<()> {
        let _ = self.state_signal.scheduled.compare_exchange(
            true,
            false,
            Ordering::AcqRel,
            Ordering::Acquire,
        );

        Pin::new(&mut self.component).poll_next_state(&mut Context::from_waker(&self.state_waker))
    }

    pub fn run(mut self) -> !
    where
        T: 'static,
    {
        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => {
                self.component
                    .on_event(&mut Event::MainEventsCleared, control_flow);

                if let ControlFlow::ExitWithCode(_) = control_flow {
                    return;
                }

                self.poll_stream();

                match self.poll_state() {
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
                self.component
                    .on_event(&mut event.map_nonuser_event().unwrap(), control_flow);
            }
        });
    }
}
