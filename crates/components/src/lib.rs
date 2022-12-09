#![doc = include_str!("../README.md")]

pub mod boxed;
pub mod map;
pub mod option;
pub mod suspense;
pub mod vec;

pub use boxed::BoxedComponent;
pub use map::HashMapComponent;
pub use option::OptionComponent;
pub use suspense::SuspenseComponent;
pub use vec::VecComponent;
