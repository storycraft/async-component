pub mod waker;

use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use async_component::AsyncComponent;
use waker::create_waker;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

pub trait WinitComponent {
    fn on_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow);
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ExecutorPollEvent;

pub fn run(
    event_loop: EventLoop<ExecutorPollEvent>,
    component: impl AsyncComponent + WinitComponent + 'static,
) -> ! {
    let mut component = component;

    let scheduled = Arc::new(AtomicBool::new(false));

    let proxy = event_loop.create_proxy();
    proxy.send_event(ExecutorPollEvent).ok();

    let waker = create_waker(scheduled.clone(), proxy.clone());
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(_) => {
                scheduled.store(false, Ordering::Release);
                if let Poll::Ready(_) =
                    Pin::new(&mut component).poll_next(&mut Context::from_waker(&waker))
                {
                    proxy.send_event(ExecutorPollEvent).ok();
                }
            }

            event => {
                component.on_event(event.map_nonuser_event().unwrap(), control_flow);
            }
        }
    });
}
