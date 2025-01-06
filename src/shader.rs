use crate::Context;
use miniquad::gl::*;

unsafe fn compile_shader(
    _context: &Context,
    shader_id: GLuint,
    shader: &str,
    shader_type: &str,
) -> Result<(), String> {
    let len = shader.len();
    glShaderSource(shader_id, 1, &(shader.as_ptr() as *const i8), &(len as i32));
    glCompileShader(shader_id);

    let mut result: GLint = 0;
    glGetShaderiv(shader_id, GL_COMPILE_STATUS, &mut result);
    let mut info_log_length: GLint = 0;
    glGetShaderiv(shader_id, GL_INFO_LOG_LENGTH, &mut info_log_length);
    if result == 0 && info_log_length > 0 {
        let mut shader_error_message: Vec<u8> = Vec::with_capacity(info_log_length as usize);
        glGetShaderInfoLog(
            shader_id,
            info_log_length,
            0 as *mut i32,
            shader_error_message.as_mut_ptr() as *mut i8,
        );
        shader_error_message.set_len(info_log_length as usize - 1);
        return Err(format!(
            "Error in {}-shader: {}",
            shader_type,
            String::from_utf8(shader_error_message).expect("Error message not parsable")
        ));
    }
    Ok(())
}

pub(crate) fn load_shaders(
    context: &Context,
    vertex_shader: &str,
    fragment_shader: &str,
) -> Result<GLuint, String> {
    unsafe {
        let vertex_shader_id = glCreateShader(GL_VERTEX_SHADER);
        let fragment_shader_id = glCreateShader(GL_FRAGMENT_SHADER);

        compile_shader(context, vertex_shader_id, vertex_shader, "VERTEX")?;
        compile_shader(context, fragment_shader_id, fragment_shader, "FRAGMENT")?;

        let program_id = glCreateProgram();
        glAttachShader(program_id, vertex_shader_id);
        glAttachShader(program_id, fragment_shader_id);
        glLinkProgram(program_id);

        let mut result: GLint = 0;
        glGetProgramiv(program_id, GL_LINK_STATUS, &mut result);
        let mut info_log_length = 0;
        glGetProgramiv(program_id, GL_INFO_LOG_LENGTH, &mut info_log_length);
        if result == 0 && info_log_length > 0 {
            let mut program_error_message: Vec<u8> = Vec::with_capacity(info_log_length as usize);
            glGetProgramInfoLog(
                program_id,
                info_log_length,
                0 as *mut i32,
                program_error_message.as_mut_ptr() as *mut i8,
            );
            program_error_message.set_len(info_log_length as usize);
            return Err(
                String::from_utf8(program_error_message).expect("Error message not parsable")
            );
        }

        Ok(program_id)
    }
}
