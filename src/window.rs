use crate::gpu::GPU;
use crate::input::InputManager;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;

const RESOLUTION_SCALAR: i32 = 10;
const WINDOW_TITLE: &str = "CHIP-8 Emulator";

pub struct WindowManager {
    active: Arc<AtomicBool>,
    window: Option<Window>,
    gpu: Arc<GPU>,
    input_manager: Arc<InputManager>,
}

impl ApplicationHandler for WindowManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_inner_size(self.gpu.get_window_size())
            .with_title(WINDOW_TITLE);

        self.window = Some(event_loop.create_window(attributes).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

impl WindowManager {
    pub fn new(active: Arc<AtomicBool>, gpu: Arc<GPU>, input_manager: Arc<InputManager>) -> Self {
        return Self {
            active,
            gpu,
            input_manager,
            window: None,
        };
    }
}
