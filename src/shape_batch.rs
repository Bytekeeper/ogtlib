use crate::Context;
use crate::{math::*, Color};
use memoffset::offset_of;
use miniquad::gl::*;
use std::ffi::CString;
use std::mem::{size_of, size_of_val};

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
    color: [u8; 4],
}

impl Vertex {
    fn from_vec_color(v: Vec2, col: Color) -> Self {
        Self {
            pos: v.to_array(),
            color: col.0,
        }
    }
}

pub struct ShapeBatch {
    max_triangles: u16,
    triangle_count: u16,
    vertex_array_id: GLuint,
    array_buffer_id: GLuint,
    array_buffer_data: Vec<Vertex>,
    shader_id: GLuint,
    uniform_mvp: GLint,
    model_view_projection: Mat4,
}

impl ShapeBatch {
    pub fn new(context: &Context) -> ShapeBatch {
        Self::with_max_triangles(context, 10_000)
    }

    fn triangles<const N: usize>(&mut self, context: &Context, vertices: [(Vec2, Color); N]) {
        assert!(N % 3 == 0);
        if self.triangle_count + N as u16 / 3 >= self.max_triangles {
            self.draw(context);
        }
        self.triangle_count += N as u16 / 3;
        let vertices = vertices.map(|(v, c)| Vertex::from_vec_color(v, c));
        self.array_buffer_data.extend(vertices);
    }

    pub fn add_triangle(&mut self, context: &Context, v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
        self.triangles(context, [(v1, color), (v2, color), (v3, color)]);
    }

    pub fn add_line(
        &mut self,
        context: &Context,
        v1: Vec2,
        v2: Vec2,
        thickness: f32,
        color: Color,
    ) {
        let dv = v2 - v1;
        let perp = dv.perp().normalize() * thickness / 2.0;
        self.triangles(
            context,
            [
                (v1 + perp, color),
                (v2 + perp, color),
                (v2 - perp, color),
                (v2 - perp, color),
                (v1 - perp, color),
                (v1 + perp, color),
            ],
        );
    }

    pub fn add_rect(
        &mut self,
        context: &Context,
        top_left: Vec2,
        bottom_right: Vec2,
        thickness: f32,
        color: Color,
    ) {
        self.add_line(
            context,
            top_left - vec2(thickness / 2.0, 0.0),
            vec2(bottom_right.x + thickness / 2.0, top_left.y),
            thickness,
            color,
        );
        self.add_line(
            context,
            vec2(bottom_right.x, top_left.y),
            bottom_right,
            thickness,
            color,
        );
        self.add_line(
            context,
            bottom_right + vec2(thickness / 2.0, 0.0),
            vec2(top_left.x - thickness / 2.0, bottom_right.y),
            thickness,
            color,
        );
        self.add_line(
            context,
            vec2(top_left.x, bottom_right.y),
            top_left,
            thickness,
            color,
        );
    }

    pub fn add_filled_rect(
        &mut self,
        context: &Context,
        top_left: Vec2,
        bottom_right: Vec2,
        color: Color,
    ) {
        self.triangles(
            context,
            [
                (top_left, color),
                (vec2(bottom_right.x, top_left.y), color),
                (bottom_right, color),
                (bottom_right, color),
                (vec2(top_left.x, bottom_right.y), color),
                (top_left, color),
            ],
        );
    }

    pub fn add_circle(
        &mut self,
        context: &Context,
        center: Vec2,
        radius: f32,
        thickness: f32,
        segments: usize,
        start: f32,
        end: f32,
        color: Color,
    ) {
        let inner = vec2(radius - thickness, 0.0);
        let outer = vec2(radius + thickness, 0.0);
        let transform = Mat2::from_angle(start);
        let mut last_outer = center + transform.mul_vec2(outer);
        let mut last_inner = center + transform.mul_vec2(inner);
        for i in 0..=segments {
            let transform = Mat2::from_angle(start + i as f32 * (end - start) / segments as f32);
            let next_outer = center + transform.mul_vec2(outer);
            let next_inner = center + transform.mul_vec2(inner);
            self.triangles(
                context,
                [
                    (last_outer, color),
                    (next_outer, color),
                    (next_inner, color),
                    (next_inner, color),
                    (last_inner, color),
                    (last_outer, color),
                ],
            );
            last_outer = next_outer;
            last_inner = next_inner;
        }
    }

    pub fn add_circle_filled(
        &mut self,
        context: &Context,
        center: Vec2,
        radius: f32,
        segments: usize,
        start: f32,
        end: f32,
        color: Color,
    ) {
        let boundary = vec2(radius, 0.0);
        let transform = Mat2::from_angle(start);
        let mut last = center + transform.mul_vec2(boundary);
        for i in 0..=segments {
            let transform = Mat2::from_angle(start + i as f32 * (end - start) / segments as f32);
            let next = center + transform.mul_vec2(boundary);
            self.triangles(context, [(center, color), (last, color), (next, color)]);
            last = next;
        }
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
            glBindBuffer(GL_ARRAY_BUFFER, self.array_buffer_id);
            // According to https://thothonegan.tumblr.com/post/135193767243/glbuffersubdata-vs-glbufferdata
            // this is better as the GPU can work on the buffer without blocking
            glBufferData(
                GL_ARRAY_BUFFER,
                (size_of_val(&self.array_buffer_data[..])) as GLsizeiptr,
                self.array_buffer_data.as_ptr() as *const GLvoid,
                GL_STREAM_DRAW,
            );
            glDrawArrays(GL_TRIANGLES, 0, 3 * self.triangle_count as i32);
        }
        self.triangle_count = 0;
        self.array_buffer_data.clear();
    }

    pub fn with_max_triangles(context: &Context, max: u16) -> Self {
        let shader_id =
            crate::shader::load_shaders(&context, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();
        let matrix_id = unsafe {
            let mvp = CString::new("viewProjectionMatrix").unwrap();
            glGetUniformLocation(shader_id, mvp.as_ptr())
        };
        let array_buffer_data = Vec::with_capacity(3 * max as usize);
        let mut vertex_array_id: GLuint = 0;
        let mut array_buffer_id: GLuint = 0;
        let mut element_buffer_id: GLuint = 0;
        unsafe {
            glGenVertexArrays(1, &mut vertex_array_id as *mut GLuint);
            glBindVertexArray(vertex_array_id);
            glEnableVertexAttribArray(0);
            glEnableVertexAttribArray(1);

            glGenBuffers(1, &mut array_buffer_id as *mut GLuint);
            glBindBuffer(GL_ARRAY_BUFFER, array_buffer_id);
            let stride = size_of::<Vertex>() as i32;
            glVertexAttribPointer(
                0,
                2,
                GL_FLOAT,
                GL_FALSE as u8,
                stride,
                offset_of!(Vertex, pos) as *const GLvoid,
            );
            glVertexAttribPointer(
                1,
                4,
                GL_UNSIGNED_BYTE,
                GL_TRUE as u8,
                stride,
                offset_of!(Vertex, color) as *const GLvoid,
            );
        }
        Self {
            max_triangles: max,
            triangle_count: 0,
            vertex_array_id,
            array_buffer_id,
            array_buffer_data,
            shader_id,
            uniform_mvp: matrix_id,
            model_view_projection: Mat4::IDENTITY,
        }
    }

    pub fn set_model_view_projection_matrix(&mut self, matrix: Mat4) {
        self.model_view_projection = matrix;
    }
}

#[cfg(feature = "webgl1")]
const VERTEX_SHADER: &str = r#"#version 100
attribute vec2 vertex_pos;
attribute vec4 vertex_color;

varying lowp vec4 fragmentColor;

uniform mat4 viewProjectionMatrix;

void main() {
    gl_Position = viewProjectionMatrix * vec4(vertex_pos, 0.0, 1.0);
    fragmentColor = vertex_color;
}
"#;

#[cfg(feature = "webgl1")]
const FRAGMENT_SHADER: &str = r#"#version 100
varying lowp vec4 fragmentColor;

void main() {
    gl_FragColor = fragmentColor;
}
"#;
