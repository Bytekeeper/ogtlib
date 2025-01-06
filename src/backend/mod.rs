use crate::Color;
use glam::IVec2;

#[cfg(feature = "miniquad")]
pub mod miniquad;

pub(crate) fn configure_blend() {
    #[cfg(feature = "miniquad")]
    unsafe {
        use ::miniquad::gl::*;

        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }
}

pub(crate) fn clear_screen(color: Color) {
    #[cfg(feature = "miniquad")]
    unsafe {
        use ::miniquad::gl::*;

        glClearColor(0.0, 0.0, 0.4, 0.0);
        let c: [f32; 4] = color.0.map(|c| c as f32 / 255.0).into();
        glClearColor(c[0], c[1], c[2], c[3]);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }
}

pub(crate) fn set_viewport(top_left: IVec2, bottom_right: IVec2) {
    #[cfg(feature = "miniquad")]
    unsafe {
        use ::miniquad::gl::*;

        glViewport(top_left.x, top_left.y, bottom_right.x, bottom_right.y);
    }
}

pub(crate) fn load_file(file: &str, handler: impl FnOnce(Result<Vec<u8>, ()>) + 'static) {
    #[cfg(feature = "miniquad")]
    {
        let delegate = std::cell::RefCell::new(Some(handler));
        ::miniquad::fs::load_file(file, move |result| {
            let handler = delegate.take().unwrap();
            match result {
                Ok(content) => handler(Ok(content)),
                Err(err) => handler(Err(())),
            }
        });
    }
}
