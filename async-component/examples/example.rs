use std::time::Duration;

use async_component::{
    context::{ComponentStream, StateContext},
    AsyncComponent, StateCell, StreamCell,
};
use futures::{StreamExt, channel::mpsc::{Receiver, channel}, SinkExt};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let (mut sender, recv) = channel(8);

    // Spawn task that sends number with 1 sec interval
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            sleep(Duration::from_secs(1)).await;
            sender.send(i).await.ok();

            i += 1;
        }
    });

    // Run LoginForm component
    run(|cx| LoginForm {
        id: StateCell::new(cx.clone(), "user".to_string()),
        password: StateCell::new(cx.clone(), "1234".to_string()),

        sub_component: CounterComponent {
            counter: StateCell::new(cx.clone(), 0),
        },
        counter_recv: StreamCell::new(cx.clone(), recv),
    })
    .await;
}

trait Drawable {
    fn draw(&self);
}

// Run function
// Wait component for update and redraw each time updated.
async fn run<C: AsyncComponent + Drawable>(func: impl FnOnce(&StateContext) -> C) {
    let mut stream = ComponentStream::new(func);

    while let Some(_) = stream.next().await {
        stream.component().draw();
    }
}

// Component which draw counter on update
#[derive(Debug, AsyncComponent)]
struct CounterComponent {
    #[state]
    pub counter: StateCell<i32>,
}

impl Drawable for CounterComponent {
    fn draw(&self) {
        println!("===== Counter =====");
        println!("counter: {}", *self.counter);
        println!("===================");
    }
}

// Simple login form component which draw login form and have [`CounterComponent`] as child
#[derive(Debug, AsyncComponent)]
// Called if any states are updated
#[component(Self::update)]
struct LoginForm {
    #[state(Self::on_id_update)]
    id: StateCell<String>,

    #[state(Self::on_password_update)]
    password: StateCell<String>,

    #[component]
    sub_component: CounterComponent,

    #[state(Self::on_counter_recv)]
    counter_recv: StreamCell<Receiver<i32>>,
}

impl LoginForm {
    // Print message if self.id updated
    fn on_id_update(&mut self, _: ()) {
        println!("Id updated: {}", *self.id);
    }

    // Print message if self.password updated
    fn on_password_update(&mut self, _: ()) {
        println!("Password updated: {}", *self.password);
    }

    // Print message if component is updated
    fn update(&mut self) {
        println!("LoginForm updated: {:?}", self);
    }

    // Update sub component when counter number is received through channel
    fn on_counter_recv(&mut self, counter: i32) {
        *self.sub_component.counter = counter;
    }
}

impl Drawable for LoginForm {
    fn draw(&self) {
        println!("===== LoginForm =====");
        println!("id: {}", *self.id);
        println!("password: {}", *self.password);
        println!();
        println!("sub_component");
        self.sub_component.draw();
        println!();
        println!("=====================");
    }
}
