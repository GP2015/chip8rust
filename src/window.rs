use crate::gpu::GPU;
use crate::input::InputManager;
use softbuffer::{Buffer, Context, Surface};
use std::cmp;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::{Window, WindowButtons, WindowId};
use winit_input_helper::WinitInputHelper;

const WINDOW_TITLE: &str = "CHIP-8 Emulator";
const BASE_RESOLUTION_SCALAR: NonZeroU32 = NonZeroU32::new(20).unwrap();

struct Size {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

impl Size {
    pub fn new(width: NonZeroU32, height: NonZeroU32) -> Self {
        Self { width, height }
    }

    pub fn from_usize(width: usize, height: usize) -> Self {
        Self {
            width: NonZeroU32::new(width as u32).unwrap(),
            height: NonZeroU32::new(height as u32).unwrap(),
        }
    }

    pub fn get_width_usize(&self) -> usize {
        return self.width.get() as usize;
    }

    pub fn get_height_usize(&self) -> usize {
        return self.height.get() as usize;
    }
}

struct Position {
    pub index: usize,
    pub x: usize,
    pub y: usize,
    width: usize,
}

impl Position {
    pub fn from_pos(x: usize, y: usize, width: usize) -> Self {
        Self {
            index: width * y + x,
            x,
            y,
            width,
        }
    }

    pub fn from_index(index: usize, width: usize) -> Self {
        Self {
            index,
            x: index % width,
            y: index / width,
            width,
        }
    }

    fn update_index(&mut self) {
        self.index = self.width * self.y + self.x;
    }

    pub fn scale(mut self, factor: usize) -> Self {
        self.width *= factor;
        self.x *= factor;
        self.y *= factor;
        self.update_index();
        return self;
    }

    pub fn add_border(mut self, x_margin: usize, y_margin: usize) -> Self {
        self.width += x_margin * 2;
        self.x += x_margin;
        self.y += y_margin;
        self.update_index();
        return self;
    }

    pub fn get_screen_width(&self) -> usize {
        return self.width;
    }
}

pub struct WindowManager {
    active: Arc<AtomicBool>,
    gpu: Arc<GPU>,
    input_manager: Arc<InputManager>,
    window: Option<Rc<Window>>,
    base_size: Size,
    size_factor: NonZeroU32,
    size: Size,
    input: WinitInputHelper,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl WindowManager {
    pub fn new(active: Arc<AtomicBool>, gpu: Arc<GPU>, input_manager: Arc<InputManager>) -> Self {
        let (base_width, base_height) = gpu.get_screen_resolution();

        let base_size = Size::new(base_width, base_height);

        let size = Size::new(
            base_width.saturating_mul(BASE_RESOLUTION_SCALAR),
            base_height.saturating_mul(BASE_RESOLUTION_SCALAR),
        );

        return Self {
            active,
            gpu,
            input_manager,
            window: None,
            base_size,
            size,
            size_factor: BASE_RESOLUTION_SCALAR,
            input: WinitInputHelper::new(),
            context: None,
            surface: None,
        };
    }

    fn render(&mut self) {
        let Some(surface) = self.surface.as_mut() else {
            return;
        };

        let border_color = self.gpu.get_border_color();

        let width_usize = self.size.get_width_usize();
        let height_usize = self.size.get_height_usize();
        let base_width_usize = self.base_size.get_width_usize();
        let base_height_usize = self.base_size.get_height_usize();
        let size_factor_usize = self.size_factor.get() as usize;

        let x_margin = (width_usize - base_width_usize * size_factor_usize) / 2;
        let y_margin = (height_usize - base_height_usize * size_factor_usize) / 2;

        let mut render_buffer = surface.buffer_mut().unwrap();
        let gpu_buffer = self.gpu.get_framebuffer();

        if x_margin > 0 {
            render_square(
                Position::from_pos(0, 0, width_usize),
                Size::from_usize(x_margin, height_usize),
                border_color,
                &mut render_buffer,
            );

            render_square(
                Position::from_pos(width_usize - x_margin, 0, width_usize),
                Size::from_usize(x_margin, height_usize),
                border_color,
                &mut render_buffer,
            );
        }

        if y_margin > 0 {
            render_square(
                Position::from_pos(x_margin, 0, width_usize),
                Size::from_usize(width_usize - (x_margin * 2), y_margin),
                border_color,
                &mut render_buffer,
            );

            render_square(
                Position::from_pos(x_margin, height_usize - y_margin, width_usize),
                Size::from_usize(width_usize - (x_margin * 2), y_margin),
                border_color,
                &mut render_buffer,
            );
        }

        for pixel in 0..gpu_buffer.len() {
            let pos = Position::from_index(pixel, base_width_usize)
                .scale(size_factor_usize)
                .add_border(x_margin, y_margin);

            let size = Size::new(self.size_factor, self.size_factor);

            let color = match gpu_buffer[pixel] {
                true => self.gpu.get_active_color(),
                false => self.gpu.get_inactive_color(),
            };

            render_square(pos, size, color, &mut render_buffer);
        }

        render_buffer.present().unwrap();
    }

    fn update_size(&mut self, new_size: PhysicalSize<u32>) {
        let Some(surface) = self.surface.as_mut() else {
            return;
        };

        let new_size = Size::from_usize(new_size.width as usize, new_size.height as usize);

        if surface.resize(new_size.width, new_size.height).is_err() {
            return;
        }

        self.size_factor = NonZeroU32::new(cmp::min(
            new_size.get_width_usize() / self.base_size.get_width_usize(),
            new_size.get_height_usize() / self.base_size.get_height_usize(),
        ) as u32)
        .unwrap();

        self.size = new_size;
    }
}

fn render_square(
    pos: Position,
    size: Size,
    color: u32,
    buffer: &mut Buffer<'_, Rc<Window>, Rc<Window>>,
) {
    let width = size.width.get() as usize;
    let height = size.height.get() as usize;

    let pixel_row = vec![color; width];

    for row in 0..height {
        let start_index = pos.index + row * pos.get_screen_width();
        buffer[start_index..start_index + width].copy_from_slice(&pixel_row);
    }
}

impl ApplicationHandler for WindowManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let size = PhysicalSize::new(self.size.width.get(), self.size.height.get());

        let increment_size =
            PhysicalSize::new(self.base_size.width.get(), self.base_size.height.get());

        let attributes = Window::default_attributes()
            .with_inner_size(size)
            .with_title(WINDOW_TITLE)
            .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
            .with_resize_increments(increment_size);

        let window = Rc::new(event_loop.create_window(attributes).unwrap());
        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();

        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if self.input.process_window_event(&event) {
            self.render();
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

        if let Some(new_size) = self.input.window_resized() {
            self.update_size(new_size);
        }

        self.input_manager.update_input(&self.input);

        if let Some(window) = self.window.as_ref() {
            if self.gpu.is_render_queued() {
                self.gpu.dequeue_render();
                window.request_redraw();
            }
        }
    }
}
