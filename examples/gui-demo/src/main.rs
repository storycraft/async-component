use std::thread;

use async_component::{AsyncComponent, AsyncComponentExt, ComponentPollFlags, StateCell};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt,
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

    let (mut sender, recv) = channel(1000);

    thread::spawn(move || {
        let app = App::new();
        let mut container = Container::new(pixels, window_size.into(), recv, app);

        futures::executor::block_on(async {
            loop {
                container.next().await;
            }
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

#[derive(AsyncComponent)]
pub struct App {
    #[component]
    center_box: Square,

    #[component]
    cursor: Square,
}

impl App {
    pub fn new() -> Self {
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
        }
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

#[derive(AsyncComponent)]
pub struct Square {
    #[state]
    pub position: StateCell<(f32, f32)>,

    #[state]
    pub size: StateCell<(f32, f32)>,

    #[state]
    pub source: StateCell<Source<'static>>,
}

impl Square {
    pub fn new(position: (f32, f32), size: (f32, f32), source: Source<'static>) -> Self {
        Self {
            position: position.into(),
            size: size.into(),
            source: source.into(),
        }
    }
}

impl AppElement for Square {
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
#[derive(AsyncComponent)]
struct Container<T: AppElement + AsyncComponent> {
    pixels: Pixels,
    win_size: (f32, f32),

    #[stream(Self::on_event)]
    event_recv: Receiver<Event<'static, ()>>,

    #[component(Self::on_update)]
    component: T,
}

impl<T: AppElement + AsyncComponent> Container<T> {
    pub fn new(
        pixels: Pixels,
        win_size: (f32, f32),
        event_recv: Receiver<Event<'static, ()>>,
        component: T,
    ) -> Self {
        Self {
            pixels,
            win_size,
            event_recv,
            component,
        }
    }

    fn on_update(&mut self, flag: ComponentPollFlags) {
        if flag.contains(ComponentPollFlags::STATE) {
            let mut target = DrawTarget::new(self.win_size.0 as _, self.win_size.1 as _);

            self.component.draw(&mut target);

            for (dst, &src) in self
                .pixels
                .get_frame_mut()
                .chunks_exact_mut(4)
                .zip(target.get_data().iter())
            {
                dst[0] = (src >> 16) as u8;
                dst[1] = (src >> 8) as u8;
                dst[2] = src as u8;
                dst[3] = (src >> 24) as u8;
            }

            self.pixels.render().unwrap();
        }
    }

    fn on_event(&mut self, event: Event<()>) {
        self.component.on_event(&event);
    }
}
