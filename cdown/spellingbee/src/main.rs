//used the format_time and tenths_left functions that were public as the timer counts down from three
//minutes. It shows the time left after guesses and hints and shows the milliseconds as well
use cdown::{format_time, tenths_left};
use std::time::Instant;
use rand::Rng;
use std::io;
use trie::{Trie, add_all, contains, new_trie, read_lines, to_upper};
const DICTIONARY: &str = "../dictionary.txt";
const COLUMNS: usize = 3;
const COLUMN_WIDTH: usize = 18;
const LETTER_COUNT: usize = 7;
const MIN_LEN: usize = 4;
const PANGRAM_BONUS: usize = 7;
const NO_REPEAT_BONUS: usize = 2;
const TIME_LIMIT: u32 = 180; 
fn main() {
    let list = read_lines(DICTIONARY);
    let mut dict = new_trie();
    add_all(&mut dict, &list);
    let (letters, answers) = new_puzzle(&list);
    let hints = make_hints(&answers, &letters);
 
    println!("SPELLING BEE");
    println!("Make words of {MIN_LEN}+ letters from the letters below. Letters can repeat.");
    println!("Every word must use the main letter, {}, shown first.", letters[0]);
    println!("Type a word, h for hints, or q to quit.");
 
    let mut found: Vec<String> = Vec::new();
    let mut total = 0;
    let mut line = String::new();
    let start = Instant::now(); // the clock starts when the game starts
    loop {
        let left = tenths_left(TIME_LIMIT, start.elapsed());
        println!("\nTime left: {}", format_time(left));
        println!("Found {} of {} words, score {total}", found.len(), answers.len());
        print_columns(&found);
        println!("\n{}", show_letters(&letters));
        line.clear();
        let bytes = io::stdin().read_line(&mut line).expect("could not read input");
        let guess = to_upper(line.trim());
        if bytes == 0 || guess == "Q" {
            break;
        }
        if tenths_left(TIME_LIMIT, start.elapsed()) == 0 {
            println!("Time's up! That last word came too late."); 
            break;
        }
        if guess == "H" {
            print_hints(&hints, answers.len());
            continue;
        }
        match check_guess(&guess, &letters, &dict, &found) {
            Guess::Scored(points) => {
                if is_pangram(&guess, &letters) {
                    println!("PANGRAM! {guess} uses all seven letters.");
                }
                println!("{guess}: +{points}");
                total += points;
                found.push(guess);
                found.sort();
                if found.len() == answers.len() {
                    println!("\nQUEEN BEE! You found all {} words. Score: {total}", answers.len());
                    return;
                }
            }
            Guess::BadLetter(c) => println!("'{c}' is not one of the seven letters."),
            Guess::TooShort => println!("Too short: words need at least {MIN_LEN} letters."),
            Guess::NoMainLetter => println!("Missing the main letter {}.", letters[0]),
            Guess::AlreadyFound => println!("You already found {guess}."),
            Guess::NotAWord => println!("{guess} is not in the dictionary."),
        }
    }
    println!("\nFinal score: {total}. Words you missed:");
    let mut missed = Vec::new();
    for word in answers {
        if !found.contains(&word) {
            missed.push(word);
        }
    }
    print_columns(&missed);
}
 
// Draws letters until the answers include at least one pangram
fn new_puzzle(list: &[String]) -> (Vec<char>, Vec<String>) {
    loop {
        let letters = pick_letters();
        let answers = find_answers(list, &letters);
        for word in &answers {
            if is_pangram(word, &letters) {
                return (letters, answers);
            }
        }
    }
}

fn print_columns(words: &[String]) {
    for (i, word) in words.iter().enumerate() {
        if (i + 1) % COLUMNS == 0 || i + 1 == words.len() {
            println!("{word}");
        } else {
            print!("{word:<COLUMN_WIDTH$}");
        }
    }
}
 
/// Why a guess was accepted or rejected. A rejected guess is discarded.
enum Guess {
    Scored(usize),
    BadLetter(char),
    TooShort,
    NoMainLetter,
    AlreadyFound,
    NotAWord,
}
 
// Picks 7 distinct letters with rand and the first letter picked is the main
// letter. A `while` loop is used because a repeated letter is thrown away
// and drawn again, so it can take more than seven draws.
fn pick_letters() -> Vec<char> {
    let mut rng = rand::rng();
    let mut letters = Vec::new();
    while letters.len() < LETTER_COUNT {
        let letter = (b'A' + rng.random_range(0..26)) as char;
        if !letters.contains(&letter) {
            letters.push(letter);
        }
    }
    letters
}
 
// The letters are displayed with spaces between them and the main letter first, e.g. "C A I M O R T"
fn show_letters(letters: &[char]) -> String {
    let mut text = String::new();
    for c in letters {
        text.push(*c);
        text.push(' ');
    }
    text.trim_end().to_string()
}
 
// Returns the first character of `word` that is not one of the letters.
fn bad_letter(word: &str, letters: &[char]) -> Option<char> {
    for c in word.chars() {
        if !letters.contains(&c) {
            return Some(c);
        }
    }
    None
}
 
// All dictionary words (sorted) that are long enough, which use only the seven
// letters and contain the main letter 
fn find_answers(dictionary: &[String], letters: &[char]) -> Vec<String> {
    // the subset of words that use only the seven letters.
    let mut subset = Vec::new();
    for word in dictionary {
        if word.len() >= MIN_LEN && bad_letter(word, letters).is_none() {
            subset.push(word.clone());
        }
    }
    // remove the words without the main letter
    let mut answers = Vec::new();
    for word in subset {
        if word.contains(letters[0]) {
            answers.push(word);
        }
    }
    answers.sort();
    answers
}
 
// A pangram uses all seven letters.
fn is_pangram(word: &str, letters: &[char]) -> bool {
    for c in letters {
        if !word.contains(*c) {
            return false;
        }
    }
    true
}
 
// True if no letter appears twice. `seen` is a fixed array of 26 flags,
// one per letter, so the check is a single pass.
fn no_repeats(word: &str) -> bool {
    let mut seen = [false; 26];
    for c in word.chars() {
        let i = (c as u8 - b'A') as usize;
        if seen[i] {
            return false;
        }
        seen[i] = true;
    }
    true
}
 
// 1 point for a 4-letter word, one more for each extra letter, plus the
// pangram and no-repeat bonuses.
fn score(word: &str, letters: &[char]) -> usize {
    let mut points = word.len() - (MIN_LEN - 1);
    if is_pangram(word, letters) {
        points += PANGRAM_BONUS;
    }
    if no_repeats(word) {
        points += NO_REPEAT_BONUS;
    }
    points
}
 
// Checks a guess's letters, length, main letter, already
// found, and finally whether the trie holds the word
fn check_guess(guess: &str, letters: &[char], dict: &Trie, found: &[String]) -> Guess {
    let word = to_upper(guess);
    if let Some(c) = bad_letter(&word, letters) {
        return Guess::BadLetter(c);
    }
    if word.len() < MIN_LEN {
        return Guess::TooShort;
    }
    if !word.contains(letters[0]) {
        return Guess::NoMainLetter;
    }
    if found.contains(&word) {
        return Guess::AlreadyFound;
    }
    if !contains(dict, &word) {
        return Guess::NotAWord;
    }
    Guess::Scored(score(&word, letters))
}
 
// Hints
 
struct Hints {
    // grid[letter][length] = number of answers starting with that letter
    // (0 = A ... 25 = Z) with that length. A fixed 26-row table indexed by
    // letter 
    grid: Vec<Vec<usize>>,
    // (two-letter start, count), in alphabetical order.
    pairs: Vec<(String, usize)>,
    points: usize,
    pangrams: usize,
}
 
// Builds the hints from the sorted list of answers.
fn make_hints(answers: &[String], letters: &[char]) -> Hints {
    let mut longest = MIN_LEN;
    for word in answers {
        if word.len() > longest {
            longest = word.len();
        }
    }
    let mut hints = Hints {
        grid: vec![vec![0; longest + 1]; 26],
        pairs: Vec::new(),
        points: 0,
        pangrams: 0,
    };
    for (i, word) in answers.iter().enumerate() {
        let row = (word.as_bytes()[0] - b'A') as usize;
        hints.grid[row][word.len()] += 1;
        hints.points += score(word, letters);
        if is_pangram(word, letters) {
            hints.pangrams += 1;
        }
        if i > 0 && answers[i - 1][..2] == word[..2] {
            let last = hints.pairs.len() - 1;
            hints.pairs[last].1 += 1;
        } else {
            hints.pairs.push((word[..2].to_string(), 1));
        }
    }
    hints
}
 
fn cell(n: usize) -> String {
    if n == 0 { "-".to_string() } else { n.to_string() }
}
 
// Prints the grid and the two-letter list
fn print_hints(hints: &Hints, total_words: usize) {
    let longest = hints.grid[0].len() - 1;
    println!(
        "WORDS: {total_words}, POINTS: {}, PANGRAMS: {}\n",
        hints.points, hints.pangrams
    );
    print!("    ");
    for len in MIN_LEN..=longest {
        print!("{len:>4}");
    }
    println!("{:>5}", "Σ");
    let mut column_totals = vec![0; longest + 1];
    for (row, counts) in hints.grid.iter().enumerate() {
        let mut row_total = 0;
        for n in counts {
            row_total += n;
        }
        if row_total == 0 {
            continue;
        }
        print!("{}:  ", (b'A' + row as u8) as char);
        for len in MIN_LEN..=longest {
            print!("{:>4}", cell(counts[len]));
            column_totals[len] += counts[len];
        }
        println!("{row_total:>5}");
    }
    print!("Σ:  ");
    for len in MIN_LEN..=longest {
        print!("{:>4}", cell(column_totals[len]));
    }
    println!("{total_words:>5}\n\nTwo letter list:");
    // One line per first letter, e.g. "CA-9 CI-4 CO-12".
    let mut line = String::new();
    for (start, count) in &hints.pairs {
        if !line.is_empty() && !line.starts_with(&start[..1]) {
            println!("{}", line.trim_end());
            line.clear();
        }
        line.push_str(&format!("{start}-{count} "));
    }
    println!("{}", line.trim_end());
}
 
#[cfg(test)]
mod tests {
    use super::*;
    fn letters() -> Vec<char>{
        vec!['C', 'A', 'I', 'M', 'O', 'R', 'T']
    }
 
    fn words(list: &[&str]) -> Vec<String> {
        let mut out = Vec::new();
        for w in list {
            out.push(w.to_string());
        }
        out
    }
 
    #[test]
    fn letters_are_seven_distinct_capitals() {
        for _ in 0..1000 {
            let picked = pick_letters();
            assert_eq!(picked.len(), LETTER_COUNT);
            for (i, c) in picked.iter().enumerate() {
                assert!(c.is_ascii_uppercase());
                assert!(!picked[i + 1..].contains(c), "repeated letter");
            }
        }
    }
 
    #[test]
    fn shows_letters_with_spaces() {
        assert_eq!(show_letters(&letters()), "C A I M O R T");
    }
 
    #[test]
    fn answers_use_only_the_letters_and_the_main_letter() {
        let dict = words(&["COMA", "ATOM", "MOAT", "COMIC", "CAT", "COMB", "AROMATIC"]);
        // ATOM, MOAT: no C. CAT: too short. COMB: B is not allowed.
        assert_eq!(find_answers(&dict, &letters()), words(&["AROMATIC", "COMA", "COMIC"]));
    }
 
    #[test]
    fn scores_by_length_with_bonuses() {
        let l = letters();
        assert_eq!(score("COMA", &l), 1 + NO_REPEAT_BONUS);
        assert_eq!(score("COMMA", &l), 2);
        assert_eq!(score("AROMATIC", &l), 5 + PANGRAM_BONUS);
        assert_eq!(score("MOTRICA", &l), 4 + PANGRAM_BONUS + NO_REPEAT_BONUS);
    }
 
    #[test]
    fn checks_each_kind_of_guess() {
        let l = letters();
        let mut dict = new_trie();
        add_all(&mut dict, &words(&["COMA", "COMMA", "ATOM"]));
        let found = words(&["COMMA"]);
        assert!(matches!(check_guess("coma", &l, &dict, &found), Guess::Scored(3)));
        assert!(matches!(check_guess("cobs", &l, &dict, &found), Guess::BadLetter('B')));
        assert!(matches!(check_guess("co a", &l, &dict, &found), Guess::BadLetter(' ')));
        assert!(matches!(check_guess("cat", &l, &dict, &found), Guess::TooShort));
        assert!(matches!(check_guess("atom", &l, &dict, &found), Guess::NoMainLetter));
        assert!(matches!(check_guess("Comma", &l, &dict, &found), Guess::AlreadyFound));
        assert!(matches!(check_guess("cami", &l, &dict, &found), Guess::NotAWord));
    }
 
    // Random puzzles on the  dictionary
    #[test]
    fn random_puzzles_agree_with_check_guess() {
        let list = trie::read_lines("../dictionary.txt");
        let mut dict = new_trie();
        add_all(&mut dict, &list);
        for _ in 0..20 {
            let l = pick_letters();
            let answers = find_answers(&list, &l);
            for word in &answers {
                assert!(matches!(check_guess(word, &l, &dict, &[]), Guess::Scored(_)));
            }
            let mut scored = 0;
            for word in &list {
                if let Guess::Scored(_) = check_guess(word, &l, &dict, &[]) {
                    scored += 1;
                }
            }
            assert_eq!(scored, answers.len());
        }
    }
 
    #[test]
    fn counts_match_the_answers() {
        let letters = vec!['C', 'A', 'I', 'M', 'O', 'R', 'T'];
        let mut answers = Vec::new();
        for w in ["ACAI", "AROMATIC", "COMA", "COMIC", "COMMA", "TACIT"] {
            answers.push(w.to_string());
        }
        let hints = make_hints(&answers, &letters);
        assert_eq!(hints.grid[0][4], 1); // ACAI
        assert_eq!(hints.grid[0][8], 1); // AROMATIC
        assert_eq!(hints.grid[2][4], 1); // COMA
        assert_eq!(hints.grid[2][5], 2); // COMIC, COMMA
        assert_eq!(hints.grid[19][5], 1); // TACIT
        assert_eq!(hints.pangrams, 1);
        let expected = [("AC", 1), ("AR", 1), ("CO", 3), ("TA", 1)];
        assert_eq!(hints.pairs.len(), expected.len());
        for (i, (start, count)) in expected.iter().enumerate() {
            assert_eq!(hints.pairs[i], (start.to_string(), *count));
        }
    }
}
 