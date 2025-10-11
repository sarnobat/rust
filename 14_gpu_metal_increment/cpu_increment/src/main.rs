use std::time::Instant;

fn main() {
    const COUNT: usize = 1_000_000;
    let mut data = vec![1_i32; COUNT];
    let mut pass: u64 = 0;
    let mut total_ops: u128 = 0;
    let mut last_ops: u128 = 0;
    let mut last_report = Instant::now();

    println!("Running CPU increment loop on {} elements", COUNT);

    loop {
        pass += 1;
        total_ops += COUNT as u128;

        for v in data.iter_mut() {
            *v += 1;
        }

        if pass % 100 == 0 {
            let elapsed = last_report.elapsed().as_secs_f64();
            let ops_since = total_ops - last_ops;
            let throughput = (ops_since as f64) / (elapsed * 1e6);

            println!(
                "CPU pass {:>8} | total {:>15} ops | +{:>15} since last | {:>10.2} M ops/s | first element = {:>8}",
                pass,
                total_ops,
                ops_since,
                throughput,
                data[0]
            );

            last_report = Instant::now();
            last_ops = total_ops;
        }
    }
}
