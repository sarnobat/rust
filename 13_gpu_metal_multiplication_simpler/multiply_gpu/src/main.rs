use metal::*;
use objc::rc::autoreleasepool;
use std::env;

fn main() {
    autoreleasepool(|| {
        // Parse factor from command-line, default 3
        let args: Vec<String> = env::args().collect();
        let factor: f32 = args.get(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(3.0);

        // Input array
        let input = vec![0f32, 1., 1., 0., 1., 0., 1., 0.];
        let count = input.len();

        // Get GPU device and queue
        let device = Device::system_default().expect("No Metal device found");
        let command_queue = device.new_command_queue();

        // Allocate Metal buffer for input/output
        let buffer = device.new_buffer(
            (count * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        );
        unsafe {
            std::ptr::copy_nonoverlapping(input.as_ptr(), buffer.contents() as *mut f32, count);
        }

        // Allocate Metal buffer for factor
        let factor_buf = device.new_buffer(
            std::mem::size_of::<f32>() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        );
        unsafe {
            *(factor_buf.contents() as *mut f32) = factor;
        }

        // Metal kernel as string
        let kernel_src = r#"
        kernel void multiply(device float* data [[buffer(0)]],
                             device float* factor [[buffer(1)]],
                             uint id [[thread_position_in_grid]]) {
            if (id < 1000) data[id] *= factor[0];
        }"#;

        let library = device.new_library_with_source(kernel_src, &CompileOptions::new())
            .expect("Failed to compile Metal kernel");
        let function = library.get_function("multiply", None)
            .expect("Failed to get function");

        // Compute pipeline
        let descriptor = ComputePipelineDescriptor::new();
        descriptor.set_compute_function(Some(&function));
        let pipeline = device.new_compute_pipeline_state(&descriptor)
            .expect("Failed to create pipeline");

        // Encode commands
        let command_buffer = command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_buffer(0, Some(&buffer), 0);
        encoder.set_buffer(1, Some(&factor_buf), 0);

        let threads = MTLSize { width: count as u64, height: 1, depth: 1 };
        let threads_per_group = MTLSize { width: 1, height: 1, depth: 1 };
        encoder.dispatch_threads(threads, threads_per_group);
        encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();

        // Copy result back
        let mut output = vec![0f32; count];
        unsafe {
            std::ptr::copy_nonoverlapping(buffer.contents() as *const f32, output.as_mut_ptr(), count);
        }

        // Print output without decimal places
        println!("Input: {:?}", input.iter().map(|x| *x as i32).collect::<Vec<_>>());
        println!("Factor: {}", factor as i32);
        println!("Output: {:?}", output.iter().map(|x| *x as i32).collect::<Vec<_>>());
    });
}
