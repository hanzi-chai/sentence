use clap::Parser;
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    pub table: String,
    pub input: String,
    pub filtered: String,
    pub output: String,
}

fn main() -> Result<(), Error> {
    let Args {
        table,
        input,
        filtered,
        output,
    } = Args::parse();
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(table)?;
    let lookup: Result<HashMap<char, String>, _> = reader.deserialize().collect();
    let lookup = lookup?;
    let file_reader = BufReader::new(File::open(input)?);
    let mut filtered_writer = File::create(filtered)?;
    let mut file_writer = File::create(output)?;
    let mut successful = 0;
    let max_samples = 10000;
    for line in file_reader.lines() {
        let sentence = line?;
        let chars: Vec<char> = sentence.chars().collect();
        if chars.len() > 20 {
            continue;
        }
        let mut test_code = String::with_capacity(120);
        let mut success = true;
        for c in sentence.chars() {
            let code = lookup.get(&c);
            if code.is_none() {
                success = false;
                break;
            } else {
                test_code.push_str(code.unwrap());
            }
            if test_code.len() > 99 {
                success = false;
                break;
            }
        }
        if !success {
            continue;
        }
        successful += 1;
        writeln!(file_writer, "{} ", test_code)?;
        writeln!(filtered_writer, "{}", sentence)?;
        if successful == max_samples {
            break;
        }
    }
    Ok(())
}
