use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use glow::HasContext;

fn main() {
    // create event loop and windowed context (glutin 0.29 style)
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Bounding Cube - glutin+glow")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let windowed_context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();

    // make current
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    // load glow using context's get_proc_address
    let gl = unsafe {
        glow::Context::from_loader_function(|s| windowed_context.get_proc_address(s) as *const _)
    };

    // --- shaders ---
    let vs_src = r#"#version 330 core
        layout(location = 0) in vec3 aPos;
        uniform mat4 uMVP;
        void main() {
            gl_Position = uMVP * vec4(aPos, 1.0);
        }
    "#;

    let fs_src = r#"#version 330 core
        out vec4 FragColor;
        void main() { FragColor = vec4(1.0, 1.0, 1.0, 1.0); }
    "#;

    unsafe {
        // compile/link
        let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vs, vs_src);
        gl.compile_shader(vs);
        if !gl.get_shader_compile_status(vs) {
            panic!("VS compile error: {}", gl.get_shader_info_log(vs));
        }

        let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(fs, fs_src);
        gl.compile_shader(fs);
        if !gl.get_shader_compile_status(fs) {
            panic!("FS compile error: {}", gl.get_shader_info_log(fs));
        }

        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("Linker error: {}", gl.get_program_info_log(program));
        }
        gl.delete_shader(vs);
        gl.delete_shader(fs);

        // cube edges (pairs of points for GL_LINES)
        // 12 edges * 2 vertices = 24 vertices, each vertex 3 floats -> 72 floats
        let vertices: [f32; 72] = [
            // bottom square
            -1.0, -1.0, -1.0,   1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,   1.0, -1.0,  1.0,
             1.0, -1.0,  1.0,  -1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,  -1.0, -1.0, -1.0,
            // top square
            -1.0,  1.0, -1.0,   1.0,  1.0, -1.0,
             1.0,  1.0, -1.0,   1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,  -1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,  -1.0,  1.0, -1.0,
            // verticals
            -1.0, -1.0, -1.0,  -1.0,  1.0, -1.0,
             1.0, -1.0, -1.0,   1.0,  1.0, -1.0,
             1.0, -1.0,  1.0,   1.0,  1.0,  1.0,
            -1.0, -1.0,  1.0,  -1.0,  1.0,  1.0,
        ];

        // VAO + VBO
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&vertices),
            glow::STATIC_DRAW,
        );

        gl.enable_vertex_attrib_array(0);
        let stride = 3 * std::mem::size_of::<f32>() as i32;
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);

        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        // GL state
        gl.clear_color(0.05, 0.05, 0.1, 1.0);
        gl.enable(glow::DEPTH_TEST);

        // uniform location
        let u_mvp_loc = gl.get_uniform_location(program, "uMVP");

        // simple animation state
        let mut angle: f32 = 0.0;

        // event loop
        el.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => {
                    windowed_context.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    // build MVP
                    let aspect = {
                        let size = windowed_context.window().inner_size();
                        size.width as f32 / size.height as f32
                    };

                    // using glam for matrices
                    let proj = glam::Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 0.1, 100.0);
                    let view = glam::Mat4::look_at_rh(
                        glam::Vec3::new(0.0, 0.0, 5.0),
                        glam::Vec3::ZERO,
                        glam::Vec3::Y,
                    );
                    let model = glam::Mat4::from_rotation_y(angle.to_radians());
                    let mvp = proj * view * model;

                    angle += 0.6;
                    if angle >= 360.0 { angle -= 360.0; }

                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                    gl.use_program(Some(program));
                    if let Some(loc) = &u_mvp_loc {
                        gl.uniform_matrix_4_f32_slice(Some(loc), false, &mvp.to_cols_array());
                    }

                    gl.bind_vertex_array(Some(vao));
                    gl.draw_arrays(glow::LINES, 0, (vertices.len() / 3) as i32);
                    gl.bind_vertex_array(None);

                    windowed_context.swap_buffers().unwrap();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(glutin::event::VirtualKeyCode::Escape) = input.virtual_keycode {
                            if input.state == glutin::event::ElementState::Pressed {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    }
}
