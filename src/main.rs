use font::*;
use miniquad::gl::*;
use miniquad::{conf, date, start, EventHandler, UserData};
use quad_rand as rnd;
use sprite_batch::*;
use std::rc::Rc;
use texture::*;

pub use glam::f32 as math;
use math::*;
use std::ffi::CString;
use std::mem::{size_of, size_of_val};

mod font;
mod rect_pack;
mod shader;
mod shape_batch;
mod sprite_batch;
mod texture;

struct Bunny {
    x: f32,
    y: f32,
    speed_x: f32,
    speed_y: f32,
    tint: sprite_batch::Color,
    rot: f32,
}

impl Bunny {
    fn new(x: f32, tint: sprite_batch::Color, speed_x: f32, speed_y: f32) -> Self {
        Self {
            tint,
            x,
            y: 200.0,
            speed_x,
            speed_y,
            rot: 0.0,
        }
    }
}

pub struct Context {}

struct Stage {
    sprite_batch: SpriteBatch,
    font: Font,
    bunnies: Vec<Bunny>,
    last_time: f64,
    frames: usize,
    t: f32,
    tex_bunny: Texture,
}

impl EventHandler for Stage {
    fn resize_event(&mut self, _ctx: &mut miniquad::Context, x: f32, y: f32) {}

    fn update(&mut self, ctx: &mut miniquad::Context) {
        let now = date::now();
        let delta = (now - self.last_time) as f32;
        self.last_time = now;

        self.frames += 1;
        self.t -= delta;
        if self.t < 0.0 {
            println!("{}", self.frames);
            self.t = 1.0;
            self.frames = 0;
        }
        let (width, height) = (ctx.screen_size().0, ctx.screen_size().1);
        for bunny in self.bunnies.iter_mut() {
            bunny.x += delta * bunny.speed_x;
            bunny.y += delta * bunny.speed_y;
            bunny.rot += delta;

            if bunny.x < 0.0 {
                bunny.x = 0.0;
                bunny.speed_x = -bunny.speed_x;
            }
            if bunny.x > width {
                bunny.x = width;
                bunny.speed_x = -bunny.speed_x;
            }
            if bunny.y < 0.0 {
                bunny.y = 0.0;
                bunny.speed_y = -bunny.speed_y;
            }
            if bunny.y > height {
                bunny.y = height;
                bunny.speed_y = -bunny.speed_y;
            }
        }

        unsafe {
            let mvp = Mat4::orthographic_rh_gl(0.0, width, 0.0, height, -1.0, 1.0);
            glViewport(0, 0, width as i32, height as i32);
            glClearColor(0.0, 0.0, 0.4, 0.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            self.sprite_batch.set_model_view_projection_matrix(mvp);

            self.sprite_batch.set_texture(self.tex_bunny.clone());
            for bunny in self.bunnies.iter() {
                self.sprite_batch.add(
                    &Context {},
                    Region {
                        top_left: [0.0, 0.0],
                        bottom_right: [26.0, 37.0],
                    },
                    bunny.tint,
                    vec2(13.0, 19.0),
                    vec2(bunny.x, bunny.y), // Affine2::from_angle_translation(bunny.rot, vec2(bunny.x, bunny.y)),
                );
            }
            self.sprite_batch.draw(&Context {});
            self.font.draw_text(
                &Context {},
                &mut self.sprite_batch,
                &format!("FPS: {:.2}, #b: {}", 1.0 / delta, self.bunnies.len()),
                vec2(20.0, 20.0),
            );
            self.sprite_batch.draw(&Context {});
        }
    }

    fn draw(&mut self, _ctx: &mut miniquad::Context) {
        // NOOP
    }
}

fn main() {
    start(conf::Conf::default(), |ctx| {
        unsafe {
            glEnable(GL_BLEND);
            glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        }
        let image = image::load_from_memory(include_bytes!("../wabbit_alpha.png"))
            .unwrap()
            .to_rgba8();
        let (width, height) = (image.width(), image.height());
        let texture =
            TextureBuilder::from_bytes(&image.into_raw(), width, height).build(&Context {});

        let mut sprite_batch = sprite_batch::SpriteBatch::new(&Context {});

        let mut bunnies = vec![];
        rnd::srand(1214442);
        for i in 0..300_000 {
            bunnies.push(Bunny::new(
                10.0,
                sprite_batch::Color::rgb(255, 255, 0),
                rnd::gen_range(-250.0, 250.0),
                rnd::gen_range(250.0, 750.0),
            ));
        }

        UserData::owning(
            Stage {
                sprite_batch,
                bunnies,
                last_time: date::now(),
                frames: 0,
                t: 1.0,
                font: Font::from_font(&Context {}, include_bytes!("../Hack-Regular.ttf"), 32.0),
                tex_bunny: texture,
            },
            ctx,
        )
    });
}
