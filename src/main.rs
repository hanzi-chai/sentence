use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};

fn levenshtein_distance(s: &[char], t: &[char]) -> usize {
    let n = s.len();
    let m = t.len();
    let mut d = vec![vec![0; m + 1]; n + 1];
    for i in 1..=n {
        d[i][0] = i;
    }
    for j in 1..=m {
        d[0][j] = j;
    }
    for j in 1..=m {
        for i in 1..=n {
            let cost = if s[i - 1] == t[j - 1] { 0 } else { 1 };
            d[i][j] = *[d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + cost]
                .iter()
                .min()
                .unwrap();
        }
    }
    d[n][m]
}

#[derive(Parser)]
struct Args {
    result: String,
    reference: String,
}

fn main() -> Result<(), Error> {
    let Args { result, reference } = Args::parse();
    let result_reader = BufReader::new(File::open(result)?);
    let reference_reader = BufReader::new(File::open(reference)?);
    let mut total_length = 0;
    let mut total_error = 0;
    for (result_line, reference_line) in result_reader.lines().zip(reference_reader.lines()) {
        let result_line = result_line?;
        let reference_line = reference_line?;
        let result_chars: Vec<char> = result_line.chars().collect();
        let reference_chars: Vec<char> = reference_line.chars().collect();
        let distance = levenshtein_distance(&result_chars, &reference_chars);
        total_length += reference_chars.len();
        total_error += distance;
    }
    println!("Total length: {}", total_length);
    println!("Total error: {}", total_error);
    println!("Error rate: {:.2}%", total_error as f64 / total_length as f64 * 100.0);
    Ok(())
}
