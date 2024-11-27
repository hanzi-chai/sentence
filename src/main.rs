use clap::Parser;
use csv::ReaderBuilder;
use sentence::Statistics;
use serde_json::{self, to_string};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};

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
    table: String,
    result: String,
    reference: String,
    statistics: String,
}

fn main() -> Result<(), Error> {
    let Args {
        table,
        result,
        reference,
        statistics,
    } = Args::parse();
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(table)?;
    let lookup: Result<HashMap<char, String>, _> = reader.deserialize().collect();
    let lookup = lookup?;
    let result_reader = BufReader::new(File::open(result)?);
    let reference_reader = BufReader::new(File::open(reference)?);
    let mut data: Vec<_> = (0..30)
        .into_iter()
        .map(|x| {
            let mut s = Statistics::default();
            s.length = x;
            s
        })
        .collect();
    let get_len = |c: &char| lookup.get(&c).unwrap().len();
    for (result_line, reference_line) in result_reader.lines().zip(reference_reader.lines()) {
        let result_line = result_line?;
        let reference_line = reference_line?;
        let result_chars: Vec<char> = result_line.chars().collect();
        let reference_chars: Vec<char> = reference_line.chars().collect();
        let result_seg: Vec<usize> = result_chars.iter().map(get_len).collect();
        let reference_seg: Vec<usize> = reference_chars.iter().map(get_len).collect();
        let segmentation_correct = result_seg.len() == reference_seg.len()
            && result_seg
                .iter()
                .zip(reference_seg.iter())
                .all(|(a, b)| a == b);
        let distance = levenshtein_distance(&result_chars, &reference_chars);
        for s in data.iter_mut() {
            if s.length == 0 || s.length == reference_chars.len() {
                s.sentences += 1;
                s.characters += reference_chars.len();
                s.distances += distance;
                if distance == 0 {
                    s.successes += 1;
                } else if segmentation_correct {
                    s.translation_errors += 1;
                } else {
                    s.segmentation_errors += 1;
                }
            }
        }
    }
    File::create(statistics)?.write_all(to_string(&data)?.as_bytes())?;
    Ok(())
}
