use std::ops::{Deref, DerefMut};

use async_component_core::AsyncComponent;

#[derive(Debug)]
pub struct BoxedComponent<T: ?Sized>(pub Box<T>);

impl<T: ?Sized> Deref for BoxedComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T: ?Sized> DerefMut for BoxedComponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<T: ?Sized + AsyncComponent> AsyncComponent for BoxedComponent<T> {
    fn update_component(&mut self) -> bool {
        self.0.update_component()
    }
}
