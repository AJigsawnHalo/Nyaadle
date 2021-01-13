/// This module handles the parsing functions of nyaadle.
mod parse;
/// This module handles all the settings and watch-list functions
pub mod settings;
pub mod tui;
use clap::{load_yaml, App};
use std::path::Path;

#[macro_use]
extern crate error_chain;
extern crate reqwest;

/// The main function of the program.
fn main() {
    let yaml = load_yaml!("args.yaml");
    let args = App::from(yaml).get_matches();

    if let Some(ref args) = args.subcommand_matches("tui") {
        if args.is_present("settings") {
            tui::arg_tui("set");
            return;
        } else if args.is_present("watch-list") {
            tui::arg_tui("wle");
            return;
        } else {
            tui::main_tui();
            return;
        }
    } else {
        default_logic();
    }
}

fn default_logic() {
    let set = settings::settings_dir();
    if Path::new(&set).exists() {
        parse::feed_parser();
    } else {
        settings::write_settings();
        parse::feed_parser();
    }
}
