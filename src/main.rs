use rand::{seq::IteratorRandom, thread_rng};
use serde_json::Value;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use std::{collections::HashMap, fs::read_to_string, io::Error};
use std::{fs, process::Command};

const RIME_FOLDER: &str = "/Users/tansongchen/Public/librime/build/bin";

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
    let data = read_to_string("assets/essay.txt")?;
    let mut frequency = HashMap::new();
    for line in data.lines() {
        let (word, value) = line.split_once("\t").unwrap();
        let f: u64 = value.parse().unwrap();
        for char in word.chars() {
            // if the character is not in the map, insert it with a value of f; otherwise, add f to the existing value.
            *frequency.entry(char).or_insert(0) += f;
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
type Lookup = HashMap<char, String>;

#[derive(PartialEq)]
enum Mode {
    PatternFree,
    ErMading,
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
                Mode::ErMading => random_em(&mut rng),
            };
            (c, s)
        })
        .collect();
    return arrange_short_code(&full, frequency);
}

fn write_rime_dictionary(table: &Table, name: &str) -> Result<(), Error> {
    let schema = read_to_string("schema.yaml")?;
    let schema_path = format!("build/{}.schema.yaml", name);
    let mut schema_file = fs::File::create(schema_path)?;
    let current_schema = schema.replace("luna_pinyin", name);
    write!(schema_file, "{}", current_schema)?;
    let dict_path = format!("build/{}.dict.yaml", name);
    let mut file = fs::File::create(dict_path)?;
    writeln!(file, "---")?;
    writeln!(file, "name: {}", name)?;
    writeln!(file, "version: \"0.1\"")?;
    writeln!(file, "sort: by_weight")?;
    writeln!(file, "use_preset_vocabulary: true")?;
    writeln!(file, "...\n")?;
    for (c, s) in table {
        writeln!(file, "{}\t{}", c, s)?;
    }
    Ok(())
}

fn initialize_rime() -> Result<(), Error> {
    Command::new("bash")
        .arg("-c")
        .arg(format!("cp build/* {}", RIME_FOLDER))
        .output()?;
    let rime_deployer = Path::new(RIME_FOLDER).join("rime_deployer");
    Command::new(rime_deployer)
        .arg("--build")
        .current_dir(RIME_FOLDER)
        .output()?;
    Ok(())
}

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

fn run_sentence(table: &Table) -> Result<f64, Error> {
    let mut total_length = 0;
    let mut total_error_length = 0;
    let lookup: Lookup = table.iter().cloned().collect();
    let rime_api_console = Path::new(RIME_FOLDER).join("rime_api_console");
    let mut child = Command::new(rime_api_console)
        .current_dir(RIME_FOLDER)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    reader.read_line(&mut buffer)?;
    let setences_file = File::open("assets/sentences.txt")?;
    let mut file_reader = BufReader::new(setences_file);
    let start = Instant::now();
    let total_lines = 10000;
    for i in 0..total_lines {
        if i % 1000 == 0 {
            println!("Processed {} sentences", i);
        }
        let mut sentence = String::new();
        file_reader.read_line(&mut sentence)?;
        let sentence = sentence.trim();
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
        test_code.push_str(" \n");
        stdin.write_all(test_code.as_bytes())?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        // let string_result = line[8..].trim();
        let result: Vec<_> = line[8..].trim().chars().collect();
        let expected: Vec<_> = sentence.chars().collect();
        let distance = levenshtein_distance(&result, &expected);
        total_length += expected.len();
        total_error_length += distance;
        // println!("{}, {}, {}", sentence, string_result, levenshtein_distance(&result, &expected));
        for _ in 0..3 {
            reader.read_line(&mut line)?;
        }
        line.clear();
    }
    let _duration = start.elapsed();
    let error_rate = total_error_length as f64 / total_length as f64;
    Ok(error_rate)
}

fn dump_schema(table: &Table) -> Result<(), Error> {
    write_rime_dictionary(&table, "test").unwrap();
    let default_path = "build/default.custom.yaml";
    let mut default_file = fs::File::create(default_path).unwrap();
    writeln!(default_file, "patch:")?;
    writeln!(default_file, "  schema_list:")?;
    writeln!(default_file, "    - schema: test")?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let gb2312 = get_gb2312()?;
    let frequency = get_frequency()?;
    fs::copy("assets/essay.txt", "build/essay.txt")?;
    let mode = Mode::ErMading;
    let mut error_rates = Vec::new();
    for trial in 0..10 {
        println!("Trial {}", trial);
        let table = generate_tables(&gb2312, &frequency, &mode);
        dump_schema(&table)?;
        initialize_rime()?;
        let error_rate = run_sentence(&table)?;
        error_rates.push(error_rate);
    }
    let mean_error_rate: f64 = error_rates.iter().sum::<f64>() / error_rates.len() as f64;
    let std_dev: f64 = error_rates
        .iter()
        .map(|x| (x - mean_error_rate).powi(2))
        .sum::<f64>()
        .sqrt();
    println!("Mean error rate: {}", mean_error_rate);
    println!("Standard deviation: {}", std_dev);
    Ok(())
}
