pub mod executor;

use std::{pin::Pin, task::Poll};

use async_component::AsyncComponent;
use executor::WinitExecutor;
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
            Event::UserEvent(_) => {
                control_flow.set_poll();
            }

            _ => {
                component.on_event(event.map_nonuser_event().unwrap(), control_flow);
            }
        }
    });
}
