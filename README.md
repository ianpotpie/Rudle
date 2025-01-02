# Rudle

## Welcome

Welcome to **Rudle**, a Wordle player and solver!
This program allows you to play Wordle in your terminal or help you solve Wordle in a minimal number of guesses!

---

### Features

1. **Play Wordle**:

   - Guess a secret 5-letter word with 6 attempts.
   - Get feedback with color-coded hints:
     - ⬜ **Grey**: Letter is not in the word.
     - 🟨 **Yellow**: Letter is in the word but in the wrong position.
     - 🟩 **Green**: Letter is in the word and in the correct position.

2. **Wordle Solver**:

   - Use a REPL to interactively narrow down possibilities.
   - Get best guesses, score words, and see worst-case scenarios.
   - Scores are based on information theory, reducing the word list optimally.
     Specifically, words are scored based on the entropy of their hint distributions.
     This corresponds to the expected amount that they will narrow down the remaining words.

3. **Custom Word Lists**:

   - Load your own vocab list for the game or solver.

---

### Installation

First, ensure you have Rust installed.
Use your package manager or follow the instructions here: https://doc.rust-lang.org/book/ch01-01-installation.html

Then clone the repository and build the program:

```bash
git clone https://github.com/ianpotpie/Rudle.git
cd rudle
make release
```

---

### Usage

Run the program with `--help` to see available options:

```bash
./rudle --help
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
./rudle --mode play --file wordlist.txt
```

- Start a game where you guess the secret word.
- Follow on-screen instructions for hints and guesses.

#### Solve Mode

```bash
./rudle --mode solve --file wordlist.txt
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

- Use `_` for incorrect letters (grey).
- Use `*` for misplaced letters (yellow).
- Reuse the correct letter for matches (green).

For example, if "HELLO" was guessed and the hint
<span style="background: green; color: black; display: inline-block; padding: 2px 5px 0px 5px;">H</span>
<span style="background: yellow; color: black; display: inline-block; padding: 2px 5px 0px 5px;">E</span>
<span style="background: green; color: black; display: inline-block; padding: 2px 5px 0px 5px;">L</span>
<span style="background: green; color: black; display: inline-block; padding: 2px 5px 0px 5px;">L</span>
<span style="background: grey; color: black; display: inline-block; padding: 2px 5px 0px 5px;">O</span>
was given, then entering

```bash
guessed hello h*ll_
```

to tell rudle the guess you made and the hint your received.

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
