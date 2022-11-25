#![doc = "README.md"]

pub mod executor;

use std::pin::Pin;

use async_component_core::AsyncComponent;
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

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawEventsCleared => {
            component.on_event(Event::RedrawEventsCleared, control_flow);

            if let ControlFlow::ExitWithCode(_) = control_flow {
                return;
            }

            if executor
                .poll_component(Pin::new(&mut component))
                .is_pending()
            {
                control_flow.set_wait();
            }
        }

        Event::UserEvent(_) => {
            let _ = executor.poll_component(Pin::new(&mut component));
        }

        _ => {
            component.on_event(event.map_nonuser_event().unwrap(), control_flow);

            let _ = executor.poll_component(Pin::new(&mut component));
        }
    });
}
