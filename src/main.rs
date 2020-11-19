use std::fs::{File, read_to_string};
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

/// Read a single puzzle from a file.
///
/// The file should have 7 characters. The first character is the center of the
/// Spelling Bee puzzle.
fn load_puzzle_from_file(path: &str) -> io::Result<Puzzle> {
    let raw = read_to_string(path)?;
    let count = raw.chars().count();
    assert_eq!(8, count);

    let chars: Vec<char> = raw.chars().collect();
    let p = Puzzle {
        center_letter: chars[0],
        outer_letters: [
            chars[1],
            chars[2],
            chars[3],
            chars[4],
            chars[5],
            chars[6],
        ],
    };
    Ok(p)
}

struct Puzzle {
    center_letter: char,
    outer_letters: [char; 6],
}

impl Puzzle {
    fn to_string(&self) -> String {
        let mut result = String::from(self.center_letter);
        result.push_str(": ");
        result.extend(self.outer_letters.iter());
        result
    }
}

fn main() {
    let dictionary = load_dictionary().unwrap();
    println!("There are {} words in the dictionary.", dictionary.len());

    let puzzle = load_puzzle_from_file("puzzle.txt").unwrap();
    println!("Puzzle: {}", puzzle.to_string());
}
