# Async component
Zero overhead reactive programming

## Example
See `async_component/examples/example.rs` for simple example.

See `examples/gui-demo` project for example using with gui(winit, raqote, pixels).

### Code
```Rust
use async_component::AsyncComponent;

#[derive(Debug, AsyncComponent)]
struct CounterComponent {
    // State must be wrapped with StateCell
    #[state(Self::on_counter_update)]
    counter: StateCell<i32>,

    // Stream
    // It iterates every queued items in single poll to prevent slowdown.
    // If the stream is immediate and resolves indefinitely, the task will fall to infinite loop. See expanded code below.
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

Running this component stream will print initial value first and print changed value if new values are sent through channel.
```
Counter updated to: 0
Counter updated to: ...
```

### Expanded
`Component` derive macro will generate `AsyncComponent` trait implementation for `CounterComponent` like below.
```Rust
impl AsyncComponent for CounterComponent {
    fn poll_next_state(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut result = Poll::Pending;

        if StateCell::poll_state(
            Pin::new(&mut self.counter),
            cx
        ) {
            Self::on_counter_update(&mut self);

            if result.is_pending() {
                result = Poll::Ready(());
            }
        }

       result
    }

    fn poll_next_stream(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut result = Poll::Pending;

        if let Poll::Ready(Some(recv)) = Stream::poll_next(Pin::new(&mut self.counter_recv), cx) {
            Self::on_counter_recv(&mut self, recv);

            if result.is_pending() {
                result = Poll::Ready(());
            }
        }

        result
    }
}
```
