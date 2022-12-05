//! Implements [`WinitComponent`] trait to default components

use async_component_components::{OptionComponent, SuspenseComponent, VecComponent};
use async_component_core::AsyncComponent;
use winit::{event::Event, event_loop::ControlFlow};

use crate::WinitComponent;

impl<T: AsyncComponent + WinitComponent> WinitComponent for OptionComponent<T> {
    fn on_event(&mut self, event: &mut Event<()>, control_flow: &mut ControlFlow) {
        if let Some(component) = self.get_mut() {
            component.on_event(event, control_flow);
        }
    }
}

impl<T: AsyncComponent + WinitComponent> WinitComponent for VecComponent<T> {
    fn on_event(&mut self, event: &mut Event<()>, control_flow: &mut ControlFlow) {
        for component in self.into_iter() {
            component.on_event(event, control_flow);
        }
    }
}

impl<F: AsyncComponent + WinitComponent, T: AsyncComponent + WinitComponent> WinitComponent
    for SuspenseComponent<F, T>
{
    fn on_event(&mut self, event: &mut Event<()>, control_flow: &mut ControlFlow) {
        match self.get_mut() {
            Ok(component) => component.on_event(event, control_flow),
            Err(component) => component.on_event(event, control_flow),
        }
    }
}
