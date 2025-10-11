use metal::*;
use std::{mem, time::Instant};

fn main() {
    // ------------------------------------------------------------
    // 1. Setup Metal device and queue
    // ------------------------------------------------------------
    let device = Device::system_default().expect("No Metal device available");
    println!("Using GPU: {}", device.name());
    let queue = device.new_command_queue();

    // ------------------------------------------------------------
    // 2. Metal shader
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

    let opts = CompileOptions::new();
    let lib = device
        .new_library_with_source(shader_src, &opts)
        .expect("Failed to compile Metal shader");
    let func = lib.get_function("increment", None).unwrap();
    let pso = device
        .new_compute_pipeline_state_with_function(&func)
        .expect("Failed to create pipeline");

    // ------------------------------------------------------------
    // 3. Shared buffer
    // ------------------------------------------------------------
    const COUNT: usize = 1_000_000;
    let mut data = vec![1_i32; COUNT];
    let buf = device.new_buffer_with_data(
        data.as_ptr() as *const _,
        mem::size_of_val(&*data) as u64,
        MTLResourceOptions::StorageModeShared,
    );

    // ------------------------------------------------------------
    // 4. Infinite GPU loop with throughput counter
    // ------------------------------------------------------------
    let mut pass: u64 = 0;
    let mut total_ops: u128 = 0;
    let mut last_ops: u128 = 0;
    let mut last_report = Instant::now();

    loop {
        pass += 1;
        total_ops += COUNT as u128;

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

        // print every 100 passes
        if pass % 100 == 0 {
            let elapsed = last_report.elapsed().as_secs_f64();
            let ops_since = total_ops - last_ops;
            let throughput = (ops_since as f64) / (elapsed * 1e6); // million ops/sec

            unsafe {
                let first = *(buf.contents() as *const i32);
                println!(
                    "Pass {:>8} | total {:>12} ops | +{:>12} since last | {:.2} M ops/s | first element = {}",
                    pass,
                    total_ops,
                    ops_since,
                    throughput,
                    first
                );
            }

            last_report = Instant::now();
            last_ops = total_ops;
        }
    }
}
