use std::ops::{Deref, DerefMut};

use async_component_core::AsyncComponent;

#[derive(Debug, Default)]
pub struct OptionComponent<T>(pub Option<T>);

impl<T> Deref for OptionComponent<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for OptionComponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsyncComponent> AsyncComponent for OptionComponent<T> {
    fn update_component(&mut self) {
        if let Some(ref mut inner) = self.0 {
            inner.update_component()
        }
    }
}
