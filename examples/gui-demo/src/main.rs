mod env;

use async_component::{ AsyncComponent, StateCell, context::StateContext};
use async_component_winit::WinitComponent;
use env::{AppContainer, AppElement};
use raqote::{DrawOptions, DrawTarget, SolidSource, Source};
use winit::{
    event::{Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

// This example is simple reactive graphical application.
// * Draws white square cursor.
// * Draws magenta square to clicked position on left click.
// * Remove magenta square on right click.
// Efficiently redraw without messy explicit redraw codes using async-component.

fn main() {
    // Setup winit window
    let event_loop = EventLoopBuilder::with_user_event().build();

    let window = WindowBuilder::new()
        .with_title("Async component GUI demo")
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    // Start winit eventloop and run Executor using async_component_winit crate
    async_component_winit::run(event_loop, |cx| {
        AppContainer::new(window, App::new(cx))
    });
}

#[derive(AsyncComponent)]
pub struct App {
    // Cursor square
    #[component]
    cursor: Square,
}

impl App {
    pub fn new(cx: &StateContext) -> Self {
        Self {
            cursor: Square::new(
                cx,
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
    // Draw children elements
    fn draw(&self, target: &mut DrawTarget) {
        self.cursor.draw(target);
    }
}

impl WinitComponent for App {
    fn on_event(&mut self, event: &mut Event<()>, _: &mut ControlFlow) {
        match *event {
            // Update position state to actual cursor position
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { ref position, .. },
                ..
            } => {
                *self.cursor.position = (position.x as _, position.y as _);
            }

            // Add center_box element on left click
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                
            }

            // Take center_box element on right click
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button: MouseButton::Right,
                        ..
                    },
                ..
            } => {
            }

            _ => {}
        }
    }
}

// Square with position, size and source states.
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
    pub fn new(cx: &StateContext, position: (f32, f32), size: (f32, f32), source: Source<'static>) -> Self {
        Self {
            position: StateCell::new(cx.clone(), position),
            size: StateCell::new(cx.clone(), size),
            source: StateCell::new(cx.clone(), source),
        }
    }
}

impl AppElement for Square {
    // Draw rectangle
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
