use crate::parse;
use crate::settings;
use crate::tui;
use clap::{load_yaml, App, ArgMatches};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};
pub fn args_parser() {
    // Arguments parser
    let yaml = load_yaml!("args.yaml");
    let args = App::from(yaml).get_matches();

    if args.is_present("check") {
        let url = settings::get_url();
        let wl = settings::get_wl();
        parse::feed_check(url, wl);
    } else {
        match args.subcommand() {
            Some(("tui", sub_m)) => arg_t(sub_m),
            Some(("download", sub_m)) => arg_dl(sub_m),
            Some(("parse", sub_m)) => arg_p(sub_m),
            _ => default_logic(),
        }
    }
}

fn default_logic() {
    debug!("Nyaadle started normally.");
    let url = settings::get_url();
    let wl = settings::get_wl();
    parse::feed_parser(url, wl);
}

fn arg_t(sub_m: &ArgMatches) {
    if sub_m.is_present("settings") {
        tui::arg_tui("set");
    } else if sub_m.is_present("watch-list") {
        tui::arg_tui("wle");
    } else {
        tui::main_tui();
    }
}

fn arg_dl(sub_m: &ArgMatches) {
    if sub_m.is_present("links") {
        let input: Vec<_> = sub_m.values_of("links").unwrap().collect();
        let mut links: Vec<String> = Vec::new();
        for tgt in input {
            let link = tgt.to_string();
            links.push(link);
        }

        parse::arg_dl(links);
    } else if sub_m.is_present("file") {
        let input = sub_m.value_of("file").unwrap();
        let file = File::open(input).expect("Failed to open file");
        let buf = BufReader::new(file);
        let links: Vec<String> = buf
            .lines()
            .map(|l| l.expect("Failed to read line"))
            .collect();

        parse::arg_dl(links);
    }
}

fn arg_p(sub_m: &ArgMatches) {
    let feed_bool = sub_m.is_present("feed");
    let item_bool = sub_m.is_present("item");
    let opt_bool = sub_m.is_present("vid-opt");

    if feed_bool && (item_bool || opt_bool) {
        let url = sub_m.value_of("feed").unwrap().to_string();
        item_parse(url, sub_m);
    } else if feed_bool && !item_bool && !opt_bool {
        let url = sub_m.value_of("feed").unwrap().to_string();
        let wl = settings::get_wl();
        info!("Nyaadle started in parse mode. Feed used: \"{}\"", &url);
        parse::feed_parser(url, wl);
    } else {
        let url = settings::get_url();
        item_parse(url, sub_m);
    }

    fn item_parse(url: String, sub_m: &ArgMatches) {
        if sub_m.is_present("item") {
            if !sub_m.is_present("vid-opt") {
                println!("Item Option not provided.");
                println!("Check 'nyaadle parse --help' for more details");
            } else {
                let item = sub_m.value_of("item").unwrap().to_string();
                let opt = sub_m.value_of("vid-opt").unwrap().to_string();
                let wl = settings::wl_builder(item, opt);
                info!("Nyaadle started in parse mode. Feed used: \"{}\"", &url);
                parse::feed_parser(url, wl);
            }
        } else {
            println!("Item to be parsed not provided.");
            println!("Check 'nyaadle parse --help' for more details.")
        }
    }
}
