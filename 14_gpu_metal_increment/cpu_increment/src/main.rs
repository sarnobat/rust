use std::{fs, time::Instant};

fn main() {
    const COUNT: usize = 1_000_000;

    // --- read the shared Metal shader ---
    let shader_src = fs::read_to_string("incremental.metal")
        .expect("Failed to read incremental.metal");

    println!("Loaded Metal kernel from file.");

    // --- extract the arithmetic operation ---
    // look for something like "data[tid] += 3;" or "data[tid] *= 2;"
    let op_line = shader_src
        .lines()
        .find(|line| line.contains("data[tid]"))
        .unwrap_or("");
    println!("Detected operation: {}", op_line.trim());

    // very simple parse:
    // find the operator (+=, -=, *=, /=) and numeric constant
    let mut op = "+=";
    let mut value: i32 = 1;
    for candidate_op in ["+=", "-=", "*=", "/="] {
        if let Some(idx) = op_line.find(candidate_op) {
            op = candidate_op;
            // try to parse constant on right-hand side
            let rhs = &op_line[idx + candidate_op.len()..];
            if let Some(num) = rhs
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '-')
                .collect::<String>()
                .parse::<i32>()
                .ok()
            {
                value = num;
            }
            break;
        }
    }

    println!("Parsed operation: data[i] {} {}", op, value);

    // --- allocate and run identical computation on CPU ---
    let mut data = vec![1_i32; COUNT];
    let mut pass: u64 = 0;
    let mut total_ops: u128 = 0;
    let mut last_ops: u128 = 0;
    let mut last_report = Instant::now();

    println!("Running CPU simulation on {} elements", COUNT);

    loop {
        pass += 1;
        total_ops += COUNT as u128;

        match op {
            "+=" => data.iter_mut().for_each(|v| *v += value),
            "-=" => data.iter_mut().for_each(|v| *v -= value),
            "*=" => data.iter_mut().for_each(|v| *v *= value),
            "/=" => data.iter_mut().for_each(|v| *v /= value),
            _ => panic!("Unsupported operator {}", op),
        }

        if pass % 100 == 0 {
            let elapsed = last_report.elapsed().as_secs_f64();
            let ops_since = total_ops - last_ops;
            let throughput = (ops_since as f64) / (elapsed * 1e6);
            println!(
                "CPU pass {:>8} | total {:>15} ops | +{:>15} since last | {:>10.2} M ops/s | first element = {:>8}",
                pass, total_ops, ops_since, throughput, data[0]
            );
            last_ops = total_ops;
            last_report = Instant::now();
        }
    }
}
