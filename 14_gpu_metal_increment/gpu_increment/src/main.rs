use metal::*;
use std::time::{Duration, Instant};

/*------------------------------------------------------------
  GPU Kernel Runner (Metal)
  - Compiles and runs the same incremental shader on the GPU.
  - Prints identical, aligned throughput output.
  - Runs for 10 seconds and reports every ~0.25 s.
------------------------------------------------------------*/
fn main() {
    const COUNT: usize = 1_000_000;
    const RUN_FOR: Duration = Duration::from_secs(10);

    /*---------------------------------
      Enumerate and select Metal device
    ---------------------------------*/
    let devices = Device::all();
    println!("MTLCopyAllDevices() -> {} device(s)", devices.len());
    for (i, dev) in devices.iter().enumerate() {
        println!("  [{}] {}", i, dev.name());
    }

    let device = Device::system_default().expect("No Metal device found");
    println!("Using device: {}", device.name());

    /*---------------------------------

                SHADER
    
      Load and compile Metal shader file

      ---------------------------------*/

    let source = include_str!("../../incremental.metal");
    let lib = device
        .new_library_with_source(source, &CompileOptions::new())
        .expect("Failed to compile Metal shader");
    let kernel = lib.get_function("increment", None).unwrap();
    let pipeline = device
        .new_compute_pipeline_state_with_function(&kernel)
        .expect("Failed to create compute pipeline state");

    /*----------------------------
      Allocate and initialize data
    ----------------------------*/
    let buffer = device.new_buffer(
        (COUNT * std::mem::size_of::<i32>()) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    let mut data = vec![1_i32; COUNT];
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), buffer.contents() as *mut i32, COUNT);
    }

    /*---------------------------------
      Main GPU execution control loop
    ---------------------------------*/
    let queue = device.new_command_queue();
    let start_time = Instant::now();
    let mut last_ops: u128 = 0;
    let mut total_ops: u128 = 0;
    let mut pass: u64 = 0;
    let mut last_report = Instant::now();

    println!("Running GPU kernel on {} elements", COUNT);

    loop {
        pass += 1;
        total_ops += COUNT as u128;

        let cmd_buffer = queue.new_command_buffer();
        let encoder = cmd_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_buffer(0, Some(&buffer), 0);

        let thread_group_size = MTLSize {
            width: pipeline.thread_execution_width(),
            height: 1,
            depth: 1,
        };
        let thread_groups = MTLSize {
            width: (COUNT as u64 + thread_group_size.width - 1) / thread_group_size.width,
            height: 1,
            depth: 1,
        };

        encoder.dispatch_thread_groups(thread_groups, thread_group_size);
        encoder.end_encoding();
        cmd_buffer.commit();
        cmd_buffer.wait_until_completed();

        // Print throughput every ~0.25 s
        if last_report.elapsed().as_secs_f64() > 0.25 {
            let elapsed = last_report.elapsed().as_secs_f64();
            let ops_since = total_ops - last_ops;
            let throughput = (ops_since as f64) / (elapsed * 1e6);
            unsafe {
                let first_val = *(buffer.contents() as *const i32);
                println!(
                    "{:<7} pass {:>8} | total {:>15} ops | +{:>15} since last | {:>10.2} M ops/s | first element = {:>8}",
                    "GPU",
                    pass,
                    total_ops,
                    ops_since,
                    throughput,
                    first_val
                );
            }
            last_report = Instant::now();
            last_ops = total_ops;
        }

        // Stop after the configured run time
        if start_time.elapsed() >= RUN_FOR {
            println!("\nReached 10-second limit, exiting.");
            break;
        }
    }
}
