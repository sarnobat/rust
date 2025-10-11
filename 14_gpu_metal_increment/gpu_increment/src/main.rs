use metal::*;
use std::{fs, mem, time::Instant};

fn main() {
    let device = Device::system_default().expect("No Metal device available");
    println!("Using GPU: {}", device.name());
    let queue = device.new_command_queue();

    // --- load shared Metal source file ---
    let shader_src = fs::read_to_string("incremental.metal")
        .expect("Failed to read incremental.metal");

    let opts = CompileOptions::new();
    let lib = device
        .new_library_with_source(&shader_src, &opts)
        .expect("Failed to compile Metal shader");
    let func = lib.get_function("increment", None).unwrap();
    let pso = device
        .new_compute_pipeline_state_with_function(&func)
        .expect("Failed to create pipeline");

    const COUNT: usize = 100_000_000;
    let mut data = vec![1_i32; COUNT];
    let buf = device.new_buffer_with_data(
        data.as_ptr() as *const _,
        mem::size_of_val(&*data) as u64,
        MTLResourceOptions::StorageModeShared,
    );

    let mut pass: u64 = 0;
    let mut total_ops: u128 = 0;
    let mut last_ops: u128 = 0;
    let mut last_report = Instant::now();

    const BATCH_SIZE: usize = 10;

    loop {
        let mut pending = Vec::with_capacity(BATCH_SIZE);

        for _ in 0..BATCH_SIZE {
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

            pending.push(cmd_buf);
        }

        for cb in pending {
            cb.wait_until_completed();
        }

        if pass % 100 == 0 {
            let elapsed = last_report.elapsed().as_secs_f64();
            let ops_since = total_ops - last_ops;
            let throughput = (ops_since as f64) / (elapsed * 1e6);
            unsafe {
                let first = *(buf.contents() as *const i32);
                println!(
                    "GPU pass {:>8} | total {:>15} ops | +{:>15} since last | {:>10.2} M ops/s | first element = {:>8}",
                    pass, total_ops, ops_since, throughput, first
                );
            }
            last_ops = total_ops;
            last_report = Instant::now();
        }
    }
}
