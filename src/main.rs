use serde_json::{from_str, Value};
use std::io::{self, BufRead, Write};

use clap::Parser;

use rj::query_json;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    query: String,
}

fn main() {
    let args = Cli::parse();

    let mut stream_out = io::stdout().lock();

    let mut stream_in = io::stdin().lock().lines().map(|v| v.unwrap());
    let first_line = match stream_in.next() {
        Some(line) => line,
        None => "".to_string(),
    };
    match from_str(&first_line.as_str()) {
        Ok(v) => {
            apply(v, &mut stream_out, &args.query.as_str());
            for line in stream_in {
                match from_str(&line.as_str()) {
                    Ok(v) => {
                        apply(v, &mut stream_out, &args.query.as_str());
                    }
                    Err(e) => {
                        stream_out.write(format!("{e}").as_bytes()).unwrap();
                    }
                };
            }
        }
        Err(_) => {
            let mut lines = vec![first_line];
            lines.extend(stream_in);
            match from_str(&lines.join(&"").as_str()) {
                Ok(v) => {
                        apply(v, &mut stream_out, &args.query.as_str());
                },
                Err(e) => {
                    stream_out.write(format!("{e}").as_bytes()).unwrap();
                }
            };
        }
    };
}

fn apply(value: Value, stream_out: &mut io::StdoutLock, query: &str) {
    let results = query_json(value, query);
    for result in results {
        let output_bytes = serde_json::to_string(&result).unwrap();
        stream_out.write(output_bytes.as_bytes()).unwrap();
        stream_out.write("\n".as_bytes()).unwrap();
    }
}
