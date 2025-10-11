use metal::*;

fn main() {
    // ------------------------------------------------------------
    // 1. Create a Metal device and command queue
    // ------------------------------------------------------------
    let device = Device::system_default().expect("❌ No Metal device available");
    println!("✅ Using GPU: {}", device.name());
    let queue = device.new_command_queue();

    // ------------------------------------------------------------
    // 2. Inline Metal shader source
    // ------------------------------------------------------------
    let shader_src = r#"
        #include <metal_stdlib>
        using namespace metal;

        kernel void hello(device char* out [[buffer(0)]],
                          uint tid [[thread_position_in_grid]]) {
            if (tid == 0) {
                const char msg[] = "Hello from Metal (Rust)!";
                for (uint i = 0; i < sizeof(msg); ++i)
                    out[i] = msg[i];
            }
        }
    "#;

    // ------------------------------------------------------------
    // 3. Compile and create pipeline
    // ------------------------------------------------------------
    let options = CompileOptions::new();
    let lib = device
        .new_library_with_source(shader_src, &options)
        .expect("compile Metal shader");
    let func = lib.get_function("hello", None).unwrap();

    // ✅ FIX: use the `_with_function` variant here
    let pso = device
        .new_compute_pipeline_state_with_function(&func)
        .expect("create pipeline");

    // ------------------------------------------------------------
    // 4. Create output buffer
    // ------------------------------------------------------------
    let buf = device.new_buffer(256, MTLResourceOptions::StorageModeShared);

    // ------------------------------------------------------------
    // 5. Encode and dispatch
    // ------------------------------------------------------------
    let cmd_buf = queue.new_command_buffer();
    let enc = cmd_buf.new_compute_command_encoder();
    enc.set_compute_pipeline_state(&pso);
    enc.set_buffer(0, Some(&buf), 0);
    enc.dispatch_threads(
        MTLSize { width: 1, height: 1, depth: 1 },
        MTLSize { width: 1, height: 1, depth: 1 },
    );
    enc.end_encoding();
    cmd_buf.commit();
    cmd_buf.wait_until_completed();

    // ------------------------------------------------------------
    // 6. Read result back and print
    // ------------------------------------------------------------
    unsafe {
        let ptr = buf.contents() as *const i8;
        let msg = std::ffi::CStr::from_ptr(ptr).to_str().unwrap_or("(invalid)");
        println!("Result buffer: \"{}\"", msg);
    }
}
