use std::{fs, time::{Instant, Duration}};
use evalexpr::*;
use regex::Regex;

fn main() {
    const COUNT: usize = 1_000_000;
    const RUN_FOR: Duration = Duration::from_secs(10);  // run ~10 seconds

    // --- load shared Metal shader ---
    let shader_src = fs::read_to_string("incremental.metal")
        .expect("Failed to read incremental.metal");

    // --- extract expression inside "data[tid] = ... ;" ---
    let expr_raw = shader_src
        .lines()
        .find_map(|line| line.split_once("data[tid] ="))
        .map(|(_, rhs)| rhs.trim().trim_end_matches(';'))
        .expect("Could not parse assignment expression in shader");

    println!("Loaded shader expression (Metal): {}", expr_raw);

    // --- normalize Metal -> evalexpr syntax ---
    let mut expr = expr_raw.replace("data[tid]", "x");

    // Convert bitwise ops (^, >>, <<) to function calls
    let re_xor = Regex::new(r"\(([^()]+)\)\s*\^\s*\(([^()]+)\)").unwrap();
    expr = re_xor.replace_all(&expr, "xor(\\1,\\2)").to_string();

    let re_shr = Regex::new(r"(\w+)\s*>>\s*(\d+)").unwrap();
    expr = re_shr.replace_all(&expr, "shr(\\1,\\2)").to_string();

    let re_shl = Regex::new(r"(\w+)\s*<<\s*(\d+)").unwrap();
    expr = re_shl.replace_all(&expr, "shl(\\1,\\2)").to_string();

    println!("Normalized expression for evalexpr: {}", expr);

    // --- setup eval context ---
    let mut ctx = HashMapContext::new();

    ctx.set_function(
        "shr".into(),
        Function::new(Box::new(|arg: &Value| {
            let tuple = arg.as_tuple()?;
            let x = tuple[0].as_int()?;
            let n = tuple[1].as_int()?;
            Ok(Value::Int(x.wrapping_shr(n as u32)))
        })),
    ).unwrap();

    ctx.set_function(
        "shl".into(),
        Function::new(Box::new(|arg: &Value| {
            let tuple = arg.as_tuple()?;
            let x = tuple[0].as_int()?;
            let n = tuple[1].as_int()?;
            Ok(Value::Int(x.wrapping_shl(n as u32)))
        })),
    ).unwrap();

    ctx.set_function(
        "xor".into(),
        Function::new(Box::new(|arg: &Value| {
            let tuple = arg.as_tuple()?;
            let a = tuple[0].as_int()?;
            let b = tuple[1].as_int()?;
            Ok(Value::Int(a ^ b))
        })),
    ).unwrap();

    // --- compile expression ---
    let parsed = build_operator_tree(&expr).expect("Failed to parse expression");

    // --- simulation loop ---
    let mut data = vec![1_i64; COUNT];
    let mut pass: u64 = 0;
    let mut total_ops: u128 = 0;
    let mut last_ops: u128 = 0;
    let mut last_report = Instant::now();
    let start_time = Instant::now();

    println!("Running CPU interpreter on {} elements", COUNT);

    loop {
        pass += 1;
        total_ops += COUNT as u128;

        // execute kernel on CPU
        for v in data.iter_mut() {
            ctx.set_value("x".into(), Value::Int(*v)).unwrap();
            if let Ok(Value::Int(out)) = parsed.eval_with_context(&ctx) {
                *v = out;
            }
        }

        // print every ~0.25 seconds
        if last_report.elapsed().as_secs_f64() > 0.25 {
            let elapsed = last_report.elapsed().as_secs_f64();
            let ops_since = total_ops - last_ops;
            let throughput = (ops_since as f64) / (elapsed * 1e6);
            println!(
                "{:<7} pass {:>8} | total {:>15} ops | +{:>15} since last | {:>10.2} M ops/s | first element = {:>8}",
                "CPU",
                pass,
                total_ops,
                ops_since,
                throughput,
                data[0]
            );
            last_report = Instant::now();
            last_ops = total_ops;
        }

        if start_time.elapsed() >= RUN_FOR {
            println!("\nReached 10-second limit, exiting.");
            break;
        }
    }
}
