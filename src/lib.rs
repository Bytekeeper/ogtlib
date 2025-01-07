pub use miniquad::error;
pub use miniquad::fs::load_file;
pub use miniquad::MouseButton;

pub use assets::*;
pub use font::*;
use miniquad::window::screen_size;
use miniquad::{conf, date, start, EventHandler};
pub use shape_batch::*;
pub use sprite_batch::*;
pub use texture::*;
pub use ui::*;

pub use glam as math;
use math::*;

mod assets;
mod backend;
mod font;
mod rect_pack;
mod shader;
mod shape_batch;
mod sprite_batch;
mod texture;
mod ui;

#[derive(Clone, Copy)]
pub struct Color([u8; 4]);

pub const BLACK: Color = Color::rgb(0, 0, 0);
pub const BLUE: Color = Color::rgb(0, 0, 255);
pub const GREEN: Color = Color::rgb(0, 255, 0);
pub const LIGHT_BLUE: Color = Color::rgb(80, 80, 255);
pub const LIGHT_GRAY: Color = Color::rgb(200, 200, 200);
pub const LIGHT_RED: Color = Color::rgb(255, 80, 80);
pub const RED: Color = Color::rgb(255, 0, 0);
pub const YELLOW: Color = Color::rgb(255, 255, 0);
pub const WHITE: Color = Color::rgb(255, 255, 255);

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color([r, g, b, 255])
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color([r, g, b, a])
    }
}

#[cfg(target_family = "wasm")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Default)]
struct MouseButtonState {
    down: bool,
    pressed: bool,
}

#[derive(Default)]
pub struct Context {
    screen_size: UVec2,
    mouse_position: UVec2,
    mouse_wheel: Vec2,
    left: MouseButtonState,
    right: MouseButtonState,
    middle: MouseButtonState,
}

impl Context {
    pub(crate) fn configure_blend(&self) {
        crate::backend::configure_blend();
    }

    pub fn clear_screen(&self, color: Color) {
        crate::backend::clear_screen(color);
    }

    pub fn set_viewport(&self, top_left: IVec2, bottom_right: IVec2) {
        crate::backend::set_viewport(top_left, bottom_right);
    }

    pub fn quit(&self) {
        crate::backend::quit();
    }

    pub fn screen_size(&self) -> UVec2 {
        self.screen_size
    }

    pub fn mouse_position(&self) -> UVec2 {
        self.mouse_position
    }

    fn mouse_button_state(&self, button: MouseButton) -> &MouseButtonState {
        match button {
            MouseButton::Left => &self.left,
            MouseButton::Right => &self.right,
            MouseButton::Middle => &self.middle,
            _ => panic!(),
        }
    }

    fn mouse_button_state_mut(&mut self, button: MouseButton) -> &mut MouseButtonState {
        match button {
            MouseButton::Left => &mut self.left,
            MouseButton::Right => &mut self.right,
            MouseButton::Middle => &mut self.middle,
            _ => panic!(),
        }
    }

    pub fn mouse_wheel(&self) -> Vec2 {
        self.mouse_wheel
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_button_state(button).pressed
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_button_state(button).down
    }
}

pub trait Application {
    fn render(&mut self, _context: &Context, _delta: f32) {}
}

struct Stage<A> {
    app: A,
    context: Context,
    last_time: f64,
}

impl<A: Application> EventHandler for Stage<A> {
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.context.mouse_position = uvec2(x as u32, y as u32);
    }

    fn mouse_button_down_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        self.context.mouse_position = uvec2(x as u32, y as u32);
        let button_state = self.context.mouse_button_state_mut(btn);
        button_state.down = true;
        button_state.pressed = true;
    }

    fn mouse_button_up_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        self.context.mouse_position = uvec2(x as u32, y as u32);
        let button_state = self.context.mouse_button_state_mut(btn);
        button_state.down = false;
    }

    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.context.mouse_wheel = vec2(x, y);
    }

    fn draw(&mut self) {
        // NOOP
    }

    fn update(&mut self) {
        let (w, h) = screen_size();
        self.context.screen_size = uvec2(w as u32, h as u32);
        let now = date::now();
        let delta = (now - self.last_time) as f32;
        self.last_time = now;

        self.app.render(&self.context, delta);

        self.context.mouse_wheel = Vec2::ZERO;
        self.context.left.pressed = false;
        self.context.right.pressed = false;
        self.context.middle.pressed = false;
    }
}

pub fn go<A: 'static + Application, F: 'static + FnOnce(&Context) -> A>(app_creator: F) {
    start(
        conf::Conf {
            sample_count: 4,
            ..conf::Conf::default()
        },
        || {
            let context = Context::default();
            context.configure_blend();
            Box::new(Stage {
                app: app_creator(&context),
                context,
                last_time: date::now(),
            })
        },
    );
}
