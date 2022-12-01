#![doc = "README.md"]

pub mod components;
pub mod executor;

use async_component_core::AsyncComponent;
use executor::{ExecutorStreamEvent, WinitExecutor};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

pub trait WinitComponent {
    fn on_event(&mut self, event: &mut Event<()>, control_flow: &mut ControlFlow);
}

pub fn run(
    event_loop: EventLoop<ExecutorStreamEvent>,
    component: impl AsyncComponent + WinitComponent + 'static,
) -> ! {
    let executor = WinitExecutor::new(event_loop);

    executor.run(component)
}
