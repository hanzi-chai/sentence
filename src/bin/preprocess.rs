use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use zip::read::ZipArchive;

fn extract_chinese_sentences(file_path: &str, output_path: &str) -> io::Result<()> {
    // Open the ZIP file
    let file = File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;

    // Regular expression to match Chinese sentences
    let chinese_re = Regex::new(r"[\u4e00-\u9fff]+").unwrap();

    // Iterate over each file in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // Check if file is not a directory
        if file.is_file() && file.name().ends_with(".txt") && !file.name().ends_with("meta.txt") {
            // Read the content of the file
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // Find and collect all Chinese sentences
            for cap in chinese_re.captures_iter(&contents) {
                writeln!(output_file, "{}", cap[0].to_string())?;
            }
        }

        if i % 1000 == 0 {
            println!("Processed {} files", i);
        }
    }
    Ok(())
}

fn main() {
    let zip_file_path = "corpus/pixiv/PixivNovel.zip"; // Replace with the path to your ZIP file
    let output_path = "chinese_sentences.txt"; // Output file for Chinese sentences

    match extract_chinese_sentences(zip_file_path, output_path) {
        Ok(_) => {
            println!("Chinese sentences successfully saved to {}", output_path);
        }
        Err(e) => eprintln!("Error reading ZIP archive: {}", e),
    }
}
