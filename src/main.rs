use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io;
use std::io::BufRead;
use std::vec::Vec;

use chrono;

const WORDS_FILE_PATH: &str = "/usr/share/dict/words";

// Words must be at least 4 characters long to be valid answers.
const MIN_LENGTH: usize = 4;

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
        outer_letters: [chars[1], chars[2], chars[3], chars[4], chars[5], chars[6]],
    };
    Ok(p)
}

pub fn human_time(duration: chrono::Duration) -> String {
    if duration.num_seconds() > 0 {
        return format!(
            "{seconds}.{milliseconds:03}s",
            seconds = duration.num_seconds(),
            milliseconds = duration.num_milliseconds()
        );
    } else if duration.num_milliseconds() > 99 {
        return format!(
            "{milliseconds}ms",
            milliseconds = duration.num_milliseconds()
        );
    } else if duration.num_milliseconds() >= 1 {
        return format!(
            "{milliseconds}.{microseconds:03}ms",
            milliseconds = duration.num_milliseconds(),
            microseconds = duration.num_microseconds().expect("microsecond overflow"),
        );
    } else if duration.num_microseconds().expect("microsecond overflow") >= 1 {
        return format!(
            "{microseconds}μs",
            microseconds = duration.num_microseconds().expect("microsecond overflow"),
        );
    } else {
        return format!(
            "{nanoseconds}ns",
            nanoseconds = duration.num_nanoseconds().expect("nanosecond overflow"),
        );
    }
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

// Solvers //

trait Solver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String>;
}

/// NaiveSolver solves a puzzle by iterating over every word in a dictionary,
/// and seeing whether that word is valid.
struct NaiveSolver {
    words: Vec<String>,
}

impl NaiveSolver {
    fn new(word_list: Vec<String>) -> NaiveSolver {
        NaiveSolver {
            words: word_list
                .iter()
                .filter(|w| w.len() >= MIN_LENGTH)
                .cloned()
                .collect(),
        }
    }

    fn word_is_valid(&self, puzzle: &Puzzle, word: &str) -> bool {
        let mut has_center = false;
        for c in word.chars() {
            if c == puzzle.center_letter {
                has_center = true;
            } else {
                if !puzzle.outer_letters.contains(&c) {
                    return false;
                }
            }
        }
        return has_center;
    }
}

impl Solver for NaiveSolver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();

        for word in self.words.iter() {
            if word.len() < MIN_LENGTH {
                continue;
            }
            if self.word_is_valid(puzzle, word) {
                result.push(word.to_string())
            }
        }

        result
    }
}

struct RadixTrieSolver {
    dictionary: RadixTrieNode,
}
impl RadixTrieSolver {
    fn new(word_list: Vec<String>) -> RadixTrieSolver {
        let mut root = RadixTrieNode::new(false);
        for word in word_list.iter() {
            root.add(word.clone());
        }
        RadixTrieSolver { dictionary: root }
    }
}
impl Solver for RadixTrieSolver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String> {
        self.dictionary.find_words(puzzle, 0, "")
    }
}

struct RadixTrieNode {
    is_word: bool,
    children: HashMap<char, RadixTrieNode>,
}

impl RadixTrieNode {
    fn new(is_word: bool) -> RadixTrieNode {
        RadixTrieNode {
            is_word: is_word,
            children: HashMap::new(),
        }
    }

    fn add(&mut self, mut word: String) {
        let first_char = word.remove(0);

        // Already have a child there, so add remainder of word.
        if let Some(child) = self.children.get_mut(&first_char) {
            // No word remainder, so we should register child as a complete word.
            if word.len() == 0 {
                child.is_word = true;
            } else {
                child.add(word)
            }
        } else {
            // No child, so make a new one
            let mut child = RadixTrieNode::new(word.len() == 0);
            // More to go
            if word.len() > 0 {
                child.add(word);
            }
            self.children.insert(first_char, child);
        }
    }

    /// Finds all the words that match a puzzle for the given graph. Used
    /// recursively, with state captured in the 'path' and 'so_far' variables.
    ///
    /// graph should be a node to visit in the radix trie. puzzle should be the
    /// puzzle to be solved. path should be a String indicating the current set
    /// of letters visited, in order, including the current node's letter.
    fn find_words(&self, puzzle: &Puzzle, center_letter_count: u32, path: &str) -> Vec<String> {
        let mut result = Vec::new();

        if center_letter_count > 0 && self.is_word && path.len() >= MIN_LENGTH {
            result.push(path.to_string());
        }

        let mut subpath = path.to_string();
        if let Some(child) = self.children.get(&puzzle.center_letter) {
            subpath.push(puzzle.center_letter);
            let child_words = &mut child.find_words(puzzle, center_letter_count + 1, &subpath);
            result.append(child_words);
            subpath.pop();
        }
        for letter in puzzle.outer_letters.iter() {
            if let Some(child) = self.children.get(&letter) {
                subpath.push(*letter);
                let child_words = &mut child.find_words(puzzle, center_letter_count, &subpath);
                result.append(child_words);
                subpath.pop();
            }
        }

        result
    }
}

struct BitmapSolver {
    bitmasks: Vec<u32>,
    words: Vec<String>,
}

impl BitmapSolver {
    fn new(dictionary: Vec<String>) -> BitmapSolver {
        let mut bitmasks = vec![0; dictionary.len()];

        for (idx, word) in dictionary.iter().enumerate() {
            bitmasks[idx] = BitmapSolver::bitmask_word(word);
        }

        BitmapSolver {
            bitmasks: bitmasks,
            words: dictionary,
        }
    }

    fn bitmask_letter(letter: &char) -> u32 {
        match letter {
            'a' => 1 << 0,
            'b' => 1 << 1,
            'c' => 1 << 2,
            'd' => 1 << 3,
            'e' => 1 << 4,
            'f' => 1 << 5,
            'g' => 1 << 6,
            'h' => 1 << 7,
            'i' => 1 << 8,
            'j' => 1 << 9,
            'k' => 1 << 10,
            'l' => 1 << 11,
            'm' => 1 << 12,
            'n' => 1 << 13,
            'o' => 1 << 14,
            'p' => 1 << 15,
            'q' => 1 << 16,
            'r' => 1 << 17,
            's' => 1 << 18,
            't' => 1 << 19,
            'u' => 1 << 20,
            'v' => 1 << 21,
            'w' => 1 << 22,
            'x' => 1 << 23,
            'y' => 1 << 24,
            'z' => 1 << 25,
            _ => 1 << 26,
        }
    }

    fn bitmask_word(word: &str) -> u32 {
        let mut chars: Vec<char> = word.chars().collect();
        chars.sort();
        chars.dedup();
        let mut mask: u32 = 0;
        for c in chars.iter() {
            mask |= BitmapSolver::bitmask_letter(c);
        }
        mask
    }
}

impl Solver for BitmapSolver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String> {
        let center_letter_mask = BitmapSolver::bitmask_letter(&puzzle.center_letter);

        // forbidden_letter_mask has 1 for every letter which must *not* be
        // used. We compute it by ORing together all the allowed words, and then
        // inverting.
        let mut forbidden_letter_mask: u32 = center_letter_mask;
        for letter in puzzle.outer_letters.iter() {
            forbidden_letter_mask |= BitmapSolver::bitmask_letter(letter)
        }
        forbidden_letter_mask = !forbidden_letter_mask;

        let mut result: Vec<String> = Vec::new();
        for (idx, mask) in self.bitmasks.iter().enumerate() {
            if (mask & center_letter_mask != 0) && (mask & forbidden_letter_mask == 0) {
                if self.words[idx].len() >= MIN_LENGTH {
                    result.push(self.words[idx].to_string());
                }
            }
        }

        result
    }
}

fn benchmark_solver(label: &str, solver: &impl Solver, puzzle: &Puzzle) {
    println!("running {} solver", label);
    let start = chrono::offset::Utc::now();
    let valid = solver.solve(puzzle);
    let end = chrono::offset::Utc::now();

    println!("computed {} answers", valid.len());
    println!("runtime: {}", human_time(end - start));
}

fn main() {
    let dictionary = load_dictionary().unwrap();
    println!("building native");
    let naive = NaiveSolver::new(dictionary.clone());
    println!("building radix");
    let trie = RadixTrieSolver::new(dictionary.clone());
    println!("building bitmask");
    let bitmask = BitmapSolver::new(dictionary.clone());

    let puzzle = load_puzzle_from_file("puzzle.txt").unwrap();
    println!("Puzzle: {}", puzzle.to_string());

    for _ in 1..100 {
        benchmark_solver("naive", &naive, &puzzle);
        benchmark_solver("trie", &trie, &puzzle);
        benchmark_solver("bitmask", &bitmask, &puzzle);
    }
}
