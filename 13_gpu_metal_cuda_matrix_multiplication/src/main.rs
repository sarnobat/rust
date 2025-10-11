use metal::*;

fn print_matrix(name: &str, data: &[f32], rows: usize, cols: usize) {
    println!("{name} =");
    for i in 0..rows {
        for j in 0..cols {
            print!("{:6.2} ", data[i * cols + j]);
        }
        println!();
    }
    println!();
}

fn main() {
    // ------------------------------------------------------------
    // 1. Initialize Metal device and command queue
    // ------------------------------------------------------------
    let device = Device::system_default().expect("No Metal device found");
    println!("Using GPU: {}", device.name());
    let queue = device.new_command_queue();

    // ------------------------------------------------------------
    // 2. Inline Metal kernel source
    // ------------------------------------------------------------
    let shader_src = r#"
        #include <metal_stdlib>
        using namespace metal;

        kernel void matmul(
            device const float* A [[buffer(0)]],
            device const float* B [[buffer(1)]],
            device float*       C [[buffer(2)]],
            constant uint& M     [[buffer(3)]],
            constant uint& N     [[buffer(4)]],
            constant uint& K     [[buffer(5)]],
            uint2 gid            [[thread_position_in_grid]]
        ) {
            if (gid.x >= N || gid.y >= M) return;
            float sum = 0.0;
            for (uint k = 0; k < K; ++k)
                sum += A[gid.y * K + k] * B[k * N + gid.x];
            C[gid.y * N + gid.x] = sum;
        }
    "#;

    // ------------------------------------------------------------
    // 3. Compile kernel and create pipeline
    // ------------------------------------------------------------
    let options = CompileOptions::new();
    let lib = device
        .new_library_with_source(shader_src, &options)
        .expect("Failed to compile Metal shader");
    let func = lib.get_function("matmul", None).unwrap();
    let pso = device
        .new_compute_pipeline_state_with_function(&func)
        .expect("Failed to create pipeline");

    // ------------------------------------------------------------
    // 4. Define matrices
    // ------------------------------------------------------------
    const M: u32 = 3;
    const K: u32 = 2;
    const N: u32 = 3;

    let a: [f32; (M * K) as usize] = [1., 2., 3., 4., 5., 6.];
    let b: [f32; (K * N) as usize] = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    let mut c: [f32; (M * N) as usize] = [0.0; (M * N) as usize];

    print_matrix("A", &a, M as usize, K as usize);
    print_matrix("B", &b, K as usize, N as usize);

    // ------------------------------------------------------------
    // 5. Create buffers
    // ------------------------------------------------------------
    let buf_a = device.new_buffer_with_data(
        a.as_ptr() as *const _,
        std::mem::size_of_val(&a) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    let buf_b = device.new_buffer_with_data(
        b.as_ptr() as *const _,
        std::mem::size_of_val(&b) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    let buf_c = device.new_buffer(
        std::mem::size_of_val(&c) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    let buf_m = device.new_buffer_with_data(
        &M as *const _ as *const _,
        std::mem::size_of_val(&M) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    let buf_n = device.new_buffer_with_data(
        &N as *const _ as *const _,
        std::mem::size_of_val(&N) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    let buf_k = device.new_buffer_with_data(
        &K as *const _ as *const _,
        std::mem::size_of_val(&K) as u64,
        MTLResourceOptions::StorageModeShared,
    );

    // ------------------------------------------------------------
    // 6. Encode commands
    // ------------------------------------------------------------
    let cmd_buf = queue.new_command_buffer();
    let enc = cmd_buf.new_compute_command_encoder();
    enc.set_compute_pipeline_state(&pso);
    enc.set_buffer(0, Some(&buf_a), 0);
    enc.set_buffer(1, Some(&buf_b), 0);
    enc.set_buffer(2, Some(&buf_c), 0);
    enc.set_buffer(3, Some(&buf_m), 0);
    enc.set_buffer(4, Some(&buf_n), 0);
    enc.set_buffer(5, Some(&buf_k), 0);

    enc.dispatch_threads(
        MTLSize {
            width: N as u64,
            height: M as u64,
            depth: 1,
        },
        MTLSize {
            width: 1,
            height: 1,
            depth: 1,
        },
    );
    enc.end_encoding();
    cmd_buf.commit();
    cmd_buf.wait_until_completed();

    // ------------------------------------------------------------
    // 7. Read back results
    // ------------------------------------------------------------
    unsafe {
        std::ptr::copy_nonoverlapping(
            buf_c.contents() as *const f32,
            c.as_mut_ptr(),
            c.len(),
        );
    }

    print_matrix("C = A×B", &c, M as usize, N as usize);
    println!("=== Done ===");
}
