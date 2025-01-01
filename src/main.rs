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
        if word.len() == 5 {
            unique_words.insert(word.to_uppercase());
        }
    }

    let words: Vec<String> = unique_words.into_iter().collect();

    println!("Loaded {} unique words", words.len());
    Ok(words)
}

fn play(word_list: Vec<String>) {
    println!("Playing...");
}

fn solve(word_list: Vec<String>) {
    println!("Solving...");
}
