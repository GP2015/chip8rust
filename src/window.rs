use crate::gpu::GPU;
use crate::input::InputManager;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;

const RESOLUTION_SCALAR: i32 = 10;
const WINDOW_TITLE: &str = "CHIP-8 Emulator";

pub struct WindowManager {
    active: Arc<AtomicBool>,
    window: Option<Window>,
    input: WinitInputHelper,
    gpu: Arc<GPU>,
    input_manager: Arc<InputManager>,
}

impl ApplicationHandler for WindowManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_inner_size(self.gpu.get_window_size())
            .with_title(WINDOW_TITLE);

        self.window = Some(event_loop.create_window(attributes).unwrap());
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if self.input.process_window_event(&event) {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        self.input.process_device_event(&event);
    }

    fn new_events(&mut self, _: &ActiveEventLoop, _: StartCause) {
        self.input.step();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.active.load(Ordering::Relaxed) {
            event_loop.exit();
            return;
        }

        self.input.end_step();

        if self.input.close_requested() || self.input.destroyed() {
            self.active.store(false, Ordering::Relaxed);
            event_loop.exit();
            return;
        }

        if self.input.key_pressed(KeyCode::KeyW) {
            println!("The 'W' key (US layout) was pressed on the keyboard");
        }
    }
}

impl WindowManager {
    pub fn new(active: Arc<AtomicBool>, gpu: Arc<GPU>, input_manager: Arc<InputManager>) -> Self {
        return Self {
            active,
            window: None,
            input: WinitInputHelper::new(),
            gpu,
            input_manager,
        };
    }
}
