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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Whether to solve the wordle or play it
    /// Possible values: "play", "solve"
    #[arg(short, long)]
    task: String,

    /// Whether the game is in hard mode or easy mode
    /// If it is in hard mode, hints MUST be used
    /// i.e. words disqualified from being the answer by previous hints
    /// cannot be played
    #[arg(short, long, default_value = "easy")]
    mode: String,

    /// The file containing the word list
    #[arg(short, long, default_value = "words.txt")]
    file: String,

    /// The number of letters in the guesses of the game
    #[arg(long, default_value = "5")]
    word_size: usize,

    /// The maximum number of attempts allowed in the game
    #[arg(long, default_value = "6")]
    max_attempts: usize,
}

fn main() -> Result<(), io::Error> {
    let config: Args = Args::parse();

    let word_list = load_words(&config)?;

    match config.task.as_str() {
        "play" => play(word_list, config),
        "solve" => solve(word_list, config),
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
fn load_words(config: &Args) -> Result<Vec<Word>, io::Error> {
    let file = File::open(config.file.clone())?;
    let reader = BufReader::new(file);

    // Use a HashSet to remove duplicates
    let mut unique_words: HashSet<Word> = HashSet::new();

    for word in reader.lines().map_while(Result::ok) {
        if is_valid_word(&word, config) {
            let word = Word::from_string(&word).unwrap();
            unique_words.insert(word);
        }
    }

    let words: Vec<Word> = unique_words.into_iter().collect();
    println!("Loaded {} unique words", words.len());
    Ok(words)
}

fn is_valid_word(word: &str, config: &Args) -> bool {
    word.len() == config.word_size && word.chars().all(char::is_alphabetic)
}

fn play(word_list: Vec<Word>, config: Args) {
    // Select a random word from the word list
    let secret_word = word_list
        .choose(&mut rand::thread_rng())
        .expect("Word list is empty");

    println!(
        "Welcome to Wordle! Guess the {}-letter word. You have 6 attempts.\n",
        config.word_size
    );
    println!("Letters are marked grey if they don't appear in the word.");
    println!(
        "Letters are marked {} if they are in the wrong position.",
        "yellow".yellow()
    );
    println!(
        "Letters are marked {} if they correct position.\n",
        "green".green()
    );

    let mut attempts = 0;

    while attempts < config.max_attempts {
        println!("You have {} attempts left.", config.max_attempts - attempts);
        print!("Enter your guess: ");
        Write::flush(&mut std::io::stdout()).unwrap();

        let mut guess = String::new();
        std::io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read input");
        let guess = guess.trim();

        if guess.len() != config.word_size {
            println!("Please enter a {}-letter word.\n", config.word_size);
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
        let hint = Hint::from_guess_and_answer(&guess, secret_word).unwrap();
        print_hint(&hint, &guess);
        println!();
        attempts += 1;
    }

    if attempts == config.max_attempts {
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
        strict: Option<String>,
    },
    /// Print the score of a word
    Score {
        /// The word to score
        word: String,
    },
    /// Add a word and its hint to narrow the list
    Hint {
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

const HELP_MESSAGE: &str =
    "top <n> [strict]     Print the top n best guesses with their scores, given the
                     remaining possible answers. Scores are the percentage by 
                     which a guessed word reduces the list of possible remaining 
                     answers. If 'strict' is provided, only consider words that
                     score words that are still in the list of possible answers.

score <word>         Print the scores of a word, given the remaining possible 
                     answers. Scores are the percentage by which a guessed word 
                     reduces the list of possible remaining answers. 

hint <word> <hint>   Add a word and its hint to reduce the possible answers.
                     For <word> retype the guessed word.
                     Here is how to type <hint>:
                     - If a letter is green/guessed correctly, retype the letter
                     - If a letter is yellow/misplaced, type '*' in its position
                     - If a letter is grey/incorrect, type '_' in its position
                     Example: 'hint hello h*ll_'

history              Print the history of guesses and feedback

undo                 Undo the last guess and restore the word list

help                 Print the help message, listing the available commands.

exit                 Exit the REPL";

fn solve(word_list: Vec<Word>, config: Args) {
    let mut remaining_guesses = word_list.clone();
    let mut remaining_answers = word_list;
    let mut removed_answers: Vec<Vec<Word>> = vec![];
    let mut guess_history: Vec<(Word, Hint)> = vec![];

    let mut word_scores: Vec<Vec<(Word, f32, f32)>> = vec![];
    word_scores.push(get_scores(&remaining_guesses, &remaining_answers));

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
            println!("{}", HELP_MESSAGE);
            continue;
        }

        // Parse the input into commands
        let args = match SolverArgs::try_parse_from(args) {
            Ok(parsed) => parsed,
            Err(_) => {
                println!("Bad command. Type 'help' for commands.");
                continue;
            }
        };

        // Process the parsed command
        match args.command {
            SolverCommand::Top { n, strict } => {
                let answer_scores;
                let scores = match strict {
                    None => &word_scores[guess_history.len()],
                    Some(s) if s == "strict" => {
                        answer_scores = word_scores[guess_history.len()]
                            .iter()
                            .filter(|(w, _, _)| remaining_answers.contains(w))
                            .cloned()
                            .collect::<Vec<(Word, f32, f32)>>();
                        &answer_scores
                    }
                    _ => {
                        println!("Bad command. Type 'help' for commands.");
                        continue;
                    }
                };

                println!("Rank | Word  | Expected | Worst-Case ");
                println!("-----|-------|----------|------------");
                for (i, (word, avg_score, min_score)) in scores.iter().enumerate() {
                    if i >= n {
                        break;
                    }
                    println!(
                        "{:>4} | {} | {:>7.3}% | {:>9.3}%",
                        i + 1,
                        word.iter().collect::<String>(),
                        avg_score,
                        min_score
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
                let scores = &word_scores[guess_history.len()];

                if let Some((i, (_, avg_score, min_score))) =
                    scores.iter().enumerate().find(|(_, (w, _, _))| w == &word)
                {
                    println!("Rank: {}", i + 1);
                    println!("Expected: {:.3}%", avg_score);
                    println!("Worst-Case: {:.3}%", min_score);
                } else {
                    println!("Word not found in word list.");
                }
            }
            SolverCommand::Hint { guess, hint } => {
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
                if guess.len() != config.word_size || hint.len() != config.word_size {
                    println!(
                        "Guess and hint must both have a size of {}",
                        config.word_size
                    );
                    continue;
                }
                if !guess.iter().all(|c| c.is_alphabetic()) {
                    println!("Guess must only contain alphabetic characters.");
                    continue;
                }
                print_hint(&hint, &guess);
                println!();
                remaining_guesses.retain(|w| w != &guess);
                let removed_words;
                (remaining_answers, removed_words) = remaining_answers.into_iter().partition(|w| {
                    let h = Hint::from_guess_and_answer(&guess, w).expect("Invalid hint");
                    h == hint
                });
                println!("Removed {} words.", removed_words.len());
                println!("{} possible answers remaining.", remaining_answers.len());
                word_scores.push(get_scores(&remaining_guesses, &remaining_answers));
                guess_history.push((guess, hint));
                removed_answers.push(removed_words);
            }
            SolverCommand::History => {
                let mut n_words = remaining_answers.len()
                    + removed_answers
                        .iter()
                        .map(|answers| answers.len())
                        .sum::<usize>();
                println!("Starting with {} words", n_words);

                for (i, ((guess, hint), removed_words)) in
                    zip(guess_history.iter(), removed_answers.iter()).enumerate()
                {
                    let percent_removed = removed_words.len() as f32 * 100.0 / n_words as f32;
                    print!("{}: ", i + 1);
                    print_hint(hint, guess);
                    println!(
                        " - Removed {} of {} ({:.2}%). {} Remaining.",
                        removed_words.len(),
                        n_words,
                        percent_removed,
                        n_words - removed_words.len()
                    );
                    n_words -= removed_words.len();
                }
            }
            SolverCommand::Undo => {
                if let Some((guess, hint)) = guess_history.pop() {
                    print!("Undoing last guess: ");
                    print_hint(&hint, &guess);
                    println!();
                    let answers = removed_answers.pop().expect(
                        "No words to undo, mismatch between history and removed_words_lists",
                    );
                    word_scores
                        .pop()
                        .expect("No word score lists to remove. Something went wrong.");
                    remaining_answers.extend(answers);
                    remaining_guesses.push(guess);
                    println!("Restored word list to {} words.", remaining_answers.len());
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

fn get_scores(guesses: &[Word], answers: &[Word]) -> Vec<(Word, f32, f32)> {
    // Create and configure the progress bar
    println!("Calculating new word scores...");
    let pb = ProgressBar::new(guesses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("##-"),
    );

    // Process words in chunks of size 500 in parallel
    let scores: Vec<(Word, f32, f32)> = guesses
        .par_chunks(100)
        .map(|chunk| {
            let mut chunk_scores = Vec::with_capacity(chunk.len());

            // Process each word in the current chunk (sequentially here)
            for guess in chunk {
                let mut hint_counts = HashMap::new();

                // Accumulate frequencies for all possible answers
                for answer in answers.iter() {
                    let hint = Hint::from_guess_and_answer(guess, answer);
                    let count = hint_counts.entry(hint).or_insert(0.0);
                    *count += 1.0;
                }

                // Calculate score using the accumulated frequencies
                let entropy = -hint_counts
                    .values()
                    .map(|&c| c / answers.len() as f32)
                    .map(|p| p * f32::ln(p))
                    .sum::<f32>();

                let min_score = hint_counts
                    .values()
                    .map(|&c| 100.0 * (1.0 - c / answers.len() as f32))
                    .fold(100.0_f32, |a, b| a.min(b));

                let avg_score = (1.0 - f32::exp(-entropy)) * 100.0;
                chunk_scores.push((guess.clone(), avg_score, min_score));
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

#[derive(PartialEq, Clone, Hash, Eq, Debug)]
struct Word {
    chars: Vec<char>,
}

impl Word {
    fn new(chars: Vec<char>) -> Result<Self, String> {
        if !chars.iter().all(|c| c.is_alphabetic()) {
            return Err("Input string must contain only alphabetic characters.".to_string());
        }

        if !chars.iter().all(|c| c.is_ascii_uppercase()) {
            return Err("Input string must contain only uppercase characters.".to_string());
        }

        Ok(Self { chars })
    }

    fn from_string(s: &str) -> Result<Self, String> {
        if !s.chars().all(char::is_alphabetic) {
            return Err("Input string must contain only alphabetic characters.".to_string());
        }

        let chars: Vec<char> = s.chars().map(|c| c.to_ascii_uppercase()).collect();

        Self::new(chars)
    }

    fn iter(&self) -> std::slice::Iter<char> {
        self.chars.iter()
    }

    fn len(&self) -> usize {
        self.chars.len()
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
    letter_hints: Vec<LetterHint>,
}

impl Hint {
    fn new(letter_hints: Vec<LetterHint>) -> Self {
        Self { letter_hints }
    }

    fn from_string(hint: &str, guess: &Word) -> Result<Self, String> {
        for (c, &w) in zip(hint.chars(), guess.iter()) {
            if c != '*' && c != '_' && c.to_ascii_uppercase() != w {
                return Err("Invalid hint character".to_string());
            }
        }

        let hint: Vec<LetterHint> = zip(hint.chars(), guess.iter())
            .map(|(c, w)| match c {
                _ if (c.to_ascii_uppercase() == *w) => LetterHint::Correct,
                '*' => LetterHint::Misplaced,
                '_' => LetterHint::Incorrect,
                _ => panic!("This case should have been caught earlier"),
            })
            .collect::<Vec<LetterHint>>();

        Ok(Self::new(hint))
    }

    fn from_guess_and_answer(guess: &Word, answer: &Word) -> Result<Self, String> {
        if guess.len() != answer.len() {
            return Err("Guess and answer must have the same length".to_string());
        };
        if !guess.iter().all(|c| c.is_alphabetic()) && !answer.iter().all(|c| c.is_alphabetic()) {
            return Err("Guess and answer must contain only alphabetic characters".to_string());
        }
        let mut letter_hints: Vec<LetterHint> = vec![LetterHint::Incorrect; guess.len()];
        let mut answer_chars = answer.chars.clone();

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

        Ok(Self { letter_hints })
    }

    fn iter(&self) -> std::slice::Iter<LetterHint> {
        self.letter_hints.iter()
    }

    fn len(&self) -> usize {
        self.letter_hints.len()
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
            LetterHint::Incorrect => c.to_string().white().to_string(),
        })
        .collect();
    print!("{}", colored_guess.join(""));
}
