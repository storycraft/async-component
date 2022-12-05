use std::time::Duration;

use async_component::{AsyncComponent, AsyncComponentExt, StateCell};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt,
};
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
    run(LoginForm {
        id: "user".to_string().into(),
        password: "1234".to_string().into(),

        sub_component: CounterComponent { counter: 0.into() },
        counter_recv: recv,
    })
    .await;
}

trait Drawable {
    fn draw(&self);
}

// Run function
// Wait component for update and redraw each time updated.
async fn run(mut component: impl AsyncComponent + Drawable) {
    loop {
        component.next().await;

        // Redraw since last render is invalid
        component.draw();
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

    #[stream(Self::on_counter_recv)]
    counter_recv: Receiver<i32>,
}

impl LoginForm {
    // Print message if self.id updated
    fn on_id_update(&mut self) {
        println!("Id updated: {}", *self.id);
    }

    // Print message if self.password updated
    fn on_password_update(&mut self) {
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
