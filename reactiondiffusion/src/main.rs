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
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA32F as i32, x_res as i32, y_res as i32, 0, gl::RGBA, gl::FLOAT, 0 as *const _);

            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffers[i]);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, textures[i], 0);
            gl::Uniform1i(3 + i as i32, i as i32);
        }
    }

    let mut event_pump = sdl_context.event_pump()?;
    let mut time : f32 = 0.0;
    let mut frame = 0;

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
            gl::Uniform1f(0, time);
            gl::Uniform1i(1, frame);

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

        frame = frame + 1;
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
    layout (location = 1) uniform int iFrame;
    layout (location = 2) uniform int iPass;
    layout (location = 3) uniform sampler2D iChannel0;
    layout (location = 4) uniform sampler2D iChannel1;

    #define R vec2(800.,600.)
    #define TS(y,u) texture(t,u)
    #define T(t,u) texture(t,(u)/r)
    #define rot(a) mat2(cos(a),-sin(a),sin(a),cos(a))

    out vec4 C;

    float rand(vec2 n) 
    { 
        return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
    }

    vec2 I(sampler2D t,vec2 u,float o,vec2 r)
    {
        vec2 f=vec2(1,0)*o;
        return vec2(T(t,u+f.xy).r-T(t,u-f.xy).r,T(t,u+f.yx).r-T(t,u-f.yx).r);
    }

    vec4 B(sampler2D t, vec2 u, float o, vec2 r)
    {
        vec3 f=vec3(-1,0,1)*o;
        return (T(t,u+f.xx)+T(t,u+f.yx)+T(t,u+f.zx)+
                T(t,u+f.xy)+T(t,u+f.yy)+T(t,u+f.zy)+
                T(t,u+f.xz)+T(t,u+f.yz)+T(t,u+f.zz)) / 9.0;
    }

    vec4 S(sampler2D t, vec2 u, float o, vec2 r, float s)
    {
        vec2 step = (1./r+.01*rand(u*99))*o;
        vec4 f=vec4(-1,1,-1,1)*step.xxyy;
        return T(t,u*r)+(T(t,u*r)-(TS(t,u+f.zx)+TS(t,u+f.yz)+TS(t,u+f.xw)+TS(t,u+f.yw))/4)*s;
    }  

   void main()
   {
    vec2 U=gl_FragCoord.xy/R;

    if (iPass==0)
    {
        if ((iFrame&511)==0) // Noising
        {
            C=vec4(rand(U*80.));
        }
        else // Zoom, Rotate & Blur
        {
            vec2 d=I(iChannel0,gl_FragCoord.xy,4.5*fract(iTime*1.5)*1.5,R);
            U=(U-.5)*(1-fract(iTime*1.5)*.002)*rot(.001+.1*pow(fract(iTime*.075),17)*.1)+.5-d*.0045*sin(iTime);

            C=B(iChannel0,U*R,.5,R);
        }
    }
     if (iPass==1) // Sharpening
     {
        C=S(iChannel0,U,1.5,R,200);   
     }

     if (iPass==2) // Post processing (vignette & greyscale)
     {
        float v=pow(3.*(U.x)*U.y*(1.-U.x)*(1.-U.y),.7);
        C=v*texture(iChannel1,U).xxxx*1.5;
     }
   }";
