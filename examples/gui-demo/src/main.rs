mod env;

use async_component::{AsyncComponent, PhantomState, StateCell};
use env::{AppContainer, AppElement};
use raqote::{DrawOptions, DrawTarget, SolidSource, Source};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoopBuilder,
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoopBuilder::with_user_event().build();

    let window = WindowBuilder::new()
        .with_title("Async component GUI demo")
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    let app = App::new();

    async_component_winit::run(event_loop, AppContainer::new(window, app));
}

#[derive(AsyncComponent)]
pub struct App {
    #[component]
    center_box: Square,

    #[component]
    cursor: Square,

    #[state]
    _phantom: PhantomState,
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

            _phantom: Default::default(),
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
            // Do not set StateCell directly as it will not wake task
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
