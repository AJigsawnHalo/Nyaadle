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
extern crate log;
extern crate time;

#[cfg(feature = "discord")]
extern crate serenity;

// The main function of the program.
#[tokio::main]
async fn main() {
    // Ensure the database exists before anything else
    settings::set_check();

    // Open a single shared connection for the lifetime of the program
    let conn = settings::open_conn().expect("Failed to open database.");

    // Set up logging
    let log_path = settings::get_log(&conn);
    if let Some(log_dir) = std::path::Path::new(&log_path).parent() {
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir).expect("Failed to create log directory.");
        }
    }

    let time_format = format_description!(
        "[year]-[month repr:short]-[day] [weekday repr:short] [hour]:[minute]:[second]"
    );
    let log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(log_path)
        .unwrap();
    let conf = ConfigBuilder::new()
        .set_time_format_custom(time_format)
        .set_time_offset_to_local()
        .unwrap()
        .add_filter_ignore_str("serenity")
        .build();

    WriteLogger::init(LevelFilter::Info, conf, log_file).unwrap();

    // TODO: replace with a proper migration system when schema changes are needed.
    settings::get_db_ver(&conn).expect("Failed to set database version.");

    args::args_parser(&conn).await;
}
