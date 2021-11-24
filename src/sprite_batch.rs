use crate::gl::{self, types::*};
use glam::f32::*;
use memoffset::*;
use std::ffi::CString;
use std::mem::{size_of, size_of_val};
use std::rc::Rc;

pub trait Transform2D {
    fn transform(&self, origin: (f32, f32), region: Region) -> [(f32, f32); 4];
}

impl Transform2D for Vec2 {
    fn transform(&self, origin: (f32, f32), region: Region) -> [(f32, f32); 4] {
        let right = self.x + region.bottom_right[0] - region.top_left[0];
        let bottom = self.y + region.bottom_right[1] - region.top_left[1];
        [
            (self.x, self.y),
            (right, self.y),
            (right, bottom),
            (self.x, bottom),
        ]
    }
}

impl Transform2D for Affine2 {
    fn transform(&self, origin: (f32, f32), region: Region) -> [(f32, f32); 4] {
        let right = region.bottom_right[0] - region.top_left[0] - origin.0;
        let bottom = region.bottom_right[1] - region.top_left[1] - origin.1;
        [
            vec2(-origin.0, -origin.1),
            vec2(right, -origin.1),
            vec2(right, bottom),
            vec2(-origin.0, bottom),
        ]
        .map(|v| self.transform_point2(v))
        .map(|p| (p.x, p.y))
    }
}

#[derive(Clone, Copy)]
pub struct Color([u8; 4]);

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color([r, g, b, 255])
    }
}

#[derive(Clone, Copy)]
pub struct TextureData {
    pub id: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct Texture(pub Rc<TextureData>);

#[derive(Clone, Copy)]
pub struct Region {
    pub top_left: [f32; 2],
    pub bottom_right: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
    color: [u8; 4],
}

pub struct SpriteBatch {
    max_sprites: usize,
    sprite_count: usize,
    vertex_array_id: GLuint,
    array_buffer_data: Vec<Vertex>,
    shader_id: GLuint,
    texture: Option<Texture>,
    uniform_mvp: GLint,
    model_view_projection: Mat4,
}

impl SpriteBatch {
    pub fn new(gl: &gl::Gl) -> Self {
        Self::with_max_sprites(gl, 10_000)
    }

    pub fn draw(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.UseProgram(self.shader_id);
            gl.UniformMatrix4fv(
                self.uniform_mvp,
                1,
                gl::FALSE,
                self.model_view_projection.to_cols_array().as_ptr(),
            );
            gl.BindVertexArray(self.vertex_array_id);
            // gl.BindBuffer(gl::ARRAY_BUFFER, self.array_buffer_id);
            // gl.BufferData(
            //     gl::ARRAY_BUFFER,
            //     (size_of_val(&self.array_buffer_data[..])) as isize,
            //     self.array_buffer_data.as_ptr() as *const GLvoid,
            //     gl::STREAM_DRAW,
            // );
            debug_assert_eq!(
                self.sprite_count * 4 * size_of::<Vertex>(),
                size_of_val(&self.array_buffer_data[..])
            );
            gl.BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.sprite_count * 4 * size_of::<Vertex>()) as isize,
                self.array_buffer_data.as_ptr() as *const GLvoid,
            );
            gl.BindTexture(
                gl::TEXTURE_2D,
                (*self
                    .texture
                    .as_ref()
                    .expect("Texture must be set on SpriteBatch")
                    .0)
                    .id,
            );
            gl.DrawElements(
                gl::TRIANGLES,
                self.sprite_count as i32 * 6,
                gl::UNSIGNED_INT,
                0 as *const GLvoid,
            );
        }
        self.sprite_count = 0;
        self.array_buffer_data.clear();
    }

    pub fn add<T: Transform2D>(
        &mut self,
        gl: &gl::Gl,
        sprite: Region,
        color: Color,
        origin: Vec2,
        transform: T,
    ) {
        if self.sprite_count == self.max_sprites {
            self.draw(gl);
        }
        let texture_data = &*self
            .texture
            .as_ref()
            .expect("Texture must be set on SpriteBatch")
            .0;
        let (width, height) = (texture_data.width as f32, texture_data.height as f32);
        let vertices = transform.transform((origin.x, origin.y), sprite);
        for (pos, uv) in vertices.iter().zip([
            [sprite.top_left[0] / width, sprite.bottom_right[1] / height],
            [
                sprite.bottom_right[0] / width,
                sprite.bottom_right[1] / height,
            ],
            [sprite.bottom_right[0] / width, sprite.top_left[1] / height],
            [sprite.top_left[0] / width, sprite.top_left[1] / height],
        ]) {
            self.array_buffer_data.push(Vertex {
                pos: [pos.0, pos.1, 0.0],
                uv,
                color: color.0,
            });
        }
        self.sprite_count += 1;
    }

    fn with_max_sprites(gl: &gl::Gl, max: usize) -> Self {
        let shader_id = crate::shader::load_shaders(&gl, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();
        let matrix_id = unsafe {
            let mvp = CString::new("MVP").unwrap();
            gl.GetUniformLocation(shader_id, mvp.as_ptr())
        };
        let array_buffer_data = Vec::with_capacity(max * 4);
        let mut element_buffer_data = Vec::with_capacity(max * 6);
        for i in (0..max * 4).step_by(4) {
            element_buffer_data.push(i as u32);
            element_buffer_data.push(i as u32 + 1);
            element_buffer_data.push(i as u32 + 3);
            element_buffer_data.push(i as u32 + 1);
            element_buffer_data.push(i as u32 + 2);
            element_buffer_data.push(i as u32 + 3);
        }
        let mut vertex_array_id: GLuint = 0;
        let mut array_buffer_id: GLuint = 0;
        let mut element_buffer_id: GLuint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut vertex_array_id as *mut GLuint);
            gl.BindVertexArray(vertex_array_id);
            gl.EnableVertexAttribArray(0);
            gl.EnableVertexAttribArray(1);
            gl.EnableVertexAttribArray(2);

            gl.GenBuffers(1, &mut array_buffer_id as *mut GLuint);
            gl.BindBuffer(gl::ARRAY_BUFFER, array_buffer_id);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<Vertex>() * 4 * max) as isize,
                0 as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );
            let stride = size_of::<Vertex>() as i32;
            gl.VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, pos) as *const GLvoid,
            );
            gl.VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, uv) as *const GLvoid,
            );
            gl.VertexAttribPointer(
                2,
                4,
                gl::UNSIGNED_BYTE,
                gl::TRUE,
                stride,
                offset_of!(Vertex, color) as *const GLvoid,
            );

            gl.GenBuffers(1, &mut element_buffer_id as *mut GLuint);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_id);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(&element_buffer_data[..]) as isize,
                element_buffer_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
        }
        Self {
            max_sprites: max,
            sprite_count: 0,
            vertex_array_id,
            array_buffer_data,
            shader_id,
            texture: None,
            uniform_mvp: matrix_id,
            model_view_projection: Mat4::IDENTITY,
        }
    }

    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = Some(texture);
    }

    pub fn set_model_view_projection_matrix(&mut self, matrix: Mat4) {
        self.model_view_projection = matrix;
    }
}

const VERTEX_SHADER: &str = r#"#version 330 core
layout(location = 0) in vec3 vertex_pos;
layout(location = 1) in vec2 tex_uv;
layout(location = 2) in vec4 vertex_color;
uniform mat4 MVP;
out vec4 fragmentColor;
out vec2 TexCoord;

void main() {
    gl_Position = MVP * vec4(vertex_pos, 1.0);
    fragmentColor = vertex_color;
    TexCoord = tex_uv;
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 330 core
in vec4 fragmentColor;
in vec2 TexCoord;

out vec4 color;

uniform sampler2D Tex;

void main() {
    color = fragmentColor * texture(Tex, TexCoord);
}"#;
