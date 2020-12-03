use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

use std::fs::File;
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

pub fn benchmark_solvers(c: &mut Criterion) {
    let dictionary = load_dictionary().unwrap();
    let naive = NaiveSolver::new(dictionary.clone());
    let trie = RadixTrieSolver::new(dictionary.clone());
    let bitmask = BitmaskSolver::new(dictionary.clone());
    let bitmask_block = BitmaskBlockSolver::new(dictionary.clone(), 50);
    let mut group = c.benchmark_group("Bee solvers");

    let puzzle = Puzzle{
        center_letter: 'e',
        outer_letters: ['x', 'p', 'u', 'n', 'i', 'g'],
    };

    group.bench_function("naive", |b| b.iter(|| naive.solve(&puzzle)));
    group.bench_function("trie", |b| b.iter(|| trie.solve(&puzzle)));
    group.bench_function("bitmask", |b| b.iter(|| bitmask.solve(&puzzle)));
    group.bench_function("bitmask block", |b| b.iter(|| bitmask_block.solve(&puzzle)));

    group.finish();
}

pub fn benchmark_block_size(c: &mut Criterion) {
    let dictionary = load_dictionary().unwrap();
    let mut group = c.benchmark_group("Block Solvers");

    let puzzle = Puzzle{
        center_letter: 'e',
        outer_letters: ['x', 'p', 'u', 'n', 'i', 'g'],
    };

    for size in [1, 2, 5, 7, 9, 10, 12, 14, 16, 18, 20, 30, 40, 50, 60, 75, 82, 100, 200, 500, 1000].iter() {
        let bitmask_block = BitmaskBlockSolver::new(dictionary.clone(), *size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| bitmask_block.solve(&puzzle));
        });
    }
}

criterion_group!(benches, benchmark_block_size);
criterion_main!(benches);
