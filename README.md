# Nyaadle
A small rss parser and downloader for a certain cat-themed website.

## How it works
Nyaadle connects to the cat-themed website and compares it to a watch-list you provide.
If a match is found, it grabs the link and downloads it to a specified folder.

## Installation

### Build Dependencies
 - [Rust compiler and cargo](https://rustup.rs/)

### Build Instructions
 1. Clone the repository or download a zip copy [here](https://github.com/AJigsawnHalo/Nyaadle/releases).
 ```
 git clone https://github.com/AJigsawnHalo/nyaadle.git
 ```


 2. Install using
  ```
  cargo install --path .
  ```

## Usage
Run nyaadle using `nyaadle`. To edit settings and watch-lists use `nyaadle tui`. For more information, run `nyaadle --help` or `nyaadle tui --help`.