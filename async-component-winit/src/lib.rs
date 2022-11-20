pub mod waker;

use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll, Waker},
};

use async_component::{AsyncComponent, ComponentPollFlags};
use waker::create_waker;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

pub trait WinitComponent {
    fn on_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow);
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ExecutorPollEvent;

#[derive(Debug)]
struct WinitExecutor {
    scheduled: Arc<AtomicBool>,

    waker: Waker,
}

impl WinitExecutor {
    pub fn new(proxy: EventLoopProxy<ExecutorPollEvent>) -> Self {
        let scheduled = Arc::new(AtomicBool::new(false));

        let waker = create_waker(scheduled.clone(), proxy);

        Self { scheduled, waker }
    }

    pub fn poll_component(
        &self,
        component: Pin<&mut impl AsyncComponent>,
    ) -> Poll<ComponentPollFlags> {
        let _ = self
            .scheduled
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire);
        component.poll_next(&mut Context::from_waker(&self.waker))
    }
}

pub fn run(
    event_loop: EventLoop<ExecutorPollEvent>,
    component: impl AsyncComponent + WinitComponent + 'static,
) -> ! {
    let mut component = component;

    let executor = WinitExecutor::new(event_loop.create_proxy());

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::RedrawEventsCleared => {
                component.on_event(Event::RedrawEventsCleared, control_flow);

                if let Poll::Ready(_) = executor.poll_component(Pin::new(&mut component)) {
                    if let ControlFlow::ExitWithCode(_) = control_flow {
                        return;
                    }

                    control_flow.set_poll();
                }
            }

            // Event::RedrawEventsCleared
            Event::UserEvent(_) => {}

            _ => {
                component.on_event(event.map_nonuser_event().unwrap(), control_flow);
            }
        }
    });
}
