# Async component
Zero overhead reactive programming

## Example
See `async_component/examples/example.rs` for simple example.

See `examples/gui-demo` project for example using with gui(winit, raqote, pixels).

### Code
```Rust
use async_component::AsyncComponent;

// AsyncComponent trait only has update method which is like future and context is available globally when accessing component.
// Due to this behavior intergrating with async functions are very easy without async trait support. See StreamCell below.
#[derive(Debug, AsyncComponent)]
struct CounterComponent {
    // State must be wrapped with StateCell
    #[state(Self::on_counter_update)]
    counter: StateCell<i32>,

    // Stream
    // Async stream is polled using global component context
    #[state(Self::on_counter_recv)]
    counter_recv: StreamCell<Receiver<i32>>,
}

impl CounterComponent {
    fn on_counter_update(&mut self, _: ()) {
        println!("Counter updated to: {}", *self.counter);
    }

    fn on_counter_recv(&mut self, counter: i32) {
        *self.sub_component.counter = counter;
    }
}
```

Running this component stream will print initial value first and print changed value if new values are sent through channel.
```
Counter updated to: 0
Counter updated to: ...
```
