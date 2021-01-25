/// This module handles the parsing functions of nyaadle.
mod parse;
/// This module handles all the settings and watch-list functions
pub mod settings;
pub mod tui;
use clap::{load_yaml, App};
use std::{  
    io::{prelude::*, BufReader},
    fs::File
};
#[macro_use]
extern crate error_chain;
extern crate reqwest;

/// The main function of the program.
fn main() {
    let yaml = load_yaml!("args.yaml");
    let args = App::from(yaml).get_matches();

    if let Some(ref args) = args.subcommand_matches("tui") {
        settings::set_check();
        if args.is_present("settings") {
            tui::arg_tui("set");
        } else if args.is_present("watch-list") {
            tui::arg_tui("wle");
        } else {
            tui::main_tui();
        }
    } else if let Some(ref args) = args.subcommand_matches("download"){
        if args.is_present("links"){
            let input: Vec<_> = args.values_of("links").unwrap().collect();
            let mut links: Vec<String> = Vec::new();
            for tgt in input {
                let link = tgt.to_string();
                links.push(link);
            }
            parse::arg_dl(links);
        } else if args.is_present("file"){
            let input = args.value_of("file").unwrap();
            let file = File::open(input).expect("Failed to open file");
            let buf = BufReader::new(file);
            let links: Vec<String> = buf.lines()
                .map(|l| l.expect("Failed to read line"))
                .collect();

           parse::arg_dl(links); 
        }
    } else {
        default_logic();
    }
}

fn default_logic() {
    settings::set_check();
    parse::feed_parser();
}
