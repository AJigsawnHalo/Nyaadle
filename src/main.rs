/// This module handles the parsing functions of nyaadle.
mod parse;
/// This module handles all the settings and watch-list functions
pub mod settings;
use std::path::Path;

#[macro_use]
extern crate error_chain;
extern crate reqwest;

/// The main function of the program.
fn main() {
    let set = settings::settings_dir();
    if Path::new(&set).exists() {
        parse::feed_parser();
    } else {
        settings::write_settings();
        parse::feed_parser();
    }
}
