/// This module handles the arguments passed on through the CLI.
pub mod args;
/// This module handles the parsing functions of nyaadle.
pub mod parse;
/// This module handles all the settings and watch-list functions
pub mod settings;
/// This module creates and handles the TUI
pub mod tui;
use simplelog::*;
use std::fs::OpenOptions;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

/// The main function of the program.
#[tokio::main]
async fn main() {
    // Setup the logging macro and functions
    settings::set_check();
    let log_path = settings::get_log();
    let time_format = String::from("%y-%b-%d %a %H:%M:%S");
    let log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(log_path)
        .unwrap();
    let conf = ConfigBuilder::new()
        .set_time_format(time_format)
        .set_time_to_local(true)
        .build();

    WriteLogger::init(LevelFilter::Debug, conf, log_file).unwrap();

    // TEMPORARY FUNCTION
    settings::get_db_ver().expect("Failed to set database version.");
    args::args_parser().await;
}
