use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
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
    #[arg(short, long)]
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
            let char_array = string_to_word(&word).unwrap();
            unique_words.insert(char_array);
        }
    }

    let words: Vec<Word> = unique_words.into_iter().collect();

    println!("Loaded {} unique words", words.len());
    Ok(words)
}

fn string_to_word(s: &str) -> Result<Word, String> {
    let chars: Vec<char> = s.chars().collect();

    if chars.len() != WORDLE_SZ {
        return Err(format!(
            "Input string must be {} characters long.",
            WORDLE_SZ
        ));
    }

    // make the word upper case
    let word: Word = chars
        .iter()
        .map(|c| c.to_ascii_uppercase())
        .collect::<Vec<char>>()
        .try_into()
        .expect("Conversion failed");
    Ok(word)
}

fn is_valid_word(word: &str) -> bool {
    word.len() == WORDLE_SZ && word.chars().all(char::is_alphabetic)
}

fn play(word_list: Vec<[char; 5]>) {
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
        let guess = guess.trim().to_uppercase();

        if guess.len() != WORDLE_SZ {
            println!("Please enter a {WORDLE_SZ}-letter word.\n");
            continue;
        }

        let guess = string_to_word(&guess).unwrap(); //TODO: deal with bad length better
        if !word_list.contains(&guess) {
            println!("Invalid word. Please try again.\n");
            continue;
        }

        if guess == *secret_word {
            println!("{}", "Congratulations! You guessed the word!".green());
            break;
        }

        // Provide feedback for the guess
        let feedback = Feedback::from_comparison(&guess, secret_word);
        println!("{}", feedback);
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

/// A hint for a given letter
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
enum LetterHint {
    /// The letter is in the word and in the correct position
    Correct,
    /// The letter is in the word but in the wrong position
    Misplaced,
    /// The letter is not in the word
    Incorrect,
}

type Word = [char; WORDLE_SZ];
type Hint = [LetterHint; WORDLE_SZ];

#[derive(Debug, Clone, PartialEq)]
struct Feedback {
    word: Word,
    hint: Hint,
}

impl fmt::Display for Feedback {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let colored_guess: Vec<String> = self
            .word
            .iter()
            .zip(self.hint.iter())
            .map(|(c, h)| match h {
                LetterHint::Correct => c.to_string().green().to_string(),
                LetterHint::Misplaced => c.to_string().yellow().to_string(),
                LetterHint::Incorrect => c.to_string().red().to_string(),
            })
            .collect();
        write!(f, "{}", colored_guess.join(""))
    }
}

impl Feedback {
    fn from_comparison(guess: &Word, answer: &Word) -> Self {
        let mut hint: Hint = [LetterHint::Incorrect; WORDLE_SZ];
        let mut answer_chars = *answer;

        // First pass: Check for correct letters (LetterHint::Correct)
        for (i, (g, a)) in zip(guess, answer).enumerate() {
            if g == a {
                hint[i] = LetterHint::Correct;
                answer_chars[i] = '_'; // Mark this character as used
            }
        }

        // Second pass: Check for misplaced letters (LetterHint::Misplaced)
        for (i, g) in guess.iter().enumerate() {
            if hint[i] == LetterHint::Correct {
                continue; // Skip already correct letters
            }

            if let Some(pos) = answer_chars.iter().position(|&a| a == *g) {
                hint[i] = LetterHint::Misplaced;
                answer_chars[pos] = '_'; // Mark this character as used
            }
        }

        Self { word: *guess, hint }
    }

    fn fits(&self, word: &Word) -> bool {
        let hint = get_hint(&self.word, word);
        hint == self.hint
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
        word: String,
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

fn string_to_hint(s: &str, word: &Word) -> Result<Hint, String> {
    let chars: Vec<char> = s.chars().collect();

    if chars.len() != WORDLE_SZ {
        return Err(format!(
            "Input string must be {} characters long.",
            WORDLE_SZ
        ));
    }

    for (&c, &w) in zip(chars.iter(), word.iter()) {
        if c != '*' && c != '_' && c != w {
            return Err("Invalid hint character".to_string());
        }
    }

    let hint: Hint = zip(chars, word)
        .map(|(c, &w)| match c {
            _ if (c == w) => LetterHint::Correct,
            '*' => LetterHint::Misplaced,
            '_' => LetterHint::Incorrect,
            _ => panic!("This case should have been caught earlier"),
        })
        .collect::<Vec<LetterHint>>()
        .try_into()
        .expect("Conversion failed");
    Ok(hint)
}

pub fn solve(word_list: Vec<Word>) {
    let mut remaining_words = word_list;
    let mut removed_words_lists: Vec<Vec<Word>> = vec![];
    let mut guess_history: Vec<Feedback> = vec![];

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
                let word = string_to_word(&word).unwrap();
                let n_guesses = word_score_lists.len();
                let scores = &word_score_lists[n_guesses - 1];
                if scores.is_none() {
                    let scores = get_scores(&remaining_words);
                    word_score_lists[n_guesses - 1] = Some(scores);
                }
                let scores = word_score_lists[n_guesses - 1].as_ref().unwrap();

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
            SolverCommand::Guessed { word, hint } => {
                let word = string_to_word(&word).unwrap();
                let hint = string_to_hint(&hint, &word).unwrap();
                let feedback = Feedback { word, hint };
                println!("{}", feedback);
                let removed_words;
                (remaining_words, removed_words) =
                    remaining_words.into_iter().partition(|w| feedback.fits(w));
                println!("Removed {} words.", removed_words.len());
                println!("{} possible words remaining.", remaining_words.len());
                word_score_lists.push(None);
                guess_history.push(feedback);
                removed_words_lists.push(removed_words);
            }
            SolverCommand::History => {
                for (i, feedback) in guess_history.iter().enumerate() {
                    println!("{}: {}", i + 1, feedback);
                }
            }
            SolverCommand::Undo => {
                if let Some(feedback) = guess_history.pop() {
                    println!("Undoing last guess: {}", feedback);
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
                    let hint = get_hint(guess, answer);
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
        // Flatten the results of each chunk into a single parallel iterator
        .flat_map_iter(|chunk_scores| chunk_scores)
        // Collect all scores into a vector
        .collect();

    pb.finish_with_message("Scoring complete!");

    let mut sorted_scores = scores;
    // Sort by score descending
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sorted_scores
}

fn get_hint(guess: &[char; 5], answer: &[char; 5]) -> [LetterHint; 5] {
    let mut hint: Hint = [LetterHint::Incorrect; WORDLE_SZ];
    let mut answer_chars = *answer;

    // First pass: Check for correct letters (LetterHint::Correct)
    for (i, (g, a)) in zip(guess, answer).enumerate() {
        if g == a {
            hint[i] = LetterHint::Correct;
            answer_chars[i] = '_'; // Mark this character as used
        }
    }

    // Second pass: Check for misplaced letters (LetterHint::Misplaced)
    for (i, g) in guess.iter().enumerate() {
        if hint[i] == LetterHint::Correct {
            continue; // Skip already correct letters
        }

        if let Some(pos) = answer_chars.iter().position(|&a| a == *g) {
            hint[i] = LetterHint::Misplaced;
            answer_chars[pos] = '_'; // Mark this character as used
        }
    }

    hint
}
