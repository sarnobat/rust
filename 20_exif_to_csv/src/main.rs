use serde::Deserialize;
use std::env;
use std::io::{self, BufRead};
use chrono::{DateTime, FixedOffset, Datelike};

#[derive(Deserialize, Debug)]
struct Record {
    GPSLatitude: Option<f64>,
    GPSLongitude: Option<f64>,
    FileModifyDate: Option<String>,
}

fn normalize(v: f64, old_min: f64, old_max: f64, new_min: f64, new_max: f64) -> f64 {
    if old_max == old_min {
        return new_min;
    }
    (v - old_min) / (old_max - old_min) * (new_max - new_min) + new_min
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let include_nulls = args.iter().any(|a| a == "--include-nulls");
    let raw = args.iter().any(|a| a == "--raw");

    println!("x,y,z");

    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut inside = false;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }

        // Start of a JSON object
        if line.trim_start().starts_with('{') {
            inside = true;
            buffer.clear();
        }

        if inside {
            buffer.push_str(&line);
            buffer.push('\n');
        }

        // End of a JSON object
        if line.trim_end().ends_with('}') && inside {
            inside = false;
            if let Ok(rec) = serde_json::from_str::<Record>(&buffer) {
                if !include_nulls
                    && (rec.GPSLatitude.is_none()
                        || rec.GPSLongitude.is_none()
                        || rec.FileModifyDate.is_none())
                {
                    continue;
                }

                if let (Some(lat), Some(lon), Some(date_str)) =
                    (rec.GPSLatitude, rec.GPSLongitude, rec.FileModifyDate)
                {
                    if raw {
                        println!("{lat},{lon},{date_str}");
                        continue;
                    }

                    if let Ok(dt) = DateTime::parse_from_str(&date_str, "%Y:%m:%d %H:%M:%S%:z") {
                        let year = dt.year() as f64;
                        let x = normalize(lon, -180.0, 180.0, -10.0, 10.0);
                        let y = normalize(lat, -90.0, 90.0, -10.0, 10.0);
                        let z = normalize(year, 1950.0, 2050.0, -10.0, 10.0);
                        println!("{x:.3},{y:.3},{z:.3}");
                    }
                }
            }
        }
    }
}
