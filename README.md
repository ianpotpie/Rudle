# Rurdle

## Welcome

Welcome to the **Rurdle**, a Wordle player and solver!
This program allows you to practice playing Wordle on your own or can help you solve Wordle in a minimal number of guesses!

---

## Features

1. **Play Wordle**:

   - Guess a secret 5-letter word within 6 attempts.
   - Get feedback with color-coded hints:
     - 🟥 **Red**: Letter is not in the word.
     - 🟨 **Yellow**: Letter is in the word but in the wrong position.
     - 🟩 **Green**: Letter is in the correct position.

2. **Solve Wordle**:

   - Use a REPL to interactively narrow down possibilities.
   - Advanced commands to score words, analyze guesses, and undo actions.

3. **Custom Word Lists**:

   - Load your own word list for the game or solver.

4. **Efficient Scoriang**:

   - Scores are based on information theory, reducing the word list optimally.
   - Words are scored based on the entropy of their hint distributions.

---

### Installation

Ensure you have Rust installed. Then clone the repository and build the program:

```bash
git clone https://github.com/your-repo/wordle-solver.git
cd rurdle
make release
```

---

### Usage

Run the program with `--help` to see available options:

```bash
./wordle_solver --help
```

#### Command-Line Arguments:

| Argument | Description                                        |
| -------- | -------------------------------------------------- |
| `--mode` | Specify the mode: `play` or `solve`.               |
| `--file` | Path to the word list file (default: `words.txt`). |

---

### Modes

#### Play Mode

```bash
./wordle_solver --mode play --file wordlist.txt
```

- Start a game where you guess the secret word.
- Follow on-screen instructions for hints and guesses.

#### Solve Mode

```bash
./wordle_solver --mode solve --file wordlist.txt
```

Enter the interactive REPL for solving Wordle puzzles.

##### REPL Commands:

| Command                 | Description                                                           |
| ----------------------- | --------------------------------------------------------------------- |
| `top <n>`               | Show the top `n` guesses and their scores.                            |
| `score <word>`          | Calculate and display the score of a specific word.                   |
| `guessed <word> <hint>` | Add a guessed word and its feedback to narrow down the possibilities. |
| `history`               | Display the history of guesses and feedback.                          |
| `undo`                  | Undo the last guess and restore the word list.                        |
| `exit`                  | Exit the REPL.                                                        |

#### Example of Hint Format:

- Use `_` for incorrect letters.
- Use `*` for misplaced letters.
- Reuse the correct letter for matches.

For example:

```bash
guessed hello h*ll_
```

---

### Word List File Format

- A plain text file containing one word per line.
- Words must be exactly 5 letters long and alphabetic.

Example:

```
apple
brave
cider
delta
...
```

---

Enjoy your Wordle adventures! 🎉
