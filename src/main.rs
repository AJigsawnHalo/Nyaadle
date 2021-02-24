/// This module handles the parsing functions of nyaadle.
mod parse;
/// This module handles all the settings and watch-list functions
pub mod settings;
pub mod tui;
use clap::{load_yaml, App};
use simplelog::*;
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, BufReader},
};
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

/// The main function of the program.
fn main() {
    // Setup the logging macro and functions
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

    WriteLogger::init(LevelFilter::Info, conf, log_file).unwrap();

    // Arguments parser
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
    } else if let Some(ref args) = args.subcommand_matches("download") {
        if args.is_present("links") {
            let input: Vec<_> = args.values_of("links").unwrap().collect();
            let mut links: Vec<String> = Vec::new();
            for tgt in input {
                let link = tgt.to_string();
                links.push(link);
            }

            parse::arg_dl(links);
        } else if args.is_present("file") {
            let input = args.value_of("file").unwrap();
            let file = File::open(input).expect("Failed to open file");
            let buf = BufReader::new(file);
            let links: Vec<String> = buf
                .lines()
                .map(|l| l.expect("Failed to read line"))
                .collect();

            parse::arg_dl(links);
        }
    } else if let Some(ref args) = args.subcommand_matches("parse") {
        if args.is_present("feed")
            && args.is_present("item") == false
            && args.is_present("vid-opt") == false
        {
            let url = args.value_of("feed").unwrap().to_string();
            let wl = settings::get_wl();
            info!("Nyaadle started in parse mode.");
            parse::feed_parser(url, wl);
        } else if args.is_present("item")
            && args.is_present("vid-opt")
            && args.is_present("feed") == false
        {
            let item = args.value_of("item").unwrap().to_string();
            let opt = args.value_of("vid-opt").unwrap().to_string();
            let wl = settings::wl_builder(item, opt);
            let url = settings::get_url();
            info!("Nyaadle started in parse mode.");
            parse::feed_parser(url, wl);
        } else if args.is_present("item") && args.is_present("vid-opt") == false {
            println!("Option not found");
            println!("Check 'nyaadle parse --help' for details");
        } else if args.is_present("vid-opt") && args.is_present("item") == false {
            println!("Item not found");
            println!("Check 'nyaadle parse --help' for details");
        } else if args.is_present("feed") && args.is_present("item") && args.is_present("vid-opt") {
            let url = args.value_of("feed").unwrap().to_string();
            let item = args.value_of("item").unwrap().to_string();
            let opt = args.value_of("vid-opt").unwrap().to_string();

            let wl = settings::wl_builder(item, opt);

            info!("Nyaadle started in parse mode.");
            parse::feed_parser(url, wl);
        }
    } else if args.is_present("check") {
        let url = settings::get_url();
        let wl = settings::get_wl();
        parse::feed_check(url, wl);
    } else {
        default_logic();
    }
}

fn default_logic() {
    info!("Nyaadle started normally.");
    settings::set_check();
    let url = settings::get_url();
    let wl = settings::get_wl();
    parse::feed_parser(url, wl);
}
