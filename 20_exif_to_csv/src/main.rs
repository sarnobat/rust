use serde::Deserialize;
use std::env;
use std::io::{self, Read};

#[derive(Deserialize, Debug)]
struct Record {
    GPSLatitude: Option<f64>,
    GPSLongitude: Option<f64>,
    FileModifyDate: Option<String>,
}

fn main() {
    // Check for CLI flag
    let include_nulls = env::args().any(|arg| arg == "--include-nulls");

    // Read all stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    // Split stream of JSON objects by "}\n{" (or similar)
    let json_objects = input
        .split("\n{")
        .map(|chunk| {
            let trimmed = chunk.trim();
            if trimmed.is_empty() {
                return None;
            }
            let mut s = trimmed.to_string();
            if !s.starts_with('{') {
                s.insert(0, '{');
            }
            if !s.ends_with('}') {
                s.push('}');
            }
            Some(s)
        })
        .flatten();

    println!("x,y,z");

    for obj in json_objects {
        match serde_json::from_str::<Record>(&obj) {
            Ok(rec) => {
                // Skip rows with any nulls unless flag is given
                if !include_nulls
                    && (rec.GPSLatitude.is_none()
                        || rec.GPSLongitude.is_none()
                        || rec.FileModifyDate.is_none())
                {
                    continue;
                }

                let lat = rec.GPSLatitude.map_or(String::new(), |v| v.to_string());
                let lon = rec.GPSLongitude.map_or(String::new(), |v| v.to_string());
                let date = rec.FileModifyDate.unwrap_or_default();

                println!("{lat},{lon},{date}");
            }
            Err(e) => eprintln!("Warning: failed to parse object: {e}"),
        }
    }
}
