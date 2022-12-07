#![doc = include_str!("../README.md")]

pub mod components;
pub mod executor;

use async_component_core::AsyncComponent;
use executor::{ExecutorPollEvent, WinitExecutor};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

/// Trait for handling winit events on component.
pub trait WinitComponent {
    fn on_event(&mut self, event: &mut Event<()>, control_flow: &mut ControlFlow);
}

/// Convenience method for initializing executor and running winit eventloop
pub fn run(
    event_loop: EventLoop<ExecutorPollEvent>,
    component: impl AsyncComponent + WinitComponent + 'static,
) -> ! {
    let executor = WinitExecutor::new(event_loop);

    executor.run(component)
}
