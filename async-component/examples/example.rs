use std::time::Duration;

use async_component::{Component, StateCell, ComponentPollFlags};
use futures::{
    channel::mpsc::{channel, Receiver},
    pin_mut, SinkExt, Stream, StreamExt,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let (mut sender, recv) = channel(8);

    tokio::spawn(async move {
        let mut i = 0;
        loop {
            sleep(Duration::from_secs(1)).await;
            sender.send(i).await.ok();

            i += 1;
        }
    });

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

async fn run(component: impl Stream<Item = ComponentPollFlags> + Drawable) {
    pin_mut!(component);

    while let Some(flag) = component.next().await {
        // Redraw since last render is invalid
        if flag.contains(ComponentPollFlags::STATE) {
            component.draw();
        }
    }
}

#[derive(Debug, Component)]
struct CounterComponent {
    #[state(Self::on_counter_update)]
    pub counter: StateCell<i32>,
}

impl CounterComponent {
    fn on_counter_update(&mut self) {
        println!("Counter updated to: {}", *self.counter);
    }
}

impl Drawable for CounterComponent {
    fn draw(&self) {
        println!("===== Counter =====");
        println!("counter: {}", *self.counter);
        println!("===================");
    }
}

#[derive(Debug, Component)]
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
    fn on_id_update(&mut self) {
        println!("Id updated: {}", *self.id);
    }

    fn on_password_update(&mut self) {
        println!("Password updated: {}", *self.password);
    }

    fn update(&mut self) {
        println!("LoginForm updated: {:?}", self);
    }

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
