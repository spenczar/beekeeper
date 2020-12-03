use std::collections::HashMap;
use std::vec::Vec;

// Words must be at least 4 characters long to be valid answers.
const MIN_LENGTH: usize = 4;

// Initialize any result vectors with this capacity
const TYPICAL_RESULT_SIZE: usize = 100;


pub struct Puzzle {
    pub center_letter: char,
    pub outer_letters: [char; 6],
}

impl Puzzle {
    pub fn to_string(&self) -> String {
        let mut result = String::from(self.center_letter);
        result.push_str(": ");
        result.extend(self.outer_letters.iter());
        result
    }
}

// Solvers //

pub trait Solver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String>;
}

/// NaiveSolver solves a puzzle by iterating over every word in a dictionary,
/// and seeing whether that word is valid.
pub struct NaiveSolver {
    words: Vec<String>,
}

impl NaiveSolver {
    pub fn new(word_list: Vec<String>) -> NaiveSolver {
        NaiveSolver {
            words: word_list
                .iter()
                .filter(|w| w.len() >= MIN_LENGTH)
                .cloned()
                .collect(),
        }
    }

    pub fn word_is_valid(&self, puzzle: &Puzzle, word: &str) -> bool {
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
        let mut result: Vec<String> = Vec::with_capacity(TYPICAL_RESULT_SIZE);

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

pub struct TrieSolver {
    dictionary: TrieNode,
}

impl TrieSolver {
    pub fn new(word_list: Vec<String>) -> TrieSolver {
        let mut root = TrieNode::new(false);
        for word in word_list.iter().filter(|w| w.len() >= MIN_LENGTH) {
            root.add(word.clone());
        }
        TrieSolver { dictionary: root }
    }
}

impl Solver for TrieSolver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String> {
        self.dictionary.find_words(puzzle, 0, "")
    }
}

struct TrieNode {
    is_word: bool,
    children: HashMap<char, TrieNode>,
}

impl TrieNode {
    fn new(is_word: bool) -> TrieNode {
        TrieNode {
            is_word: is_word,
            children: HashMap::with_capacity(26),
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
            let mut child = TrieNode::new(word.len() == 0);
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
        let mut result = Vec::with_capacity(TYPICAL_RESULT_SIZE);

        if center_letter_count > 0 && self.is_word {
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

pub struct BitmaskSolver {
    bitmasks: Vec<BitmaskedWord>,
}

struct BitmaskedWord {
    mask: u32,
    word: String,
}

impl BitmaskSolver {
    pub fn new(dictionary: Vec<String>) -> BitmaskSolver {
        let mut bitmasks = Vec::with_capacity(dictionary.len());

        for word in dictionary.iter() {
            if word.len() >= MIN_LENGTH {
                bitmasks.push(BitmaskedWord {
                    mask: BitmaskSolver::bitmask_word(word),
                    word: word.to_string(),
                });
            }
        }

        BitmaskSolver { bitmasks: bitmasks }
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
            mask |= BitmaskSolver::bitmask_letter(c);
        }
        mask
    }
}

impl Solver for BitmaskSolver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String> {
        let center_letter_mask = BitmaskSolver::bitmask_letter(&puzzle.center_letter);

        // forbidden_letter_mask has 1 for every letter which must *not* be
        // used. We compute it by ORing together all the allowed words, and then
        // inverting.
        let mut forbidden_letter_mask: u32 = center_letter_mask;
        for letter in puzzle.outer_letters.iter() {
            forbidden_letter_mask |= BitmaskSolver::bitmask_letter(letter)
        }
        forbidden_letter_mask = !forbidden_letter_mask;

        let mut result: Vec<String> = Vec::with_capacity(TYPICAL_RESULT_SIZE);
        for mask in self.bitmasks.iter() {
            if (mask.mask & center_letter_mask != 0) && (mask.mask & forbidden_letter_mask == 0) {
                result.push(mask.word.to_string());
            }
        }

        result
    }
}

/*
BlockBitmaskSolver is like BitmaskSolver, but with one layer of hierarchy: words
in the dictionary are lexigraphically sorted and then split into "blocks" of
fixed size. Each block has a shared block-level pair of bitmasks, indicating all
characters that are shared by all words, and all characters that are present in
at least one of the words.

These block bitmasks can be consulted quickly to skip over large tranches of
words. For example, if every word in a block has an "f" character, but "f" is
not in the puzzle, then we can skip that block - none of its words will be
valid. Likewise, if the center letter of a puzzle is "q", but none of the words
in the block have a "q", then we can skip it.

It's not immediately clear what the block size should be, so it is left
configurable for now while I do some experimentation.
*/
pub struct BitmaskBlockSolver {
    blocks: Vec<BitmaskBlock>,
}

impl BitmaskBlockSolver {
    pub fn new(dictionary: Vec<String>, chunk_size: usize) -> BitmaskBlockSolver {
        let mut blocks = Vec::with_capacity(dictionary.len() / chunk_size + 1);
        let mut sorted: Vec<String> = dictionary
            .iter()
            .filter(|w| w.len() >= MIN_LENGTH)
            .cloned()
            .collect();
        sorted.sort();
        for chunk in sorted.chunks(chunk_size) {
            blocks.push(BitmaskBlock::new(chunk));
        }
        BitmaskBlockSolver { blocks: blocks }
    }
}

impl Solver for BitmaskBlockSolver {
    fn solve(&self, puzzle: &Puzzle) -> Vec<String> {
        let center_letter_mask = BitmaskSolver::bitmask_letter(&puzzle.center_letter);

        // forbidden_letter_mask has 1 for every letter which must *not* be
        // used. We compute it by ORing together all the allowed words, and then
        // inverting.
        let mut forbidden_letter_mask: u32 = center_letter_mask;
        for letter in puzzle.outer_letters.iter() {
            forbidden_letter_mask |= BitmaskSolver::bitmask_letter(letter)
        }
        forbidden_letter_mask = !forbidden_letter_mask;

        let mut result: Vec<String> = Vec::with_capacity(TYPICAL_RESULT_SIZE);

        for block in self.blocks.iter() {
            if let Some(matches) = &mut block.matches(center_letter_mask, forbidden_letter_mask) {
                result.append(matches);
            }
        }
        result
    }
}

struct BitmaskBlock {
    // Mask encoding the characters present in all words in the block.
    common_chars_mask: u32,
    // Mask encoding the characters present in no words in the block.
    missing_chars_mask: u32,
    // The words present in the block.
    words: Vec<BitmaskedWord>,
}

impl BitmaskBlock {
    fn new(words: &[String]) -> BitmaskBlock {
        let mut common_chars_mask: u32 = !0;
        let mut missing_chars_mask: u32 = 0;
        let mut masked_words = Vec::with_capacity(words.len());

        for w in words.iter() {
            let masked_word = BitmaskedWord {
                mask: BitmaskSolver::bitmask_word(&w),
                word: w.to_string(),
            };
            missing_chars_mask |= masked_word.mask;
            common_chars_mask &= masked_word.mask;
            masked_words.push(masked_word);
        }

        BitmaskBlock {
            common_chars_mask: common_chars_mask,
            missing_chars_mask: missing_chars_mask,
            words: masked_words,
        }
    }

    /// Returns the list of all words that match, if there are any matches. If
    /// there aren't any, then returns None.
    fn matches(&self, center_letter_mask: u32, forbidden_letter_mask: u32) -> Option<Vec<String>> {
        if (self.common_chars_mask & forbidden_letter_mask) != 0 {
            return None;
        }
        if (self.missing_chars_mask & center_letter_mask) == 0 {
            return None;
        }
        let mut result: Vec<String> = Vec::with_capacity(self.words.len());
        for w in self.words.iter() {
            if (w.mask & center_letter_mask != 0) && (w.mask & forbidden_letter_mask == 0) {
                result.push(w.word.to_string());
            }
        }
        if result.len() == 0 {
            return None;
        } else {
            return Some(result);
        }
    }
}
