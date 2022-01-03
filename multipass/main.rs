extern crate gl;

use core::mem::size_of;

use std::time::Duration;
use std::ffi::CString;

use sdl2::video::{GLProfile};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use gl::types::*;

static QUAD_VERTICES : [ GLfloat; 12 ] = [
    -1.0, -1.0, 0.0,
     1.0, -1.0, 0.0,
    -1.0,  1.0, 0.0,
     1.0,  1.0, 0.0,
 ];

fn compile_shader(shader_source: &str, shader_type: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(shader_type);

        let c_str = CString::new(shader_source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), 0 as *const _);

        gl::CompileShader(shader);

        let mut params = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut params);
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        let mut params = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut params);

        program
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().expect("Failed to initialize SDL context");

    let video_subsystem = sdl_context.video().expect("Failed to initialize SDL video subsystem");

    let gl_attributes = video_subsystem.gl_attr();
        gl_attributes.set_context_profile(GLProfile::Core);
        gl_attributes.set_context_version(3, 3);

    let x_res = 800;
    let y_res = 600;
    let _window = video_subsystem.window("outline 4k", x_res, y_res).opengl().build().unwrap();
        _window.gl_create_context().unwrap();

    let _context = _window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let shader = link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;

    let mut textures : [GLuint; 2] = [0, 0];
    let mut framebuffers : [GLuint; 3] = [0, 0, 0];

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        let length = size_of::<GLfloat>() as isize * 12;
        gl::BufferData(gl::ARRAY_BUFFER, length, QUAD_VERTICES.as_ptr() as *const gl::types::GLvoid, gl::STATIC_DRAW);

        gl::UseProgram(shader);

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, 0, 3 * size_of::<GLfloat>() as GLint, 0 as *const _);

        gl::GenTextures(2, &mut textures[0]);
        gl::GenFramebuffers(2, &mut framebuffers[0]);
        gl::DrawBuffer(gl::COLOR_ATTACHMENT0);

        for i in 0..2 {
            gl::ActiveTexture(gl::TEXTURE0 + (i as u32));
            gl::BindTexture(gl::TEXTURE_2D, textures[i]);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, x_res as i32, y_res as i32, 0, gl::RGBA, gl::FLOAT, 0 as *const _);

            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffers[i]);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, textures[i], 0);
            gl::Uniform1i(3 + i as i32, i as i32);
        }
    }

    let mut event_pump = sdl_context.event_pump()?;
    let mut time : f32 = 0.0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        
            gl::Uniform1f(0, time);

            for i in 0..3 {
                gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffers[i]);
                gl::Uniform1i(2, i as i32);

                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                gl::Flush();
            }
        } 
        
        _window.gl_swap_window();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        time += 1.0 / 60.0f32;
    }

    Ok(())
}

static VS_SRC: &'static str =
    "#version 450
    in vec2 position;
    void main() 
    {
        gl_Position = vec4(position, 0.0, 1.0);
    }";
    
static FS_SRC: &'static str =
   "#version 450

   layout (location = 0) uniform float iTime;
   layout (location = 2) uniform int iPass;
   layout (location = 3) uniform sampler2D iChannel0;
   layout (location = 4) uniform sampler2D iChannel1;
   
   out vec4 out_color;

   void main() // rood, groen, uv
   {
       vec2 iResolution = vec2(800,600);
       vec2 uv = (gl_FragCoord.xy - 0.5 * iResolution.xy) / iResolution.y;
       if (iPass == 0)
        out_color = vec4(uv.x, uv.y, 0.0, 1.0);
        if (iPass == 1)
        out_color = vec4(0.0, uv.y, uv.x, 1.0);
        if (iPass == 2)
        out_color = texture(iChannel1, uv);
    
   }";
