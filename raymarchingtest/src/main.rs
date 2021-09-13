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

    let _window = video_subsystem.window("lesson01", 800, 600).opengl().build().unwrap();
        _window.gl_create_context().unwrap();

    let _context = _window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let shader = link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        let length = size_of::<GLfloat>() as isize * 12;
        gl::BufferData(gl::ARRAY_BUFFER, length, QUAD_VERTICES.as_ptr() as *const gl::types::GLvoid, gl::STATIC_DRAW);

        gl::UseProgram(shader);

        let color_param = CString::new("out_color").unwrap();
        gl::BindFragDataLocation(shader, 0, color_param.as_ptr());

        let position_param = CString::new("position").unwrap();
        let attrib_position = gl::GetAttribLocation(shader, position_param.as_ptr());

        gl::EnableVertexAttribArray(attrib_position as GLuint);
        gl::VertexAttribPointer(attrib_position as GLuint, 3, gl::FLOAT, gl::FALSE as GLboolean, 3 * size_of::<GLfloat>() as GLint, 0 as *const _);
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
            let c_str = CString::new("iTime").unwrap();
            let location : i32 = gl::GetUniformLocation(shader, c_str.as_ptr());
            gl::Uniform1f(location, time);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
        
        _window.gl_swap_window();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        time += 1.0 / 60.0f32;
    }

    Ok(())
}

static VS_SRC: &'static str =
    "#version 150
    in vec2 position;
    void main() 
    {
        gl_Position = vec4(position, 0.0, 1.0);
    }";

static FS_SRC: &'static str =
   "#version 150
    uniform float iTime;
    out vec4 out_color;

    float smin(float a, float b, float k)
    {
        float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
        return mix(b, a, h) - k * h * (1.0 - h);
    }
    
    vec2 rotate2D(vec2 plane, float angle) 
    {
        return cos(angle) * plane + sin(angle) * vec2(plane.y,-plane.x);
    }
    
    vec3 rotate3D(vec3 p, vec3 axis, float angle) 
    {
        vec3 a = cross(axis, p);        
        vec3 b = cross(a, axis);
        
        return b * cos(angle) + a * sin(angle) + axis * dot(p, axis);   
    }
    
    vec3 fold(vec3 p, vec3 dir) 
    {
        return p + max(0.0, -2.0 * dot(p, dir)) * dir;
    }
    
    float spikeball(vec3 p, float angle)
    {	
        vec3 p0 = p; 
        vec3 p1 = p0;
    
        float d0 = 0.0;
        float d1 = sqrt(1.0 / 3.0);
        float d2 = sqrt(2.0 / 3.0);
        float d3 = sqrt(1.0 / 4.0);
        float d4 = sqrt(3.0 / 4.0);
            
        vec3 a = vec3(d0, d1, d2);
        vec3 b = vec3(d2 * d4, d1, -d2 * d3); 
        vec3 c = vec3(-d2 * d4, d1, -d2 * d3);
        
        p = normalize(p);
        p0 = normalize(p0);
        p1 = normalize(p1);
        
        float l = 2.6;
            
        p = fold(p, a);
        p = fold(p, b);
        p = fold(p, c);
        
        float spike = -smin(-(p.x + p.y + p.z)/l, -(angle - 1.7), 1.00);           
        
        p0 = rotate3D(p0, vec3(0,1,0), -90.0);
    
        p0 = fold(p0, a);
        p0 = fold(p0, b);
        p0 = fold(p0, c);    
        
        spike = smin(spike, -smin(-(p0.x + p0.y + p0.z)/l, -(angle - 1.7), 1.0), 0.24);
        
        p1 = rotate3D(p1, vec3(0,0,1), 90.0);
    
        p1 = fold(p1, a);
        p1 = fold(p1, b);
        p1 = fold(p1, c);    
        
        spike = smin(spike, -smin(-(p1.x + p1.y + p1.z)/l, -(angle - 1.7), 1.0), 0.24);
         
        return spike * 0.9;  
    }
    
    float map(vec3 p)
    {
        vec3 p0 = p;
        
        float angle = length(p);
       
        p.xy = rotate2D(p.xy, angle - iTime * 0.6);
        p.yz = rotate2D(p.yz, angle - iTime * 0.5);
        p.xz = rotate2D(p.xz, angle - iTime * 0.7);
           
        float d = spikeball(p, angle);   
        
        float sphere = length(p0) - 0.3;    
       
        d = smin(d, sphere, 0.3);    
        
        return d;
    }

    vec3 calcNormal(vec3 p)
    {
        float dist = map(p);
        return normalize(vec3(map(p + vec3(0.001, 0.0, 0.0)) - dist,
                              map(p + vec3(0.0, 0.001, 0.0)) - dist,
                              map(p + vec3(0.0, 0.0, 0.001)) - dist));
    }

    float Raymarch(vec3 origin, vec3 direction)
    {
        float t = 0.0;
        
        for (int i = 0; i < 64; i++) 
        {
            vec3 pos = origin + t * direction;
            float dist = map(pos);
            t += dist;
            
            if(dist < 0.001)
                return t - 0.001;  
        }
        return 0.0;
    }

    void main() 
    {
       vec2 uv = (2.0 * gl_FragCoord.xy - vec2(800.0, 600.0)) / 600.0;

       vec3 ro = vec3(0.1 * sin(iTime * 0.4), 0.1 * cos(iTime * 0.6), -7.0);
       vec3 rd = normalize(vec3(uv, 0.0) - ro);

       float dist = Raymarch(ro, rd);
       vec4 color = vec4(0.0, 0.0, 0.0, 1.0);

       if (dist > 0.0)
       {
        vec3 p = ro + dist * rd;
        vec3 N = calcNormal(ro + dist * rd);
        vec3 L = -rd;
        
        vec3 c0 = vec3(1,0.2,0.3) * N.y;
        color.rgb = c0;
        
        vec3 c1 = vec3(0.4,1,1) * -N.z;
        color.rgb += .5 * c1;
        
        vec3 c2 = vec3(0.5,0.5,1);
        
        float ambient = mod(0.5 + 0.45 * cos(dist * 7.0), 1.0);
        color.rgb *= 0.7 + ambient * c2;
       }
    
       out_color = color;
    }";

