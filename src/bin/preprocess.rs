use regex::Regex;
use std::fs::{self, File};
use std::io::{Error, Read};
use zip::read::ZipArchive;

fn main() -> Result<(), Error> {
    let mut archive = ZipArchive::new(File::open("corpus/pixiv/PixivNovel.zip")?)?;
    let chinese_re = Regex::new(r"[\u4e00-\u9fff]+").unwrap();
    let mut processed_files = 0;
    let mut buffer = Vec::new();
    let mut all_files = Vec::new();
    println!("Processing files...");
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if !file.is_file() || !file.name().ends_with(".txt") || file.name().ends_with("meta.txt") {
            continue;
        }
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        for cap in chinese_re.captures_iter(&contents) {
            buffer.push(cap[0].to_string());
        }
        processed_files += 1;
        if processed_files % 1000 == 0 {
            println!("Processed {} files", processed_files);
            let name = format!("assets/corpus-{}.txt", processed_files / 1000);
            fs::write(&name, buffer.join("\n"))?;
            buffer.clear();
            all_files.push(name);
        }
    }
    fs::write("assets/index.txt", all_files.join("\n"))?;
    Ok(())
}
