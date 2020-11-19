use std::fs::File;
use std::io;
use std::io::BufRead;
use std::vec::Vec;

const WORDS_FILE_PATH: &str = "/usr/share/dict/words";

/// Load all the words from the unix dictionary.
fn load_dictionary() -> io::Result<Vec<String>> {
    let file = File::open(WORDS_FILE_PATH)?;
    let reader = io::BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(lines)
}

fn main() {
    let dictionary = load_dictionary().unwrap();
    println!("There are {} words in the dictionary.", dictionary.len());
}
