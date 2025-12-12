use glutin::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
use glow::HasContext;

fn main() {
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Rust OpenGL Triangle")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let ctx = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();
    let ctx = unsafe { ctx.make_current().unwrap() };
    let gl = unsafe { glow::Context::from_loader_function(|s| ctx.get_proc_address(s) as *const _) };

    // --- Shader setup ---
    let vs_src = r#"#version 330 core
        const vec2 verts[3] = vec2[3](
            vec2( 0.0,  0.5),
            vec2(-0.5, -0.5),
            vec2( 0.5, -0.5)
        );
        void main() {
            gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
        }"#;

    let fs_src = r#"#version 330 core
        out vec4 color;
        void main() { color = vec4(0.9, 0.3, 0.2, 1.0); }"#;

    unsafe {
        let program = gl.create_program().unwrap();

        let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vs, vs_src);
        gl.compile_shader(vs);
        assert!(gl.get_shader_compile_status(vs), "vertex shader failed");

        let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(fs, fs_src);
        gl.compile_shader(fs);
        assert!(gl.get_shader_compile_status(fs), "fragment shader failed");

        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        assert!(gl.get_program_link_status(program), "link failed");

        gl.delete_shader(vs);
        gl.delete_shader(fs);

        // FIX: Create and bind a dummy VAO
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        el.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::MainEventsCleared => ctx.window().request_redraw(),
                Event::RedrawRequested(_) => {
                    gl.clear_color(0.1, 0.1, 0.12, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                    gl.use_program(Some(program));
                    gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    ctx.swap_buffers().unwrap();
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }
        });
    }
}
