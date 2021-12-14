use crate::texture::*;
use crate::{Color, Context};
use glam::f32::*;
use memoffset::*;
use miniquad::gl::*;
use std::borrow::Borrow;
use std::ffi::CString;
use std::mem::{size_of, size_of_val};
use std::rc::Rc;

pub trait Transform2D {
    fn transform(&self, origin: Vec2, region: Region) -> [Vec3; 4];
}

impl Transform2D for Vec2 {
    fn transform(&self, _origin: Vec2, region: Region) -> [Vec3; 4] {
        let right = self.x + region.bottom_right[0] - region.top_left[0];
        let bottom = self.y + region.bottom_right[1] - region.top_left[1];
        [
            vec3(self.x, self.y, 0.0),
            vec3(right, self.y, 0.0),
            vec3(right, bottom, 0.0),
            vec3(self.x, bottom, 0.0),
        ]
    }
}

impl Transform2D for Vec3 {
    fn transform(&self, _origin: Vec2, region: Region) -> [Vec3; 4] {
        let right = self.x + region.bottom_right[0] - region.top_left[0];
        let bottom = self.y + region.bottom_right[1] - region.top_left[1];
        [
            *self,
            vec3(right, self.y, self.z),
            vec3(right, bottom, self.z),
            vec3(self.x, bottom, self.z),
        ]
    }
}

impl Transform2D for Affine2 {
    fn transform(&self, origin: Vec2, region: Region) -> [Vec3; 4] {
        let right = region.bottom_right[0] - region.top_left[0] - origin.x;
        let bottom = region.bottom_right[1] - region.top_left[1] - origin.y;
        [
            vec2(-origin.x, -origin.y),
            vec2(right, -origin.y),
            vec2(right, bottom),
            vec2(-origin.x, bottom),
        ]
        .map(|v| self.transform_point2(v))
        .map(|p| vec3(p.x + origin.x, p.y + origin.y, 0.0))
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
    color: [u8; 4],
}

pub struct SpriteBatch {
    max_sprites: u16,
    sprite_count: u16,
    vertex_array_id: GLuint,
    array_buffer_data: Vec<Vertex>,
    shader_id: GLuint,
    texture: Option<Texture>,
    uniform_mvp: GLint,
    model_view_projection: Mat4,
}

impl SpriteBatch {
    pub fn new(gl: &Context) -> Self {
        Self::with_max_sprites(gl, 10_000)
    }

    pub fn draw(&mut self, context: &Context) {
        unsafe {
            glUseProgram(self.shader_id);
            glUniformMatrix4fv(
                self.uniform_mvp,
                1,
                GL_FALSE as u8,
                self.model_view_projection.to_cols_array().as_ptr(),
            );
            glBindVertexArray(self.vertex_array_id);
            // glBindBuffer(GL_ARRAY_BUFFER, self.array_buffer_id);
            // According to https://thothonegan.tumblr.com/post/135193767243/glbuffersubdata-vs-glbufferdata
            // this is better as the GPU can work on the buffer without blocking
            glBufferData(
                GL_ARRAY_BUFFER,
                (size_of_val(&self.array_buffer_data[..])) as GLsizeiptr,
                self.array_buffer_data.as_ptr() as *const GLvoid,
                GL_STREAM_DRAW,
            );
            // debug_assert_eq!(
            //     self.sprite_count * 4 * size_of::<Vertex>(),
            //     size_of_val(&self.array_buffer_data[..])
            // );
            // glBufferSubData(
            //     GL_ARRAY_BUFFER,
            //     0,
            //     (self.sprite_count * 4 * size_of::<Vertex>()) as GLsizeiptr,
            //     self.array_buffer_data.as_ptr() as *const GLvoid,
            // );
            self.texture
                .as_ref()
                .expect("Texture must be set on SpriteBatch")
                .borrow()
                .bind(context);
            glDrawElements(
                GL_TRIANGLES,
                self.sprite_count as i32 * 6,
                GL_UNSIGNED_SHORT,
                0 as *const GLvoid,
            );
        }
        self.sprite_count = 0;
        self.array_buffer_data.clear();
    }

    pub fn add<X: Transform2D>(
        &mut self,
        gl: &Context,
        sprite: Region,
        color: Color,
        origin: Vec2,
        transform: X,
    ) {
        if self.sprite_count == self.max_sprites {
            self.draw(gl);
        }
        let texture_data = &self
            .texture
            .as_ref()
            .expect("Texture must be set on SpriteBatch")
            .borrow();
        let (width, height) = (texture_data.width as f32, texture_data.height as f32);
        let vertices = transform.transform(origin, sprite);
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
                pos: [pos.x, pos.y, pos.z],
                uv,
                color: color.0,
            });
        }
        self.sprite_count += 1;
    }

    pub fn with_max_sprites(context: &Context, max: u16) -> Self {
        let shader_id =
            crate::shader::load_shaders(&context, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();
        let matrix_id = unsafe {
            let mvp = CString::new("viewProjectionMatrix").unwrap();
            glGetUniformLocation(shader_id, mvp.as_ptr())
        };
        let array_buffer_data = Vec::with_capacity(max as usize * 4);
        let mut element_buffer_data = Vec::with_capacity(max as usize * 6);
        for i in (0..max * 4).step_by(4) {
            element_buffer_data.push(i as u16);
            element_buffer_data.push(i as u16 + 1);
            element_buffer_data.push(i as u16 + 3);
            element_buffer_data.push(i as u16 + 1);
            element_buffer_data.push(i as u16 + 2);
            element_buffer_data.push(i as u16 + 3);
        }
        let mut vertex_array_id: GLuint = 0;
        let mut array_buffer_id: GLuint = 0;
        let mut element_buffer_id: GLuint = 0;
        unsafe {
            glGenVertexArrays(1, &mut vertex_array_id as *mut GLuint);
            glBindVertexArray(vertex_array_id);
            glEnableVertexAttribArray(0);
            glEnableVertexAttribArray(1);
            glEnableVertexAttribArray(2);

            glGenBuffers(1, &mut array_buffer_id as *mut GLuint);
            glBindBuffer(GL_ARRAY_BUFFER, array_buffer_id);
            // glBufferData(
            //     GL_ARRAY_BUFFER,
            //     (size_of::<Vertex>() * 4 * max) as GLsizeiptr,
            //     0 as *const GLvoid,
            //     GL_DYNAMIC_DRAW,
            // );
            let stride = size_of::<Vertex>() as i32;
            glVertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                GL_FALSE as u8,
                stride,
                offset_of!(Vertex, pos) as *const GLvoid,
            );
            glVertexAttribPointer(
                1,
                2,
                GL_FLOAT,
                GL_FALSE as u8,
                stride,
                offset_of!(Vertex, uv) as *const GLvoid,
            );
            glVertexAttribPointer(
                2,
                4,
                GL_UNSIGNED_BYTE,
                GL_TRUE as u8,
                stride,
                offset_of!(Vertex, color) as *const GLvoid,
            );

            glGenBuffers(1, &mut element_buffer_id as *mut GLuint);
            glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buffer_id);
            glBufferData(
                GL_ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(&element_buffer_data[..]) as GLsizeiptr,
                element_buffer_data.as_ptr() as *const GLvoid,
                GL_STATIC_DRAW,
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
        assert_eq!(self.sprite_count, 0, "Texture must be set before drawing");
        self.texture = Some(texture);
    }

    pub fn set_model_view_projection_matrix(&mut self, matrix: Mat4) {
        self.model_view_projection = matrix;
    }
}

#[cfg(feature = "webgl1")]
const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 vertex_pos;
attribute vec2 tex_uv;
attribute vec4 vertex_color;

varying lowp vec4 fragmentColor;
varying lowp vec2 texCoord;

uniform mat4 viewProjectionMatrix;

void main() {
    gl_Position = viewProjectionMatrix * vec4(vertex_pos, 1.0);
    fragmentColor = vertex_color;
    texCoord = tex_uv;
}
"#;

#[cfg(feature = "webgl1")]
const FRAGMENT_SHADER: &str = r#"#version 100
varying lowp vec4 fragmentColor;
varying lowp vec2 texCoord;

uniform sampler2D Tex;

void main() {
    gl_FragColor = fragmentColor * texture2D(Tex, texCoord);
}
"#;
