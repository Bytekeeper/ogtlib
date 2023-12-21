use crate::Context;
use miniquad::gl::*;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub top_left: [f32; 2],
    pub bottom_right: [f32; 2],
}

struct GLTexture(GLuint);

#[derive(Clone)]
pub struct Texture {
    _gl_texture: Rc<GLTexture>,
    pub(crate) texture_id: GLuint,
    pub width: u32,
    pub height: u32,
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.texture_id == other.texture_id
    }
}

impl Texture {
    pub fn bind(&self, ctx: &Context) {
        unsafe { glBindTexture(GL_TEXTURE_2D, self.texture_id) }
    }
}

impl Drop for GLTexture {
    fn drop(&mut self) {
        unsafe {
            glDeleteTextures(1, &mut self.0);
        }
    }
}

pub struct TextureBuilder<'a> {
    data: &'a [u8],
    width: u32,
    height: u32,
    mag_filter: i32,
    min_filter: i32,
}

impl<'a> TextureBuilder<'a> {
    pub fn from_bytes(data: &'a [u8], width: u32, height: u32) -> TextureBuilder<'a> {
        assert_eq!(data.len() as u32, width * height * 4);
        TextureBuilder {
            data,
            width,
            height,
            mag_filter: GL_LINEAR as i32,
            min_filter: GL_LINEAR as i32,
        }
    }

    pub fn build(self, context: &Context) -> Texture {
        let texture = unsafe {
            let mut texture_id = 0;
            glGenTextures(1, &mut texture_id);
            Texture {
                _gl_texture: Rc::new(GLTexture(texture_id)),
                texture_id,
                width: self.width,
                height: self.height,
            }
        };
        texture.bind(context);
        unsafe {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, self.mag_filter);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, self.min_filter);

            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA as i32,
                self.width as i32,
                self.height as i32,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                self.data.as_ptr() as *const GLvoid,
            );
            if self.width.is_power_of_two() && self.height.is_power_of_two() {
                glGenerateMipmap(GL_TEXTURE_2D);
            } else {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as i32);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as i32);
            }
        }
        texture
    }
}
