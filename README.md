# fcard

fcard is a terminal user interface (TUI) flashcard application written in Rust. It uses the SM-2 spaced repetition algorithm to help you memorize information efficiently.

All flashcard decks are stored as plain text files on your computer. You can write and edit decks in Markdown, YAML, or JSON using any text editor, or add and delete cards directly inside the application.

---

## Table of Contents

- [Description](#description)
- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command-Line Options](#command-line-options)
- [Supported Deck Formats](#supported-deck-formats)
  - [Markdown](#markdown)
  - [JSON](#json)
  - [YAML](#yaml)
- [Modes of Operation](#modes-of-operation)
  - [Browse Mode](#browse-mode)
  - [Flashcard Mode](#flashcard-mode)
  - [Learn Mode](#learn-mode)
  - [Spaced Repetition Mode](#spaced-repetition-mode)
- [Keybindings and Controls](#keybindings-and-controls)
  - [Navigation and Global Keys](#navigation-and-global-keys)
  - [Search Mode](#search-mode)
  - [Command Mode](#command-mode)
  - [Add Card Dialog](#add-card-dialog)
- [Spaced Repetition and Scoring](#spaced-repetition-and-scoring)
- [Themes](#themes)
- [Configuration and Data Storage](#configuration-and-data-storage)

---

## Description

fcard gives you a fast, keyboard-driven study environment inside your terminal. It is built using Ratatui and Crossterm.

Unlike proprietary flashcard apps that store data in closed formats or cloud databases, fcard reads your flashcards directly from human-readable text files in your project or note directories. Review statistics and scheduling data are kept locally on your machine.

---

## Features

- **Multiple File Formats**: Create decks using Markdown (`.md`), YAML (`.yaml`, `.yml`), or JSON (`.json`).
- **SM-2 Spaced Repetition**: Schedules reviews based on your recall performance so you review cards right before you forget them.
- **Learn Mode**: Type answers to test active recall. Includes fuzzy string matching, typo tolerance, and a color-coded character diff that shows where your answer differed.
- **Flashcard Mode**: Traditional study mode where you flip cards and rate your recall from 1 to 4.
- **In-App Editing**: Add new cards through a popup dialog or launch your system text editor directly from the app.
- **Search**: Search across question prompts, answers, and tags with real-time highlighting and match navigation.
- **Vim-Style Navigation**: Navigate panes, decks, and cards using standard keys (`h`, `j`, `k`, `l`, `gg`, `G`, `Ctrl+d`, `Ctrl+u`, `/`, `:`).
- **Multiple Themes**: Switch between 7 built-in themes including Catppuccin Mocha, Dracula, Tokyo Night, Nord, Solarized Dark, High Contrast, and default Terminal.
- **Position Persistence**: Automatically remembers which card you were viewing in each deck between sessions.

---

## Requirements

- **Rust**: Version 1.85.0 or later (edition 2024 support is required).
- **Cargo**: Standard Rust package manager.
- **Operating System**: Linux, macOS, or Android (via Termux).
- Optional: A text editor set in your `EDITOR` or `VISUAL` environment variable (defaults to `nvim`).

---

## Installation

### From Source

1. Clone or download this repository:
   ```sh
   git clone <repository-url> fcard
   cd fcard
   ```

2. Build the release binary:
   ```sh
   cargo build --release
   ```

3. The compiled binary will be placed at `target/release/fcard`. You can copy it to a directory in your `PATH`:
   ```sh
   cp target/release/fcard ~/.local/bin/
   ```

### Install with Cargo

You can install the binary directly into `~/.cargo/bin`:

```sh
cargo install --path .
```

Ensure `~/.cargo/bin` is in your shell `PATH`.

---

## Quick Start

1. Create a sample Markdown deck:
   ```sh
   fcard init geography.md
   ```

2. Launch fcard to view all decks in the current directory:
   ```sh
   fcard
   ```

3. Or launch fcard with a specific deck:
   ```sh
   fcard geography.md
   ```

4. Inside the application:
   - Use `j` and `k` to move between cards.
   - Press `Tab` to switch between the Deck List, Card List, and Preview panes.
   - Press `Enter` to enter Flashcard Mode.
   - Press `Space` to flip a card.
   - Press `1` through `4` to record your rating.
   - Press `l` to enter Learn Mode and type your answers.
   - Press `r` to enter Spaced Repetition Mode to review due cards.
   - Press `q` to return or quit.

---

## Command-Line Options

```text
Usage: fcard [OPTIONS] [DECKS]... [COMMAND]

Arguments:
  [DECKS]...  Paths to deck files or directories to load

Options:
  -p, --path <PATH>    Path to a deck file or directory
  -t, --theme <THEME>  Theme to use (terminal, catppuccin, dracula, tokyo-night,
                       solarized-dark, nord, high-contrast)
  -h, --help           Print help
  -V, --version        Print version

Commands:
  init  Create a new template deck file (detects format from extension)
  edit  Open an existing deck file in your text editor
```

### Discovery Rules

- If you pass file paths directly as arguments, fcard loads them immediately.
- If you pass a directory (or run `fcard` with no arguments to scan the current directory), fcard scans subdirectories recursively. To prevent loading unrelated documents, recursive scanner only loads files that contain an `#fcard` marker anywhere in the file (such as `#fcard`, `# fcard`, or `<!-- fcard -->`).

---

## Supported Deck Formats

### Markdown

Markdown files are identified by `.md` or `.markdown`.

#### Method 1: Field Prefix Format (Recommended)

```markdown
# #fcard
# Capital Cities of the World
Study guide for world capitals.

## France
Front: What is the capital of France?
Back: Paris
Hints: Located on the River Seine.
Tags: geography, europe

## Japan
Front: What is the capital of Japan?
Back: Tokyo
Hints: Formerly known as Edo.
Tags: geography, asia
```

Field labels supported:
- Front: `Front:`, `Q:`, `Question:`
- Back: `Back:`, `A:`, `Answer:`
- Hints: `Hint:`, `Hints:` (comma-separated list)
- Tags: `Tag:`, `Tags:` (comma-separated list)

#### Method 2: Section Header Format

```markdown
# #fcard
# Chemistry Basics

## Water Molecule
### Front
What is the chemical formula for water?
### Back
H2O
```

#### Method 3: Header and Body Format

If no field prefixes or subheadings are present, the level 2 header (`##`) becomes the question front, and the text below it becomes the answer back:

```markdown
# #fcard
# Vocabulary

## Ephemeral
Lasting for a very short time; fleeting.

## Ubiquitous
Present, appearing, or found everywhere.
```

---

### JSON

JSON files are identified by `.json`. You can place `#fcard` as a comment on the very first line.

```json
#fcard
{
  "title": "Rust Programming Basics",
  "description": "Core syntax and language concepts.",
  "cards": [
    {
      "front": "What keyword is used to declare an immutable variable in Rust?",
      "back": "let",
      "hints": ["It has three letters."],
      "tags": ["basics", "variables"]
    },
    {
      "front": "How do you declare a constant in Rust?",
      "back": "const",
      "hints": ["Requires an explicit type annotation."],
      "tags": ["basics", "constants"]
    }
  ]
}
```

---

### YAML

YAML files are identified by `.yaml` or `.yml`.

```yaml
# #fcard
title: Rust Programming Basics
description: Core syntax and language concepts.
cards:
  - front: What keyword is used to declare an immutable variable in Rust?
    back: let
    hints:
      - It has three letters.
    tags:
      - basics
      - variables
  - front: How do you declare a constant in Rust?
    back: const
    hints:
      - Requires an explicit type annotation.
    tags:
      - basics
      - constants
```

---

## Modes of Operation

### Browse Mode

The default mode upon opening the app.
- View decks on the top left.
- View deck statistics in the middle left.
- View the card list on the bottom left.
- Preview question prompts, answers, hints, and tags in the main view pane on the right.

### Flashcard Mode

Traditional self-paced study.
- Press `Enter` from Browse Mode to start.
- Press `Space` to reveal the back of the card.
- Press `1`, `2`, `3`, or `4` to grade your recall:
  - `1`: Again (Failed recall; interval resets to 1 day)
  - `2`: Hard (Difficult recall)
  - `3`: Good (Correct recall; normal interval increase)
  - `4`: Easy (Immediate recall; larger interval increase)
- Press `s` to shuffle the cards.

### Learn Mode

Active recall practice with typing and automated grading.
- Press `l` from Browse Mode to start.
- Type your answer into the prompt.
- Press `Enter` (for single-line answers) or `Ctrl+Enter` / `Ctrl+s` (for multi-line answers) to submit.
- The matcher compares your input with the expected answer:
  - Scores of 75% match or higher are accepted.
  - A color-coded character diff shows exact matches, missing characters, and incorrect additions.
- Cards you miss are placed back into the queue until you answer them correctly.

### Spaced Repetition Mode

Review only the cards that are scheduled for review today based on SM-2 calculation.
- Press `r` from Browse Mode to start.
- If no cards are due, a message informs you that your deck is up to date.
- Review and rate cards as in Flashcard Mode.

---

## Keybindings and Controls

### Navigation and Global Keys

| Key | Action |
| --- | --- |
| `Tab` / `Right` | Move focus to the next pane |
| `BackTab` / `Left` | Move focus to the previous pane |
| `j` / `Down` | Move selection down / Next card |
| `k` / `Up` | Move selection up / Previous card |
| `gg` / `H` | Jump to the first item |
| `G` | Jump to the last item |
| `M` | Jump to the middle item |
| `Ctrl+d` | Scroll down half a page |
| `Ctrl+u` | Scroll up half a page |
| `Enter` | Open Flashcard Mode (from Browse Mode) |
| `l` | Open Learn Mode |
| `r` | Open Spaced Repetition Mode |
| `s` | Shuffle deck |
| `S` | Restore original deck order |
| `a` | Add new card to current deck |
| `d` | Delete currently selected card |
| `e` | Open current deck in external text editor |
| `t` | Open Theme Picker menu |
| `T` | Cycle to the next theme |
| `/` | Start text search |
| `:` | Open Command bar |
| `q` / `Esc` | Return to Browse Mode or quit application |
| `Ctrl+c` | Force quit |

### Search Mode

Press `/` from Browse Mode to start searching.

| Key | Action |
| --- | --- |
| `Type text` | Enter search term |
| `Enter` | Execute search and filter matches |
| `n` | Jump to next matching card |
| `N` | Jump to previous matching card |
| `Esc` | Press once to view status; press twice to clear search filter |

### Command Mode

Press `:` from Browse Mode to open the command line.

| Command | Action |
| --- | --- |
| `:w` or `:write` | Save statistics and configuration to disk |
| `:q` or `:quit` | Exit the program |
| `:wq` or `:x` | Save and exit |
| `:set theme=<name>` | Switch theme (e.g. `:set theme=nord`) |
| `:theme <name>` | Alternative syntax to set theme |
| `:s` or `:shuffle` | Shuffle the current deck |
| `:S` or `:unshuffle` | Reset cards to original file order |
| `:l` or `:learn` | Enter Learn Mode |
| `:fc` or `:flashcard` | Enter Flashcard Mode |
| `:r` or `:review` | Enter Spaced Repetition Mode |
| `:a` or `:add` | Open the Add Card popup |
| `:d` or `:delete` | Delete the currently selected card |
| `:e` or `:edit` | Edit deck file in external editor |
| `:reset` | Reset review statistics and intervals for the active deck |
| `:h` or `:help` | Show command list summary |

### Add Card Dialog

Press `a` to open the card creation window.

| Key | Action |
| --- | --- |
| `Tab` / `Up` / `Down` | Switch between Front (Question) and Back (Answer) |
| `Enter` | Advance to Back field, or save card if in Back field |
| `Ctrl+s` / `Ctrl+Enter` | Save card immediately |
| `Esc` | Cancel and close dialog |

---

## Spaced Repetition and Scoring

fcard implements the **SuperMemo-2 (SM-2)** algorithm:

1. **Repetitions**: Tracks how many times in a row you have successfully recalled a card.
2. **Interval**: The number of days until the card is scheduled for review.
   - First successful recall: 1 day.
   - Second successful recall: 6 days.
   - Subsequent recalls: `Previous Interval * Ease Factor`.
3. **Ease Factor (EF)**: A multiplier representing card difficulty. Starts at `2.5` and never drops below `1.3`.
   - Quality ratings of 4 or 5 increase the Ease Factor.
   - Quality ratings below 3 reset the repetition count and set the review interval back to 1 day.

In **Learn Mode**, text matching is scored between 0.0 and 1.0 using character-level longest common subsequence comparison:
- `Score >= 0.90`: Rated "Correct" (Quality 4)
- `Score >= 0.75`: Rated "Close enough" (Quality 4)
- `Score >= 0.50`: Rated "Almost - check spelling" (Quality 1 - repeats in queue)
- `Score < 0.50`: Rated "Incorrect" (Quality 1 - repeats in queue)

---

## Themes

fcard includes 7 built-in color schemes:

- `terminal` (Uses your default terminal colors)
- `catppuccin-mocha`
- `dracula`
- `tokyo-night`
- `solarized-dark`
- `nord`
- `high-contrast`

Themes can be changed:
- From CLI: `fcard -t dracula`
- In-app via picker: Press `t`, select with arrows, and press `Enter`.
- In-app via command: `:set theme=tokyo-night`
- Quick cycle: Press `T` to cycle through available themes.

Your chosen theme is saved and remembered automatically.

---

## Configuration and Data Storage

fcard stores application data in your system configuration directory:

- **Linux / Termux**: `~/.config/fcard/`
- **macOS**: `~/Library/Application Support/com.fcard.fcard/`

Files stored:
- `config.json`: Contains your selected theme and the last active card index for each deck.
- `stats.json`: Contains SM-2 metrics (repetition counts, ease factors, intervals, review timestamps, correct/incorrect counters) keyed by card ID.

Because statistics are separated from deck files, your Markdown, YAML, and JSON deck files remain clean and easy to edit or track in git.
