use std::ops::{DerefMut, Deref};

use async_component_core::AsyncComponent;

#[derive(Debug)]
pub struct VecComponent<T>(pub Vec<T>);

impl<T> Deref for VecComponent<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for VecComponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsyncComponent> AsyncComponent for VecComponent<T> {
    fn update_component(&mut self) {
        for component in &mut self.0 {
            component.update_component();
        }
    }
}
