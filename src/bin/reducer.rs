use clap::Parser;
use csv::{Reader, ReaderBuilder};
use sentence::Statistics;
use std::{fs::read_to_string, io::Error};

#[derive(Parser)]
struct Args {
    pub results: String,
    pub output: String,
}

fn main() -> Result<(), Error> {
    let Args { results, output } = Args::parse();
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(results)?;
    let list_of_results: Result<Vec<String>, _> = reader.deserialize().collect();
    let list_of_results = list_of_results?;
    println!("{:?}", list_of_results);
    let mut data: Vec<_> = (0..30)
        .into_iter()
        .map(|x| {
            let mut s = Statistics::default();
            s.length = x;
            s
        })
        .collect();
    for result in list_of_results {
        let path = format!("assets/result-{}.json", result);
        let json = read_to_string(path)?;
        let result: Vec<Statistics> = serde_json::from_str(&json)?;
        for (i, s) in result.iter().enumerate() {
            data[i].sentences += s.sentences;
            data[i].characters += s.characters;
            data[i].distances += s.distances;
            data[i].successes += s.successes;
            data[i].translation_errors += s.translation_errors;
            data[i].segmentation_errors += s.segmentation_errors;
        }
    }
    let total = serde_json::to_string(&data)?;
    std::fs::write(output, total)?;
    Ok(())
}
