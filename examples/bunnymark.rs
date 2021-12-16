use math::*;
use miniquad::gl::*;
use miniquad::{conf, date, start, EventHandler};
use ogt::*;
use quad_rand as rnd;
use std::rc::Rc;

struct Stage {
    sprite_batch: SpriteBatch,
    shape_batch: ShapeBatch,
    font: Font,
    bunnies: Vec<Bunny>,
    last_time: f64,
    tex_bunny: Texture,
}

impl Application for Stage {
    fn render(&mut self, ctx: &Context) {
        let now = date::now();
        let delta = (now - self.last_time) as f32;
        self.last_time = now;

        if ctx.is_mouse_button_pressed(MouseButton::Left) {
            for i in 0..10_000 {
                self.bunnies.push(Bunny::new(
                    10.0,
                    Color::rgb(255, 255, 0),
                    rnd::gen_range(-250.0, 250.0),
                    rnd::gen_range(250.0, 750.0),
                ));
            }
        }

        let (width, height) = (ctx.screen_size().x as f32, ctx.screen_size().y as f32);
        for bunny in self.bunnies.iter_mut() {
            bunny.x += delta * bunny.speed_x;
            bunny.y += delta * bunny.speed_y;
            bunny.rot += delta;

            if bunny.x < 0.0 {
                bunny.x = 0.0;
                bunny.speed_x = -bunny.speed_x;
            }
            if bunny.speed_x > 0.0 && bunny.x > width - self.tex_bunny.width as f32 {
                bunny.x = width - self.tex_bunny.width as f32;
                bunny.speed_x = -bunny.speed_x;
            }
            if bunny.y < 30.0 + self.tex_bunny.height as f32 {
                bunny.y = 30.0 + self.tex_bunny.height as f32;
                bunny.speed_y = -bunny.speed_y;
            }
            if bunny.speed_y > 0.0 && bunny.y > height {
                bunny.y = height;
                bunny.speed_y = -bunny.speed_y;
            }
        }

        unsafe {
            glViewport(0, 0, width as i32, height as i32);
            glClearColor(0.0, 0.0, 0.4, 0.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }
        let mvp = Mat4::orthographic_rh_gl(0.0, width, 0.0, height, -1.0, 1.0);
        self.sprite_batch.set_model_view_projection_matrix(mvp);
        self.shape_batch.set_model_view_projection_matrix(mvp);

        self.sprite_batch.set_texture(self.tex_bunny.clone());
        for bunny in self.bunnies.iter() {
            self.sprite_batch.add(
                ctx,
                Region {
                    top_left: [0.0, 0.0],
                    bottom_right: [self.tex_bunny.width as f32, self.tex_bunny.height as f32],
                },
                bunny.tint,
                vec2(13.0, 19.0),
                vec2(bunny.x, bunny.y), // Affine2::from_angle_translation(bunny.rot, vec2(bunny.x, bunny.y)),
            );
        }
        self.sprite_batch.draw(ctx);
        self.font.draw_text(
            ctx,
            &mut self.sprite_batch,
            &format!("FPS: {:.2}, #b: {}", 1.0 / delta, self.bunnies.len()),
            vec2(20.0, 20.0),
        );
        self.sprite_batch.draw(ctx);
        self.shape_batch.add_triangle(
            ctx,
            vec2(10.0, 10.0),
            vec2(20.0, 10.0),
            vec2(15.0, 15.0),
            WHITE,
        );
        self.shape_batch
            .add_line(ctx, vec2(10.0, 100.0), vec2(150.0, 300.0), 20.0, RED);
        self.shape_batch
            .add_filled_rect(ctx, vec2(150.0, 110.0), vec2(300.0, 290.0), YELLOW);
        self.shape_batch
            .add_rect(ctx, vec2(15.0, 110.0), vec2(140.0, 290.0), 5.0, YELLOW);
        self.shape_batch.add_circle(
            ctx,
            vec2(50.0, 150.0),
            60.0,
            5.0,
            4,
            0.0,
            std::f32::consts::PI * 1.0,
            WHITE,
        );
        self.shape_batch.add_circle_filled(
            ctx,
            vec2(450.0, 150.0),
            50.0,
            30,
            0.3,
            std::f32::consts::PI * 1.8,
            WHITE,
        );
        self.shape_batch.draw(ctx);
    }
}
struct Bunny {
    x: f32,
    y: f32,
    speed_x: f32,
    speed_y: f32,
    tint: Color,
    rot: f32,
}

impl Bunny {
    fn new(x: f32, tint: Color, speed_x: f32, speed_y: f32) -> Self {
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

fn main() {
    go(|ctx| {
        unsafe {
            glEnable(GL_BLEND);
            glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        }
        let image = image::load_from_memory(include_bytes!("ogt_thing.png"))
            .unwrap()
            .to_rgba8();
        let (width, height) = (image.width(), image.height());
        let texture = TextureBuilder::from_bytes(&image.into_raw(), width, height).build(ctx);

        let mut sprite_batch = SpriteBatch::new(ctx);

        let mut bunnies = vec![];
        rnd::srand(1214442);
        for i in 0..1 {
            bunnies.push(Bunny::new(
                10.0,
                Color::rgb(255, 255, 0),
                rnd::gen_range(-250.0, 250.0),
                rnd::gen_range(250.0, 750.0),
            ));
        }

        Stage {
            sprite_batch,
            shape_batch: ShapeBatch::new(ctx),
            bunnies,
            last_time: date::now(),
            font: Font::from_font(ctx, include_bytes!("Hack-Regular.ttf"), 32.0),
            tex_bunny: texture,
        }
    });
}
