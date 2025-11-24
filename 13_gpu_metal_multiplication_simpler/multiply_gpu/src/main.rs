use metal::*;
use objc::rc::autoreleasepool;

fn main() {
    autoreleasepool(|| {
        let device = Device::system_default().unwrap();
        let command_queue = device.new_command_queue();

        let input = vec![0f32, 1., 1., 0., 1., 0., 1., 0.];
        let count = input.len();

        let buffer = device.new_buffer(
            (count * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        );

        unsafe {
            std::ptr::copy_nonoverlapping(input.as_ptr(), buffer.contents() as *mut f32, count);
        }

        let kernel_src = r#"
        kernel void multiply(device float* data [[buffer(0)]], uint id [[thread_position_in_grid]]) {
            if (id < 1000) data[id] *= 3.0;
        }"#;

        let library = device
            .new_library_with_source(kernel_src, &CompileOptions::new())
            .unwrap();
        let function = library.get_function("multiply", None).unwrap();

        // <-- Updated for metal-rs 0.32+
        let descriptor = ComputePipelineDescriptor::new();
        descriptor.set_compute_function(Some(&function));
        let pipeline = device.new_compute_pipeline_state(&descriptor).unwrap();

        let command_buffer = command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_buffer(0, Some(&buffer), 0);

        let threads = MTLSize {
            width: count as u64,
            height: 1,
            depth: 1,
        };
        let threads_per_group = MTLSize {
            width: 1,
            height: 1,
            depth: 1,
        };

        encoder.dispatch_threads(threads, threads_per_group);
        encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();

        let mut output = vec![0f32; count];
        unsafe {
            std::ptr::copy_nonoverlapping(buffer.contents() as *const f32, output.as_mut_ptr(), count);
        }

        println!("Input: {:?}", input);
        println!("Output: {:?}", output);
    });
}
