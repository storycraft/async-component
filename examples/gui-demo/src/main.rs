use std::thread;

use async_component::{Component, ComponentPollFlags, StateCell};
use futures::{
    channel::mpsc::{channel, Receiver},
    pin_mut, SinkExt, Stream, StreamExt,
};
use pixels::{Pixels, SurfaceTexture};
use raqote::{DrawOptions, DrawTarget, SolidSource, Source};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Async component GUI demo")
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    let window_size = window.inner_size();

    let pixels = {
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(window_size.width, window_size.height, surface_texture).unwrap();

        pixels
    };

    let (mut sender, recv) = channel(10000);

    thread::spawn(move || {
        futures::executor::block_on(async move {
            let target = DrawTarget::new(window_size.width as _, window_size.height as _);

            let app = App::new(recv);
            run(pixels, target, app).await;
        });
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Some(event) = event.to_static() {
            futures::executor::block_on(sender.send(event)).ok();
        }
    });
}

trait AppElement {
    fn draw(&self, target: &mut DrawTarget);

    fn on_event(&mut self, _: &Event<()>) {}
}

#[derive(Component)]
pub struct App {
    #[component]
    center_box: Square<'static>,

    #[component]
    cursor: Square<'static>,

    #[stream(Self::on_event)]
    event_recv: Receiver<Event<'static, ()>>,
}

impl App {
    pub fn new(event_recv: Receiver<Event<'static, ()>>) -> Self {
        Self {
            center_box: Square::new(
                (100.0, 100.0),
                (100.0, 100.0),
                Source::Solid(SolidSource {
                    r: 0xff,
                    g: 0x00,
                    b: 0xff,
                    a: 0xff,
                }),
            ),
            cursor: Square::new(
                (0.0, 0.0),
                (20.0, 20.0),
                Source::Solid(SolidSource {
                    r: 0xff,
                    g: 0xff,
                    b: 0xff,
                    a: 0xff,
                }),
            ),
            event_recv,
        }
    }

    // This is ok. However when using top-down propagated global events like this, consider calling [`AppElement::onevent`] on [`run`] method for more efficiency.
    fn on_event(&mut self, event: Event<()>) {
        <Self as AppElement>::on_event(self, &event);
    }
}

impl AppElement for App {
    fn draw(&self, target: &mut DrawTarget) {
        self.center_box.draw(target);
        self.cursor.draw(target);
    }

    fn on_event(&mut self, event: &Event<()>) {
        self.center_box.on_event(event);
        self.cursor.on_event(event);

        if let Event::WindowEvent {
            event: WindowEvent::CursorMoved { ref position, .. },
            ..
        } = event
        {
            *self.cursor.position = (position.x as _, position.y as _);
        }
    }
}

#[derive(Component)]
pub struct Square<'a> {
    #[state]
    pub position: StateCell<(f32, f32)>,

    #[state]
    pub size: StateCell<(f32, f32)>,

    #[state]
    pub source: StateCell<Source<'a>>,
}

impl<'a> Square<'a> {
    pub fn new(position: (f32, f32), size: (f32, f32), source: Source<'a>) -> Self {
        Self {
            position: position.into(),
            size: size.into(),
            source: source.into(),
        }
    }
}

impl AppElement for Square<'_> {
    fn draw(&self, target: &mut DrawTarget) {
        target.fill_rect(
            self.position.0,
            self.position.1,
            self.size.0,
            self.size.1,
            &self.source,
            &DrawOptions::default(),
        );
    }
}

async fn run(
    mut pixels: Pixels,
    mut target: DrawTarget,
    component: impl Stream<Item = ComponentPollFlags> + AppElement,
) {
    pin_mut!(component);

    while let Some(flag) = component.next().await {
        if flag.contains(ComponentPollFlags::STATE) {
            target.clear(SolidSource {
                r: 0x00,
                g: 0x00,
                b: 0x00,
                a: 0x00,
            });

            component.draw(&mut target);

            for (dst, &src) in pixels
                .get_frame_mut()
                .chunks_exact_mut(4)
                .zip(target.get_data().iter())
            {
                dst[0] = (src >> 16) as u8;
                dst[1] = (src >> 8) as u8;
                dst[2] = src as u8;
                dst[3] = (src >> 24) as u8;
            }

            pixels.render().unwrap();
        }
    }
}
