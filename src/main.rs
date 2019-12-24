
/// This module contains all the functions of nyaadle.
mod parse;

#[macro_use]
extern crate error_chain;
extern crate reqwest;


/// The main function of the program.
fn main() {
    parse::feed_parser();
}