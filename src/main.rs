use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufRead, BufReader, Write};
use std::iter::zip;

const WORDLE_SZ: usize = 5;
const MAX_ATTEMPTS: usize = 6;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The whether to solve the wordle or play it
    #[arg(short, long)]
    mode: String,

    /// The file containing the word list
    #[arg(short, long, default_value = "words.txt")]
    file: String,
}

fn main() -> Result<(), io::Error> {
    let args: Args = Args::parse();

    let word_list = load_words(args.file)?;
    let mode = args.mode.as_str();

    match mode {
        "play" => play(word_list),
        "solve" => solve(word_list),
        _ => println!("Invalid mode"),
    }

    Ok(())
}

/// Load words from a file and return a Vec of unique words of length WORDLE_SZ (5)
/// # Arguments
/// * `file` - The file containing the word list
/// # Returns
/// A Vec of unique words of length WORDLE_SZ (5)
/// # Errors
/// If the file cannot be opened
fn load_words(file: String) -> Result<Vec<Word>, io::Error> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);

    // Use a HashSet to remove duplicates
    let mut unique_words: HashSet<Word> = HashSet::new();

    for word in reader.lines().map_while(Result::ok) {
        if is_valid_word(&word) {
            let word = Word::from_string(&word).unwrap();
            unique_words.insert(word);
        }
    }

    let words: Vec<Word> = unique_words.into_iter().collect();
    println!("Loaded {} unique words", words.len());
    Ok(words)
}

fn is_valid_word(word: &str) -> bool {
    word.len() == WORDLE_SZ && word.chars().all(char::is_alphabetic)
}

fn play(word_list: Vec<Word>) {
    // Select a random word from the word list
    let secret_word = word_list
        .choose(&mut rand::thread_rng())
        .expect("Word list is empty");

    println!("Welcome to Wordle! Guess the {WORDLE_SZ}-letter word. You have 6 attempts.\n");
    println!(
        "Letters are marked {} if they don't appear in the word.",
        "red".red()
    );
    println!(
        "Letters are marked {} if they are in the wrong position.",
        "yellow".yellow()
    );
    println!(
        "Letters are marked {} if they correct position.\n",
        "green".green()
    );

    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        println!("You have {} attempts left.", MAX_ATTEMPTS - attempts);
        print!("Enter your guess: ");
        Write::flush(&mut std::io::stdout()).unwrap();

        let mut guess = String::new();
        std::io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read input");
        let guess = guess.trim();

        if guess.len() != WORDLE_SZ {
            println!("Please enter a {WORDLE_SZ}-letter word.\n");
            continue;
        }

        if !guess.chars().all(char::is_alphabetic) {
            println!("Please enter a word containing only alphabetic characters.\n");
            continue;
        }

        let guess = Word::from_string(guess).unwrap();

        if !word_list.contains(&guess) {
            println!("Invalid word. Please try again.\n");
            continue;
        }

        if guess == *secret_word {
            println!("{}", "Congratulations! You guessed the word!".green());
            break;
        }

        // Provide feedback for the guess
        let hint = Hint::from_guess_and_answer(&guess, secret_word);
        print_hint(&hint, &guess);
        attempts -= 1;
    }

    if attempts == 0 {
        let secret_word: String = secret_word.iter().collect();
        println!(
            "{} The correct word was: {}",
            "Game Over!".red(),
            secret_word.green()
        );
    }
}

/// Command-line arguments for the REPL
#[derive(Parser)]
#[command(author, version, about)]
struct SolverArgs {
    #[command(subcommand)]
    command: SolverCommand,
}

/// REPL commands
#[derive(Subcommand)]
enum SolverCommand {
    /// Print the top n best guesses with their scores
    Top {
        /// Number of guesses to print
        n: usize,
    },
    /// Print the score of a word
    Score {
        /// The word to score
        word: String,
    },
    /// Add a word and feedback to narrow the list
    Guessed {
        /// The guessed word
        guess: String,
        /// Feedback for the guessed word (e.g., "g*y**")
        hint: String,
    },
    /// Print the history of guesses and feedback
    History,
    /// Undo the last guess and restore the word list
    Undo,
    /// Exit the REPL
    Exit,
}

fn solve(word_list: Vec<Word>) {
    let mut remaining_words = word_list;
    let mut removed_words_lists: Vec<Vec<Word>> = vec![];
    let mut guess_history: Vec<(Word, Hint)> = vec![];

    let mut word_score_lists: Vec<Option<Vec<(Word, f32)>>> = vec![];
    word_score_lists.push(None);

    println!("Starting Wordle Solver REPL. Type 'help' for commands.");

    loop {
        // Print the REPL prompt
        print!("> ");
        io::stdout().flush().unwrap();

        // Read user input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let input = input.trim();
        let mut args: Vec<&str> = vec!["repl"];
        args.extend(input.split_whitespace());

        if args.len() == 2 && args[1] == "help" {
            println!("Commands:");
            println!("  top <n> - Print the top n best guesses with their scores, given the current word list.");
            println!("            Scores are the expected percent-reduction of the word list after guessing a word.");
            println!("  score <word> - Print the score of a word, given the current word list.");
            println!("  guessed <word> <feedback> - Add a word and feedback to narrow the list");
            println!("                              Rewriting a letter indicates a match.");
            println!("                              Use \"*\" to represent a misplaced letter.");
            println!("                              Use \"_\" to represent an incorrect letter.");
            println!("                              Example: guessed hello h*ll_");
            println!("  history - Print the history of guesses and feedback");
            println!("  undo - Undo the last guess and restore the word list");
            println!("  exit - Exit the REPL");
            continue;
        }

        // Parse the input into commands
        let args = match SolverArgs::try_parse_from(args) {
            Ok(parsed) => parsed,
            Err(err) => {
                println!("Error: {}", err);
                continue;
            }
        };

        // Process the parsed command
        match args.command {
            SolverCommand::Top { n } => {
                let n_guesses = word_score_lists.len();
                let scores = &word_score_lists[n_guesses - 1];
                if scores.is_none() {
                    let scores = get_scores(&remaining_words);
                    word_score_lists[n_guesses - 1] = Some(scores);
                }
                let scores = word_score_lists[n_guesses - 1].as_ref().unwrap();

                for (i, (word, score)) in scores.iter().enumerate() {
                    if i >= n {
                        break;
                    }
                    println!(
                        "{}: {} ({:.2})",
                        i + 1,
                        word.iter().collect::<String>(),
                        score
                    );
                }
            }
            SolverCommand::Score { word } => {
                let word = match Word::from_string(&word) {
                    Ok(w) => w,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                };
                let n_guesses = word_score_lists.len();
                let scores = &word_score_lists[n_guesses - 1];
                if scores.is_none() {
                    let scores = get_scores(&remaining_words);
                    word_score_lists[n_guesses - 1] = Some(scores);
                }
                let scores = word_score_lists[n_guesses - 1].as_ref().expect(
                    "No scores found. Something went wrong. Scores should have been calculated.",
                );

                if let Some((_, score)) = scores.iter().find(|(w, _)| w == &word) {
                    println!(
                        "Score for {}: {:.2}",
                        word.iter().collect::<String>(),
                        score
                    );
                } else {
                    println!("Word not found in word list.");
                }
            }
            SolverCommand::Guessed { guess, hint } => {
                let guess = match Word::from_string(&guess) {
                    Ok(w) => w,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                };
                let hint = match Hint::from_string(&hint, &guess) {
                    Ok(h) => h,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                };
                print_hint(&hint, &guess);
                let removed_words;
                (remaining_words, removed_words) = remaining_words.into_iter().partition(|w| {
                    let h = Hint::from_guess_and_answer(&guess, w);
                    h == hint
                });
                println!("Removed {} words.", removed_words.len());
                println!("{} possible words remaining.", remaining_words.len());
                word_score_lists.push(None);
                guess_history.push((guess, hint));
                removed_words_lists.push(removed_words);
            }
            SolverCommand::History => {
                for (i, (guess, hint)) in guess_history.iter().enumerate() {
                    print!("{}: ", i + 1);
                    print_hint(hint, guess);
                }
            }
            SolverCommand::Undo => {
                if let Some((guess, hint)) = guess_history.pop() {
                    print!("Undoing last guess: ");
                    print_hint(&hint, &guess);
                    let last_removed_words = removed_words_lists.pop().expect(
                        "No words to undo, mismatch between history and removed_words_lists",
                    );
                    word_score_lists
                        .pop()
                        .expect("No word score lists to remove. Something went wrong.");
                    remaining_words.extend(last_removed_words);
                    println!("Restored word list to {} words.", remaining_words.len());
                } else {
                    println!("Nothing to undo.");
                }
            }
            SolverCommand::Exit => {
                println!("Exiting solver...");
                break;
            }
        }
    }
}

fn get_scores(words: &Vec<Word>) -> Vec<(Word, f32)> {
    let n_words = words.len();
    let freq_unit = 1.0 / n_words as f32;

    // Create and configure the progress bar
    let pb = ProgressBar::new(n_words as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("##-"),
    );

    // Process words in chunks of size 500 in parallel
    let scores: Vec<(Word, f32)> = words
        .par_chunks(100)
        .map(|chunk| {
            let mut chunk_scores = Vec::with_capacity(chunk.len());

            // Process each word in the current chunk (sequentially here)
            for guess in chunk {
                let mut hint_frequencies = HashMap::new();

                // Accumulate frequencies for all possible answers
                for answer in words.iter() {
                    let hint = Hint::from_guess_and_answer(guess, answer);
                    let freq = hint_frequencies.entry(hint).or_insert(0.0);
                    *freq += freq_unit;
                }

                // Calculate score using the accumulated frequencies
                let entropy = -hint_frequencies
                    .values()
                    .map(|&p| p * f32::ln(p))
                    .sum::<f32>();

                let score = (1.0 - f32::exp(-entropy)) * 100.0;
                chunk_scores.push((*guess, score));
            }

            // To reduce contention, update once per chunk
            pb.inc(chunk.len() as u64);

            chunk_scores
        })
        .flat_map_iter(|chunk_scores| chunk_scores)
        .collect();

    pb.finish_with_message("Scoring complete!");

    let mut sorted_scores = scores;
    // Sort by score descending
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sorted_scores
}

#[derive(PartialEq, Clone, Copy, Hash, Eq, Debug)]
struct Word {
    chars: [char; WORDLE_SZ],
}

impl Word {
    fn new(chars: [char; WORDLE_SZ]) -> Result<Self, String> {
        if !chars.iter().all(|c| char::is_alphabetic(*c)) {
            return Err("Input string must contain only alphabetic characters.".to_string());
        }

        if !chars.iter().all(|c| c.is_ascii_uppercase()) {
            return Err("Input string must contain only uppercase characters.".to_string());
        }

        Ok(Self { chars })
    }

    fn from_string(s: &str) -> Result<Self, String> {
        if s.len() != WORDLE_SZ {
            return Err(format!(
                "Input string must be {} characters long.",
                WORDLE_SZ
            ));
        }

        if !s.chars().all(char::is_alphabetic) {
            return Err("Input string must contain only alphabetic characters.".to_string());
        }

        let word: [char; WORDLE_SZ] = s
            .chars()
            .map(|c| c.to_ascii_uppercase())
            .collect::<Vec<char>>()
            .try_into()
            .expect("Conversion failed");

        Self::new(word)
    }

    fn iter(&self) -> std::slice::Iter<char> {
        self.chars.iter()
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.chars.iter().collect::<String>())
    }
}

impl Iterator for Word {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.chars.first().copied()
    }
}

/// A hint for a given letter
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
enum LetterHint {
    /// The letter is in the word and in the correct position
    Correct,
    /// The letter is in the word but in the wrong position
    Misplaced,
    /// The letter is not in the word
    Incorrect,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct Hint {
    letter_hints: [LetterHint; WORDLE_SZ],
}

impl Hint {
    fn new(letter_hints: [LetterHint; WORDLE_SZ]) -> Self {
        Self { letter_hints }
    }

    fn from_string(s: &str, guess: &Word) -> Result<Self, String> {
        let chars: Vec<char> = s.chars().collect();

        if chars.len() != WORDLE_SZ {
            return Err(format!(
                "Input string must be {} characters long.",
                WORDLE_SZ
            ));
        }

        for (&c, &w) in zip(chars.iter(), guess.iter()) {
            if c != '*' && c != '_' && c.to_ascii_uppercase() != w {
                return Err("Invalid hint character".to_string());
            }
        }

        let hint: [LetterHint; WORDLE_SZ] = zip(chars, guess.iter())
            .map(|(c, w)| match c {
                _ if (c.to_ascii_uppercase() == *w) => LetterHint::Correct,
                '*' => LetterHint::Misplaced,
                '_' => LetterHint::Incorrect,
                _ => panic!("This case should have been caught earlier"),
            })
            .collect::<Vec<LetterHint>>()
            .try_into()
            .expect("Conversion failed");

        Ok(Self::new(hint))
    }

    fn from_guess_and_answer(guess: &Word, answer: &Word) -> Self {
        let mut letter_hints: [LetterHint; WORDLE_SZ] = [LetterHint::Incorrect; WORDLE_SZ];
        let mut answer_chars = answer.chars.to_vec();

        // First pass: Check for correct letters (LetterHint::Correct)
        for (i, (g, a)) in zip(guess.iter(), answer.iter()).enumerate() {
            if g == a {
                letter_hints[i] = LetterHint::Correct;
                answer_chars[i] = '_'; // Mark this character as used
            }
        }

        // Second pass: Check for misplaced letters (LetterHint::Misplaced)
        for (i, g) in guess.iter().enumerate() {
            if letter_hints[i] == LetterHint::Correct {
                continue; // Skip already correct letters
            }

            if let Some(pos) = answer_chars.iter().position(|&a| a == *g) {
                letter_hints[i] = LetterHint::Misplaced;
                answer_chars[pos] = '_'; // Mark this character as used
            }
        }

        Self { letter_hints }
    }

    fn iter(&self) -> std::slice::Iter<LetterHint> {
        self.letter_hints.iter()
    }
}

impl Iterator for Hint {
    type Item = LetterHint;

    fn next(&mut self) -> Option<Self::Item> {
        self.letter_hints.first().copied()
    }
}

fn print_hint(hint: &Hint, guess: &Word) {
    let colored_guess: Vec<String> = zip(guess.iter(), hint.iter())
        .map(|(c, h)| match h {
            LetterHint::Correct => c.to_string().green().to_string(),
            LetterHint::Misplaced => c.to_string().yellow().to_string(),
            LetterHint::Incorrect => c.to_string().red().to_string(),
        })
        .collect();
    println!("{}", colored_guess.join(""));
}
