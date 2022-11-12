# Async component
Zero overhead component composing using async iterator (stream)

The current goal is composing gui components easily without performance degrade in Rust.

## Concept
UI components are retained. It only need to recalculate some layout when properties are changed.

States are never updated unless some event occur from outside.

Using stream, we can poll for events from outside and apply update of changed value simultaneously.

## Example
See `examples/example.rs` for working example.

### Code
```Rust
use async_component::Component;

#[derive(Debug, Component)]
struct CounterComponent {
    // State must be wrapped with StateCell
    #[state(Self::on_counter_update)]
    counter: StateCell<i32>,

    // Stream or child components
    #[stream(Self::on_counter_recv)]
    counter_recv: Receiver<i32>,
}

impl CounterComponent {
    fn on_counter_update(&mut self) {
        println!("Counter updated to: {}", *self.counter);
    }

    fn on_counter_recv(&mut self, counter: i32) {
        *self.sub_component.counter = counter;
    }
}
```

Running this component stream will print initial value first and print changed value when new values are sent from channel.
```
Counter updated to: 0
Counter updated to: ...
```

### Expanded
Codes like this will be generated
```Rust
use async_component::StateCell;
use futures::Stream;

use std::{pin::Pin, task::{Poll, Context}};

#[derive(Debug, Component)]
struct CounterComponent {
    // State
    #[state(Self::on_counter_update)]
    counter: StateCell<i32>,

    // Stream or child components
    #[stream(Self::on_counter_recv)]
    counter_recv: Receiver<i32>,
}

impl Stream for CounterComponent {
    type Item = ComponentPollFlags;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut result = ComponentPollFlags::empty();

        if StateCell::poll_changed(
            Pin::new(&mut self.counter),
            cx
        ).is_ready() {
            Self::on_counter_update(&mut self);
            result |= ComponentPollFlags::STATE;
        }
        
        if let Poll::Ready(Some(recv)) = Stream::poll_next(Pin::new(&mut self.counter_recv), cx) {
            Self::on_counter_recv(&mut self, recv);
            result |= ComponentPollFlags::STREAM;
        }

        if result.is_empty() {
            Poll::Pending
        } else {
            Poll::Ready(Some(result))
        }
    }
}

impl CounterComponent {
    fn on_counter_update(&mut self) {
        println!("Counter updated to: {}", *self.counter);
    }

    fn on_counter_recv(&mut self, counter: i32) {
        *self.sub_component.counter = counter;
    }
}
```
