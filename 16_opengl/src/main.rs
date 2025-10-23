use glutin::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
use glow::HasContext;

fn main() {
    // Create event loop and window
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Hello OpenGL (Rust)")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));

    // Create OpenGL context
    let ctx = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();

    // Make current
    let ctx = unsafe { ctx.make_current().unwrap() };

    // Load OpenGL via glow
    let gl = unsafe { glow::Context::from_loader_function(|s| ctx.get_proc_address(s) as *const _) };

    // Event loop
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::MainEventsCleared => {
                // Redraw request every frame
                ctx.window().request_redraw();
            }

            Event::RedrawRequested(_) => unsafe {
                // Clear the screen to a bluish color
                gl.clear_color(0.2, 0.4, 0.6, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
                ctx.swap_buffers().unwrap();
            },

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            _ => (),
        }
    });
}
