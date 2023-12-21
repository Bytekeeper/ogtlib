pub use font::*;
use miniquad::gl::*;
pub use miniquad::MouseButton;
use miniquad::{conf, date, start, EventHandler, UserData};
use quad_rand as rnd;
pub use shape_batch::*;
pub use sprite_batch::*;
pub use texture::*;
pub use ui::*;

pub use glam as math;
use math::*;
use std::ffi::CString;
use std::mem::{size_of, size_of_val};

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
    left: MouseButtonState,
    right: MouseButtonState,
    middle: MouseButtonState,
}

impl Context {
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

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_button_state(button).pressed
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_button_state(button).down
    }
}

pub trait Application {
    fn render(&mut self, context: &Context, delta: f32) {}
}

struct Stage {
    app: Box<dyn Application>,
    context: Context,
    last_time: f64,
}

impl EventHandler for Stage {
    fn mouse_motion_event(&mut self, _ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.context.mouse_position = uvec2(x as u32, y as u32);
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        btn: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.context.mouse_position = uvec2(x as u32, y as u32);
        let button_state = self.context.mouse_button_state_mut(btn);
        button_state.down = true;
        button_state.pressed = true;
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        btn: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.context.mouse_position = uvec2(x as u32, y as u32);
        let button_state = self.context.mouse_button_state_mut(btn);
        button_state.down = false;
    }

    fn draw(&mut self, _ctx: &mut miniquad::Context) {
        // NOOP
    }

    fn update(&mut self, ctx: &mut miniquad::Context) {
        let (w, h) = ctx.screen_size();
        self.context.screen_size = uvec2(w as u32, h as u32);
        let now = date::now();
        let delta = (now - self.last_time) as f32;
        self.last_time = now;

        self.app.render(&self.context, delta);

        self.context.left.pressed = false;
        self.context.right.pressed = false;
        self.context.middle.pressed = false;
    }
}

pub fn go<A: 'static + Application, F: 'static + FnOnce(&Context) -> A>(app_creator: F) {
    start(conf::Conf::default(), |ctx| {
        let context = Context::default();
        UserData::owning(
            Stage {
                app: Box::new(app_creator(&context)),
                context,
                last_time: date::now(),
            },
            ctx,
        )
    });
}
