use metal::*;
use std::{mem, ptr, thread, time::Instant};

fn main() {
    // ------------------------------------------------------------
    // 1. Setup Metal device and command queue
    // ------------------------------------------------------------
    let device = Device::system_default().expect("No Metal device available");
    println!("Using GPU: {}", device.name());
    let queue = device.new_command_queue();

    // ------------------------------------------------------------
    // 2. Metal shader source: increment each element by 1
    // ------------------------------------------------------------
    let shader_src = r#"
        #include <metal_stdlib>
        using namespace metal;

        kernel void increment(device int *data [[buffer(0)]],
                              uint  count [[threads_per_grid]],
                              uint  tid   [[thread_position_in_grid]]) {
            if (tid < count)
                data[tid] += 1;
        }
    "#;

    // ------------------------------------------------------------
    // 3. Compile shader and create pipeline
    // ------------------------------------------------------------
    let opts = CompileOptions::new();
    let lib = device
        .new_library_with_source(shader_src, &opts)
        .expect("Failed to compile Metal shader");
    let func = lib.get_function("increment", None).unwrap();
    let pso = device
        .new_compute_pipeline_state_with_function(&func)
        .expect("Failed to create pipeline");

    // ------------------------------------------------------------
    // 4. Create large shared buffer
    // ------------------------------------------------------------
    const COUNT: usize = 1_000_000;
    let mut data = vec![1_i32; COUNT];
    let buf = device.new_buffer_with_data(
        data.as_ptr() as *const _,
        mem::size_of_val(&*data) as u64,
        MTLResourceOptions::StorageModeShared,
    );

    // ------------------------------------------------------------
    // 5. Infinite GPU dispatch loop
    // ------------------------------------------------------------
    let mut pass: u64 = 0;
    let mut last_report = Instant::now();

    loop {
        pass += 1;

        // Launch GPU kernel
        let cmd_buf = queue.new_command_buffer();
        let enc = cmd_buf.new_compute_command_encoder();
        enc.set_compute_pipeline_state(&pso);
        enc.set_buffer(0, Some(&buf), 0);

        let grid = MTLSize {
            width: COUNT as u64,
            height: 1,
            depth: 1,
        };
        let tg = MTLSize {
            width: 256,
            height: 1,
            depth: 1,
        };

        enc.dispatch_threads(grid, tg);
        enc.end_encoding();
        cmd_buf.commit();
        cmd_buf.wait_until_completed();

        // Periodically print progress (every 100 iterations)
        if pass % 100 == 0 {
            let elapsed = last_report.elapsed().as_micros();
            last_report = Instant::now();

            unsafe {
                ptr::copy_nonoverlapping(buf.contents() as *const i32, data.as_mut_ptr(), COUNT);
            }

            let sample = data[0];
            println!(
                "Pass {:>8}: first element = {:>8}, avg time = {:>7.2} µs per pass",
                pass,
                sample,
                elapsed as f64 / 100.0
            );
        }

        // Small sleep so printing doesn’t dominate runtime
        // (remove this for max throughput)
        // thread::sleep(std::time::Duration::from_millis(1));
    }
}
