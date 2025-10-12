use std::env;

fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as u32) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

fn primes_up_to(limit: u32) -> Vec<u32> {
    (2..limit).filter(|&n| is_prime(n)).collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let limit = if args.len() > 1 {
        args[1].parse::<u32>().unwrap_or_else(|_| {
            eprintln!("Invalid number: {}", args[1]);
            std::process::exit(1);
        })
    } else {
        50
    };

    for p in primes_up_to(limit) {
        println!("{}", p);
    }
}
