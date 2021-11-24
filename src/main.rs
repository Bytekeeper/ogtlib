use gl::types::*;
use glam::f32::*;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::ffi::CString;
use std::mem::{size_of, size_of_val};

mod shader;
mod sprite_batch;
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

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

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("test");

    let windowed_context = ContextBuilder::new()
        // .with_vsync(true)
        .with_multisampling(4)
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let gl = gl::Gl::load_with(|s| windowed_context.context().get_proc_address(s));

    let image = image::open("wabbit_alpha.png").unwrap().to_rgba8();
    let texture = unsafe {
        let mut texture_id = 0;
        gl.GenTextures(1, &mut texture_id);

        gl.BindTexture(gl::TEXTURE_2D, texture_id);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        let (width, height) = (image.width(), image.height());
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            image.into_raw().as_ptr() as *const GLvoid,
        );
        gl.GenerateMipmap(gl::TEXTURE_2D);
        sprite_batch::Texture(std::rc::Rc::new(sprite_batch::TextureData {
            id: texture_id,
            width,
            height,
        }))
    };

    unsafe {
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    let mut sprite_batch = sprite_batch::SpriteBatch::new(&gl);
    sprite_batch.set_texture(texture);

    let mut bunnies = vec![];
    let mut rng = XorShiftRng::seed_from_u64(12324234);
    for i in 0..1 {
        bunnies.push(Bunny::new(
            10.0,
            sprite_batch::Color::rgb(255, 255, 0),
            rng.gen_range(-250.0..250.0),
            rng.gen_range(250.0..750.0),
        ));
    }

    let mut last_time = std::time::Instant::now();
    let mut x = 0.0;
    let mut t = 1.0;
    let mut frames = 0;

    el.run(move |event, _, control_flow| match event {
        Event::LoopDestroyed => return,
        Event::MainEventsCleared => {
            let now = std::time::Instant::now();
            let delta = now.duration_since(last_time).as_secs_f32();
            last_time = now;
            frames += 1;
            t -= delta;
            if t < 0.0 {
                println!("{}", frames);
                t = 1.0;
                frames = 0;
            }

            for bunny in bunnies.iter_mut() {
                // bunny.x += delta * bunny.speed_x;
                // bunny.y += delta * bunny.speed_y;
                bunny.rot += delta;

                if bunny.x < 0.0 {
                    bunny.x = 0.0;
                    bunny.speed_x = -bunny.speed_x;
                }
                if bunny.x > 1000.0 {
                    bunny.x = 1000.0;
                    bunny.speed_x = -bunny.speed_x;
                }
                if bunny.y < 0.0 {
                    bunny.y = 0.0;
                    bunny.speed_y = -bunny.speed_y;
                }
                if bunny.y > 1000.0 {
                    bunny.y = 1000.0;
                    bunny.speed_y = -bunny.speed_y;
                }
            }

            x += delta;
            let (width, height) = (
                windowed_context.window().inner_size().width as i32,
                windowed_context.window().inner_size().height as i32,
            );
            let mvp = Mat4::orthographic_rh_gl(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
            unsafe {
                gl.Viewport(0, 0, width, height);
                // let projection =
                //     Mat4::perspective_rh_gl(45.0_f32.to_radians(), 800.0 / 600.0, 0.1, 1000.0);
                // let view = Mat4::look_at_rh(
                //     vec3(0.0, 3.0, 800.0),
                //     vec3(0.0, 0.0, 0.0),
                //     vec3(0.0, 1.0, 0.0),
                // );
                // let model = Mat4::IDENTITY;

                // let mvp = projection * view * model;

                gl.ClearColor(0.0, 0.0, 0.4, 0.0);
                gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                sprite_batch.set_model_view_projection_matrix(mvp);

                for bunny in bunnies.iter() {
                    sprite_batch.add(
                        &gl,
                        sprite_batch::Region {
                            top_left: [0.0, 0.0],
                            bottom_right: [26.0, 37.0],
                        },
                        bunny.tint,
                        vec2(13.0, 19.0),
                        Affine2::from_angle_translation(bunny.rot, vec2(bunny.x, bunny.y)),
                    );
                }
                sprite_batch.draw(&gl);
            }
            windowed_context.swap_buffers().unwrap();
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => (),
        },
        _ => (),
    });
}
