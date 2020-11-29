use std::fs::{read_to_string, File};
use std::io;
use std::io::BufRead;

use beekeeper::{Puzzle, Solver, NaiveSolver, BitmaskSolver, BitmaskBlockSolver, RadixTrieSolver};

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
        outer_letters: [chars[1], chars[2], chars[3], chars[4], chars[5], chars[6]],
    };
    Ok(p)
}

fn main() {
    let dictionary = load_dictionary().unwrap();
    println!("building native");
    let naive = NaiveSolver::new(dictionary.clone());
    println!("building radix");
    let trie = RadixTrieSolver::new(dictionary.clone());
    println!("building bitmask");
    let bitmask = BitmaskSolver::new(dictionary.clone());
    println!("building blockwise bitmask (50-size blocks)");
    let bitmask_block = BitmaskBlockSolver::new(dictionary.clone(), 50);

    let puzzle = load_puzzle_from_file("puzzle.txt").unwrap();
    println!("Puzzle: {}", puzzle.to_string());

    benchmark_solver("naive", &naive, &puzzle);
    benchmark_solver("trie", &trie, &puzzle);
    benchmark_solver("bitmask", &bitmask, &puzzle);
    benchmark_solver("bitmask-block", &bitmask_block, &puzzle);
}

fn benchmark_solver(label: &str, solver: &impl Solver, puzzle: &Puzzle) {
    println!("running {} solver", label);
    let start = chrono::offset::Utc::now();
    let valid = solver.solve(puzzle);
    let end = chrono::offset::Utc::now();

    println!("computed {} answers", valid.len());
    println!("runtime: {}", end - start);
}

#[test]
fn test_trie_solver() {
    let dictionary = load_dictionary().unwrap();
    let naive = NaiveSolver::new(dictionary.clone());
    let trie = RadixTrieSolver::new(dictionary.clone());
    let puzzle = Puzzle {
        center_letter: 'a',
        outer_letters: ['b', 'c', 'd', 'e', 'f', 'g'],
    };

    let mut naive_solution = naive.solve(&puzzle);
    let mut trie_solution = trie.solve(&puzzle);

    naive_solution.sort();
    trie_solution.sort();
    assert!(naive_solution == trie_solution);
}

#[test]
fn test_bitmask_solver() {
    let dictionary = load_dictionary().unwrap();
    let naive = NaiveSolver::new(dictionary.clone());
    let bitmask = BitmaskSolver::new(dictionary.clone());
    let puzzle = Puzzle {
        center_letter: 'a',
        outer_letters: ['b', 'c', 'd', 'e', 'f', 'g'],
    };

    let mut naive_solution = naive.solve(&puzzle);
    let mut bitmask_solution = bitmask.solve(&puzzle);

    naive_solution.sort();
    bitmask_solution.sort();
    assert!(naive_solution == bitmask_solution);
}

#[test]
fn test_blockbitmask_solver() {
    let dictionary = load_dictionary().unwrap();
    let naive = NaiveSolver::new(dictionary.clone());
    let block_bitmask = BitmaskBlockSolver::new(dictionary.clone(), 50);
    let puzzle = Puzzle {
        center_letter: 'a',
        outer_letters: ['b', 'c', 'd', 'e', 'f', 'g'],
    };

    let mut naive_solution = naive.solve(&puzzle);
    let mut block_bitmask_solution = block_bitmask.solve(&puzzle);

    naive_solution.sort();
    block_bitmask_solution.sort();
    assert!(naive_solution == block_bitmask_solution);
}
