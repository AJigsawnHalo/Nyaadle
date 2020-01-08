
/// This module contains all the functions of nyaadle.
mod parse;
use std::path::Path;

#[macro_use]
extern crate error_chain;
extern crate reqwest;


/// The main function of the program.
fn main() {
    let settings = parse::settings_dir();
    if Path::new(&settings).exists(){
        parse::feed_parser();
    } else {
        parse::write_settings();
        parse::feed_parser();
    }
}