use std::env;
use std::thread;

fn main() {
    // Parse factor from first argument, default to 3
    let args: Vec<String> = env::args().collect();
    let factor: i32 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    // Input array
    let input = vec![0, 1, 1, 0, 1, 0, 1, 0];
    let mut output = vec![0; input.len()];

    // Simple parallelism: spawn threads
    let num_threads = 4;
    let chunk_size = (input.len() + num_threads - 1) / num_threads;
    let mut handles = Vec::new();

    for chunk in input.chunks(chunk_size) {
        // clone the chunk so it is 'static for the thread
        let chunk_owned = chunk.to_vec();
        let factor = factor;
        let handle = thread::spawn(move || {
            chunk_owned.iter().map(|&x| x * factor).collect::<Vec<i32>>()
        });
        handles.push(handle);
    }

    // Collect results
    let mut idx = 0;
    for h in handles {
        let chunk_res = h.join().unwrap();
        for v in chunk_res {
            output[idx] = v;
            idx += 1;
        }
    }

    // Print output
    println!("Input: {:?}", input);
    println!("Factor: {}", factor);
    println!("Output: {:?}", output);
}
