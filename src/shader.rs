use crate::gl::types::*;
use crate::gl::*;
use std::ffi::CString;

unsafe fn compile_shader(
    gl: &Gl,
    shader_id: GLuint,
    shader: &str,
    shader_type: &str,
) -> Result<(), String> {
    let shader = CString::new(shader).expect("Invalid shader code");
    gl.ShaderSource(shader_id, 1, &shader.as_ptr(), std::ptr::null());
    gl.CompileShader(shader_id);

    let mut result: GLint = 0;
    gl.GetShaderiv(shader_id, COMPILE_STATUS, &mut result);
    let mut info_log_length: GLint = 0;
    gl.GetShaderiv(shader_id, INFO_LOG_LENGTH, &mut info_log_length);
    if info_log_length > 0 {
        let mut shader_error_message: Vec<u8> = Vec::with_capacity(info_log_length as usize);
        gl.GetShaderInfoLog(
            shader_id,
            info_log_length,
            0 as *mut i32,
            shader_error_message.as_mut_ptr() as *mut i8,
        );
        shader_error_message.set_len(info_log_length as usize);
        return Err(format!(
            "Error in {}-shader: {}",
            shader_type,
            String::from_utf8(shader_error_message).expect("Error message not parsable")
        ));
    }
    Ok(())
}

pub(crate) fn load_shaders(
    gl: &Gl,
    vertex_shader: &str,
    fragment_shader: &str,
) -> Result<GLuint, String> {
    unsafe {
        let vertex_shader_id = gl.CreateShader(VERTEX_SHADER);
        let fragment_shader_id = gl.CreateShader(FRAGMENT_SHADER);

        compile_shader(gl, vertex_shader_id, vertex_shader, "VERTEX")?;
        compile_shader(gl, fragment_shader_id, fragment_shader, "FRAGMENT")?;

        let program_id = gl.CreateProgram();
        gl.AttachShader(program_id, vertex_shader_id);
        gl.AttachShader(program_id, fragment_shader_id);
        gl.LinkProgram(program_id);

        let mut result: GLint = 0;
        gl.GetProgramiv(program_id, LINK_STATUS, &mut result);
        let mut info_log_length = 0;
        gl.GetProgramiv(program_id, INFO_LOG_LENGTH, &mut info_log_length);
        if info_log_length > 0 {
            let mut program_error_message: Vec<u8> = Vec::with_capacity(info_log_length as usize);
            gl.GetProgramInfoLog(
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

        gl.DetachShader(program_id, vertex_shader_id);
        gl.DetachShader(program_id, fragment_shader_id);

        gl.DeleteShader(vertex_shader_id);
        gl.DeleteShader(fragment_shader_id);
        Ok(program_id)
    }
}
