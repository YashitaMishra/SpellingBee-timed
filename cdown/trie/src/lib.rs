//This is the trie function that I originally submitted, it is needed for spelling
//bee to run.

use std::collections::HashMap;
use std::fs;

/// Constants for the random word generator.
const MULTIPLIER: u64 = 6364136223846793005;
const INCREMENT: u64 = 1442695040888963407;

// Node structure: its children keyed by letter, and whether a word ends here.
// The HashMap keeps the children on the heap and only stores letters
// that are actually used.
struct Node {
    children: HashMap<char, Node>,
    is_word: bool,
}

pub struct Trie {
    root: Node,
    count: usize,
}

fn new_node() -> Node {
    Node {
        children: HashMap::new(),
        is_word: false,
    }
}

// Creates a new, empty trie
pub fn new_trie() -> Trie {
    Trie {
        root: new_node(),
        count: 0,
    }
}

// Number of words in the trie
pub fn size(trie: &Trie) -> usize {
    trie.count
}

// Turns a word into uppercase
pub fn to_upper(word: &str) -> String {
    word.to_uppercase()
}

// Trims and uppercases a word. Returns None if it is empty or has any
// character other than A-Z.
fn clean(word: &str) -> Option<String> {
    let upper = to_upper(word.trim());
    if upper.is_empty() {
        return None;
    }
    for c in upper.chars() {
        if !c.is_ascii_uppercase() {
            return None;
        }
    }
    Some(upper)
}

// Adds one word. Returns true if it was new, false if it was already there
// or invalid.
pub fn insert(trie: &mut Trie, word: &str) -> bool {
    let word = match clean(word) {
        Some(w) => w,
        None => return false,
    };
    let mut node = &mut trie.root;
    for c in word.chars() {
        node = node.children.entry(c).or_insert(new_node());
    }
    if node.is_word {
        return false;
    }
    node.is_word = true;
    trie.count += 1;
    true
}

// Returns true if the word is in the trie.
pub fn contains(trie: &Trie, word: &str) -> bool {
    let word = match clean(word) {
        Some(w) => w,
        None => return false,
    };
    let mut node = &trie.root;
    for c in word.chars() {
        match node.children.get(&c) {
            Some(next) => node = next,
            None => return false,
        }
    }
    node.is_word
}

// Deletes one word. Returns true if it was there.
pub fn delete(trie: &mut Trie, word: &str) -> bool {
    let word = match clean(word) {
        Some(w) => w,
        None => return false,
    };
    let letters: Vec<char> = word.chars().collect();
    let removed = delete_from(&mut trie.root, &letters, 0);
    if removed {
        trie.count -= 1;
    }
    removed
}

// Walks down to the end of the word, unmarks it, and on the way back up
// removes nodes that no longer lead to any word.
fn delete_from(node: &mut Node, letters: &[char], i: usize) -> bool {
    if i == letters.len() {
        if !node.is_word {
            return false;
        }
        node.is_word = false;
        return true;
    }
    let c = letters[i];
    let removed = match node.children.get_mut(&c) {
        Some(child) => delete_from(child, letters, i + 1),
        None => return false,
    };
    if removed {
        let child = &node.children[&c];
        if !child.is_word && child.children.is_empty() {
            node.children.remove(&c);
        }
    }
    removed
}

// Deletes every word in the list. Returns how many were removed.
pub fn delete_list(trie: &mut Trie, words: &[String]) -> usize {
    let mut removed = 0;
    for word in words {
        if delete(trie, word) {
            removed += 1;
        }
    }
    removed
}

// Adds every word in the list. Returns how many were new.
pub fn add_all(trie: &mut Trie, words: &[String]) -> usize {
    let mut added = 0;
    for word in words {
        if insert(trie, word) {
            added += 1;
        }
    }
    added
}

// Reads a file with one word per line.
pub fn read_lines(path: &str) -> Vec<String> {
    let text = fs::read_to_string(path).expect("could not read file");
    let mut words = Vec::new();
    for line in text.lines() {
        let word = line.trim();
        if !word.is_empty() {
            words.push(word.to_string());
        }
    }
    words
}

// Reads a file of words separated by commas and/or whitespace
pub fn read_comma_space(path: &str) -> Vec<String> {
    let text = fs::read_to_string(path).expect("could not read file");
    let mut words = Vec::new();
    for word in text.replace(',', " ").split_whitespace() {
        words.push(word.to_string());
    }
    words
}

// Builds a trie from a file with one word per line
pub fn build_from_lines(path: &str) -> Trie {
    let mut trie = new_trie();
    add_all(&mut trie, &read_lines(path));
    trie
}

//= Builds a trie from a comma/space separated file
pub fn build_from_comma_space(path: &str) -> Trie {
    let mut trie = new_trie();
    add_all(&mut trie, &read_comma_space(path));
    trie
}

// Returns the words in the file that are in the trie, in uppercase.
pub fn found_in_file(trie: &Trie, path: &str) -> Vec<String> {
    let mut found = Vec::new();
    for word in read_comma_space(path) {
        if contains(trie, &word) {
            found.push(to_upper(&word));
        }
    }
    found
}

// Returns the words in the file that are not in the trie, in uppercase.
pub fn not_found_in_file(trie: &Trie, path: &str) -> Vec<String> {
    let mut missing = Vec::new();
    for word in read_comma_space(path) {
        if !contains(trie, &word) {
            missing.push(to_upper(&word));
        }
    }
    missing
}

// Returns a random lowercase word of length min_len..=max_len .
pub fn random_word(seed: &mut u64, min_len: usize, max_len: usize) -> String {
    let len = min_len + next_random(seed) % (max_len - min_len + 1);
    let mut word = String::new();
    for _ in 0..len {
        let letter = (b'a' + (next_random(seed) % 26) as u8) as char;
        word.push(letter);
    }
    word
}

fn next_random(seed: &mut u64) -> usize {
    *seed = seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    (*seed >> 33) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str, text: &str) -> String {
        let path = std::env::temp_dir().join(name);
        fs::write(&path, text).unwrap();
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn new_trie_is_empty() {
        let trie = new_trie();
        assert_eq!(size(&trie), 0);
        assert!(!contains(&trie, "A"));
    }

    #[test]
    fn to_upper_works() {
        assert_eq!(to_upper("apple"), "APPLE");
        assert_eq!(to_upper("MiXeD"), "MIXED");
    }

    #[test]
    fn insert_and_find() {
        let mut trie = new_trie();
        assert!(insert(&mut trie, "car"));
        assert!(!insert(&mut trie, "CAR"), "duplicate");
        assert!(!insert(&mut trie, "c4r"), "invalid");
        assert!(!insert(&mut trie, ""), "empty");
        assert!(insert(&mut trie, "cart"));
        assert!(contains(&trie, "Car"));
        assert!(contains(&trie, "CART"));
        assert!(!contains(&trie, "CA"), "prefix is not a word");
        assert!(!contains(&trie, "CARTS"));
        assert_eq!(size(&trie), 2);
    }

    #[test]
    fn delete_keeps_other_words() {
        let mut trie = new_trie();
        insert(&mut trie, "CAR");
        insert(&mut trie, "CART");
        insert(&mut trie, "CAT");
        assert!(delete(&mut trie, "car"));
        assert!(!delete(&mut trie, "car"), "already deleted");
        assert!(!delete(&mut trie, "CA"), "never a word");
        assert!(!contains(&trie, "CAR"));
        assert!(contains(&trie, "CART"));
        assert!(contains(&trie, "CAT"));
        assert!(delete(&mut trie, "CART"));
        assert!(contains(&trie, "CAT"));
        assert_eq!(size(&trie), 1);
    }

    #[test]
    fn delete_last_word_prunes_to_root() {
        let mut trie = new_trie();
        insert(&mut trie, "HELLO");
        assert!(delete(&mut trie, "HELLO"));
        assert!(trie.root.children.is_empty());
    }

    #[test]
    fn delete_a_list() {
        let mut trie = new_trie();
        let words = vec!["ONE".to_string(), "TWO".to_string(), "THREE".to_string()];
        add_all(&mut trie, &words);
        let to_delete = vec!["one".to_string(), "four".to_string(), "two".to_string()];
        assert_eq!(delete_list(&mut trie, &to_delete), 2);
        assert!(contains(&trie, "THREE"));
        assert_eq!(size(&trie), 1);
    }

    #[test]
    fn both_builders_agree() {
        let lines = temp_file(
            "trie_lines.txt",
            "apple\nBanana\n\n  cherry \napple\nco-op\n",
        );
        let commas = temp_file("trie_commas.txt", "apple, Banana,,cherry\napple co-op");
        let a = build_from_lines(&lines);
        let b = build_from_comma_space(&commas);
        assert_eq!(size(&a), 3);
        assert_eq!(size(&b), 3);
        for word in ["APPLE", "BANANA", "CHERRY"] {
            assert!(contains(&a, word) && contains(&b, word));
        }
    }

    #[test]
    fn found_and_not_found_in_file() {
        let mut trie = new_trie();
        add_all(&mut trie, &["SUN".to_string(), "MOON".to_string()]);
        let path = temp_file("trie_query.txt", "sun\nstar, moon planet");
        assert_eq!(found_in_file(&trie, &path), vec!["SUN", "MOON"]);
        assert_eq!(not_found_in_file(&trie, &path), vec!["STAR", "PLANET"]);
    }

    #[test]
    fn random_words_are_valid_and_repeatable() {
        let mut seed1 = 57;
        let mut seed2 = 57;
        for _ in 0..1000 {
            let word = random_word(&mut seed1, 2, 5);
            assert!(word.len() >= 2 && word.len() <= 5);
            for c in word.chars() {
                assert!(c.is_ascii_lowercase());
            }
            assert_eq!(word, random_word(&mut seed2, 2, 5));
        }
    }

    /// Random insert/find/delete checked against a HashMap .
    #[test]
    fn random_operations_match_hashmap() {
        for start in [1, 57, 2102] {
            let mut seed = start;
            let mut trie = new_trie();
            let mut expected: HashMap<String, bool> = HashMap::new();
            for _ in 0..3000 {
                let word = to_upper(&random_word(&mut seed, 1, 4));
                let is_new = !expected.contains_key(&word);
                assert_eq!(insert(&mut trie, &word), is_new);
                expected.insert(word, true);
            }
            assert_eq!(size(&trie), expected.len());
            for _ in 0..3000 {
                let guess = random_word(&mut seed, 1, 4);
                assert_eq!(
                    contains(&trie, &guess),
                    expected.contains_key(&to_upper(&guess))
                );
            }
            for _ in 0..3000 {
                let word = to_upper(&random_word(&mut seed, 1, 4));
                assert_eq!(delete(&mut trie, &word), expected.remove(&word).is_some());
            }
            assert_eq!(size(&trie), expected.len());
        }
    }
}
