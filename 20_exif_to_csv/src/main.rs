use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize, Debug)]
struct Record {
    GPSLatitude: Option<f64>,
    GPSLongitude: Option<f64>,
    FileModifyDate: Option<String>,
}

fn main() {
    // Read entire stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    // Split by '}\n{' so that we can handle multiple JSON objects in a stream
    let json_objects = input
        .split("\n{")
        .map(|chunk| {
            if chunk.trim().is_empty() {
                return None;
            }
            let normalized = if !chunk.trim_start().starts_with('{') {
                format!("{{{}", chunk)
            } else {
                chunk.to_string()
            };
            let normalized = if !normalized.trim_end().ends_with('}') {
                format!("{}{}", normalized, "}")
            } else {
                normalized
            };
            Some(normalized)
        })
        .flatten();

    // Print CSV header
    println!("x,y,z");

    for obj in json_objects {
        match serde_json::from_str::<Record>(&obj) {
            Ok(rec) => {
                let lat = rec.GPSLatitude.map_or(String::new(), |v| v.to_string());
                let lon = rec.GPSLongitude.map_or(String::new(), |v| v.to_string());
                let date = rec.FileModifyDate.unwrap_or_default();
                println!("{lat},{lon},{date}");
            }
            Err(e) => eprintln!("Warning: failed to parse object: {e}"),
        }
    }
}
