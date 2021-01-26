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
 git clone https://github.com/AJigsawnHalo/Nyaadle.git
 cd Nyaadle
 ```


 2. Install using
  ```
  cargo install --path .
  ```

## Usage
```
Nyaadle 2.2.0-dev.3
A small rss parser and downloader for a certain cat-themed website

USAGE:
    nyaadle.exe [FLAGS] [SUBCOMMAND]

FLAGS:
    -c, --check      Parses the RSS Feed normally but does not download anything
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    download    Downloads the given URL to the set downloads directory [aliases: dl]
    help        Prints this message or the help of the given subcommand(s)
    parse       Parses the given URL [aliases: p]
    tui         Opens an terminal interface to adjust watch-lists and settings [aliases: t]

EXAMPLE:
    nyaadle
    nyaadle tui
    nyaadle dl -l https://foo.bar/bar.file
```
## License
This software is licensed under a [BSD 2-clause license](https://github.com/AJigsawnHalo/Nyaadle/blob/master/LICENSE).
