use glutin::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
use glow::HasContext;

// Import windowing, event loop, and GL context helpers.
fn main() {

    // Establish the platform event loop before we create windows.
    let el = glutin::event_loop::EventLoop::new();

    // Prepare a window builder with a fixed size and friendly title.
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Rust OpenGL Triangle")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));

    // Build the OpenGL context and make it current so GL calls succeed.
    let ctx = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();
    let ctx = unsafe { ctx.make_current().unwrap() };

    // Load OpenGL function pointers through glow to issue GPU calls.
    let gl = unsafe { glow::Context::from_loader_function(|s| ctx.get_proc_address(s) as *const _) };

    // --- Shader setup ---
    // Simple vertex shader emits three hard-coded clip-space positions.
    let vs_src = r#"#version 330 core
        const vec2 verts[3] = vec2[3](
            vec2( 0.0,  0.5),
            vec2(-0.5, -0.5),
            vec2( 0.5, -0.5)
        );
        void main() {
            gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
        }"#;

    // Fragment shader paints the triangle with a reddish color.
    let fs_src = r#"#version 330 core
        out vec4 color;
        void main() { color = vec4(0.9, 0.3, 0.2, 1.0); }"#;

    unsafe {

        // Every GL shader program is identified by a GLuint handle.
        let program = gl.create_program().unwrap();

        // Compile the vertex shader and confirm it succeeded.
        let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vs, vs_src);
        gl.compile_shader(vs);
        assert!(gl.get_shader_compile_status(vs), "vertex shader failed");

        // Compile the fragment shader and ensure the GPU accepted it.
        let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(fs, fs_src);
        gl.compile_shader(fs);
        assert!(gl.get_shader_compile_status(fs), "fragment shader failed");

        // Once attached, link the program to produce executable GPU code.
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        assert!(gl.get_program_link_status(program), "link failed");

        // Individual shader objects can be freed after linking.
        gl.delete_shader(vs);
        gl.delete_shader(fs);

        // FIX: Create and bind a dummy VAO because core profile contexts require one before drawing.
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        el.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            // Wait for events so we only redraw when necessary.
            match event {
                Event::MainEventsCleared => ctx.window().request_redraw(),

                // Request a redraw when the event queue is flushed.
                Event::RedrawRequested(_) => {

                    // Draw a frame, then swap buffers to show it.
                    gl.clear_color(0.1, 0.1, 0.12, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                    gl.use_program(Some(program));
                    gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    ctx.swap_buffers().unwrap();
                }

                // Gracefully exit if the user asks to close the window.
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                }

                // Ignore all other events and continue waiting.
                _ => (),
            }
        });
    }
}
