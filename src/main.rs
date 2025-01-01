use clap::{Parser, Subcommand};
use colored::*;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

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

/// Load words from a file and return a Vec of unique words of length 5
/// # Arguments
/// * `file` - The file containing the word list
/// # Returns
/// A Vec of unique words of length 5
/// # Errors
/// If the file cannot be opened
fn load_words(file: String) -> Result<Vec<String>, io::Error> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);

    // Use a HashSet to remove duplicates
    let mut unique_words: HashSet<String> = HashSet::new();

    for word in reader.lines().map_while(Result::ok) {
        if is_valid_word(&word) {
            unique_words.insert(word.to_uppercase());
        }
    }

    let words: Vec<String> = unique_words.into_iter().collect();

    println!("Loaded {} unique words", words.len());
    Ok(words)
}

fn is_valid_word(word: &str) -> bool {
    word.len() == 5
        && word
            .chars()
            .all(|c| c.is_lowercase() && char::is_alphabetic(c))
}

fn play(word_list: Vec<String>) {
    // Select a random word from the word list
    let secret_word = word_list
        .choose(&mut rand::thread_rng())
        .expect("Word list is empty")
        .clone();

    println!("Welcome to Wordle! Guess the 5-letter word. You have 6 attempts.\n");
    println!("Letters are marked red if they don't appear in the word.");
    println!("Letters are marked yellow if they appear in the word but are in the wrong position.");
    println!(
        "Letters are marked green if they appear in the word and are in the correct position.\n"
    );

    let mut attempts = 6;

    while attempts > 0 {
        println!("You have {} attempts left.", attempts);
        print!("Enter your guess: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut guess = String::new();
        std::io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read input");
        let guess = guess.trim().to_uppercase();

        if guess.len() != 5 {
            println!("Please enter a 5-letter word.\n");
            continue;
        }

        if !word_list.contains(&guess) {
            println!("Invalid word. Please try again.\n");
            continue;
        }

        if guess == secret_word {
            println!("{}", "Congratulations! You guessed the word!".green());
            break;
        }

        // Provide feedback for the guess
        let feedback = provide_feedback(&guess, &secret_word);
        println!("{}", feedback);
        attempts -= 1;
    }

    if attempts == 0 {
        println!(
            "{} The correct word was: {}",
            "Game Over!".red(),
            secret_word.green()
        );
    }
}

/// Provide feedback for the guessed word with colors
/// Green: Correct letter in the correct position
/// Yellow: Correct letter in the wrong position
/// Red: Incorrect letter
fn provide_feedback(guess: &str, secret_word: &str) -> String {
    let mut feedback = String::new();

    for (i, c) in guess.chars().enumerate() {
        if secret_word.chars().nth(i).unwrap() == c {
            feedback.push_str(&c.to_string().green().to_string());
        } else if secret_word.contains(c) {
            feedback.push_str(&c.to_string().yellow().to_string());
        } else {
            feedback.push_str(&c.to_string().red().to_string());
        }
    }

    feedback
}

fn solve(word_list: Vec<String>) {
    println!("Solving...");
}
