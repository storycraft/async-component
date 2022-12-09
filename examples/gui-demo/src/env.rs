use async_component::AsyncComponent;
use async_component_winit::WinitComponent;
use pixels::{Pixels, SurfaceTexture};
use raqote::DrawTarget;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

// Trait for drawing graphical element
pub trait AppElement {
    fn draw(&self, target: &mut DrawTarget);
}

// Container component which handles rendering on child changes.
#[derive(Debug, AsyncComponent)]
pub struct AppContainer<T: AppElement + WinitComponent + AsyncComponent> {
    window: Window,

    pixels: Pixels,

    #[component(Self::on_update)]
    component: T,
}

impl<T: AppElement + WinitComponent + AsyncComponent> AppContainer<T> {
    // Create new [`AppContainer`]
    pub fn new(window: Window, component: T) -> Self {
        let window_size = window.inner_size();

        let pixels = {
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);

            Pixels::new(window_size.width, window_size.height, surface_texture).unwrap()
        };

        Self {
            window,
            pixels,
            component,
        }
    }

    // Called on if any child component's states are updated.
    fn on_update(&mut self) {
        // Request redraw
        self.window.request_redraw();
    }

    // Redraw child [`AppElement`]
    fn redraw(&mut self) {
        let (width, height) = self.window.inner_size().into();
        let mut target = DrawTarget::new(width, height);

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

impl<T: AppElement + WinitComponent + AsyncComponent> WinitComponent for AppContainer<T> {
    fn on_event(&mut self, event: &mut Event<()>, control_flow: &mut ControlFlow) {
        self.component.on_event(event, control_flow);

        match event {
            Event::RedrawRequested(_) => self.redraw(),

            // Resize surface on window resize
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                self.pixels.resize_buffer(new_size.width, new_size.height);
                self.pixels.resize_surface(new_size.width, new_size.height);
            }

            // Exit if close button is clicked
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => control_flow.set_exit(),

            _ => {}
        }
    }
}
