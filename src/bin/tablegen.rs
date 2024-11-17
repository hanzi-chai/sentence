use clap::{Parser, ValueEnum};
use csv::{ReaderBuilder, WriterBuilder};
use rand::{seq::IteratorRandom, thread_rng};
use serde_json::Value;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::{collections::HashMap, fs::read_to_string, io::Error};

fn get_gb2312() -> Result<Vec<char>, Error> {
    let data = read_to_string("assets/repertoire.json")?;
    let value: Value = serde_json::from_str(&data)?;
    let characters = value.as_array().unwrap();
    let gb2312: Vec<char> = characters
        .iter()
        .filter_map(|v| {
            if v["gb2312"].as_bool().unwrap() {
                let unicode = v["unicode"].as_i64().unwrap();
                // Convert the Unicode code point to a character.
                let c = std::char::from_u32(unicode as u32).unwrap();
                Some(c)
            } else {
                None
            }
        })
        .collect();
    Ok(gb2312)
}

fn get_frequency() -> Result<HashMap<char, u64>, Error> {
    let mut essay_file = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path("assets/essay.txt")?;
    let essay: Result<Vec<(String, u64)>, _> = essay_file.deserialize().collect();
    let mut frequency = HashMap::new();
    for (word, value) in essay? {
        for char in word.chars() {
            *frequency.entry(char).or_insert(0) += value;
        }
    }
    Ok(frequency)
}

type Frequency = HashMap<char, u64>;

fn arrange_short_code(full: &Table, frequency: &Frequency) -> Table {
    let mut short = full.clone();
    let mut known_short_codes = HashSet::new();
    short.sort_by_key(|(c, _)| Reverse(frequency.get(c).unwrap_or(&0)));
    for (_, s) in short.iter_mut() {
        let prefix: String = s.chars().take(2).collect();
        if !known_short_codes.contains(&prefix) {
            s.pop();
            known_short_codes.insert(prefix);
        }
    }
    short
}

type Table = Vec<(char, String)>;

#[derive(PartialEq, Debug, Clone, ValueEnum)]
enum Mode {
    PatternFree,
    Ermading,
}

fn generate_tables(gb2312: &Vec<char>, frequency: &Frequency, mode: &Mode) -> Table {
    let alphabet = "abcdefghijklmnopqrstuvwxyz";
    let large_set = "ijklmnopqrstuvwxyz";
    let small_set = "abcdefgh";
    let mut rng = thread_rng();
    let random_pf = |rng: &mut rand::rngs::ThreadRng| {
        let mut full = String::new();
        for _ in 0..3 {
            let char = alphabet.chars().choose(rng).unwrap();
            full.push(char);
        }
        full
    };
    let random_em = |rng: &mut rand::rngs::ThreadRng| {
        let char1 = large_set.chars().choose(rng).unwrap();
        let char2 = alphabet.chars().choose(rng).unwrap();
        let char3 = small_set.chars().choose(rng).unwrap();
        format!("{}{}{}", char1, char2, char3)
    };
    let full: Vec<(char, String)> = gb2312
        .iter()
        .map(|&c| {
            let s = match mode {
                Mode::PatternFree => random_pf(&mut rng),
                Mode::Ermading => random_em(&mut rng),
            };
            (c, s)
        })
        .collect();
    return arrange_short_code(&full, frequency);
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(value_enum)]
    pub mode: Mode,
    pub output: String,
}

fn main() -> Result<(), Error> {
    let Args { mode, output } = Args::parse();
    let gb2312 = get_gb2312()?;
    let frequency = get_frequency()?;
    let table = generate_tables(&gb2312, &frequency, &mode);
    let mut writer = WriterBuilder::new().delimiter(b'\t').from_path(output)?;
    for (c, s) in table {
        writer.write_record(&[c.to_string(), s])?;
    }
    Ok(())
}
