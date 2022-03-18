//use std::default;

use crate::parse;
use crate::settings;
use crate::tui;
//use clap::FromArgMatches;
//use clap::{load_yaml, App, ArgMatches};
use clap::{Parser, Subcommand};
//use std::collections::linked_list;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

#[derive(Parser)]
#[clap(author,version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    check: bool,

    #[clap(subcommand)]
    subcommand: Option<Subcommands>,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    #[clap(visible_alias = "t", subcommand_required = false)]
    Tui {
        #[clap(short, long)]
        settings: bool,
        #[clap(short, long)]
        watchlist: bool,
    },

    #[clap(visible_alias = "dl")]
    Download {
        #[clap(short, long, multiple_values = true)]
        links: Option<Vec<String>>,
        #[clap(short, long)]
        file: Option<String>,
    },

    #[clap(visible_alias = "p")]
    Parse {
        #[clap(short, long)]
        feed: Option<String>,

        #[clap(short = 't', long = "title")]
        item: Option<String>,

        #[clap(short = 'o', long = "option", possible_values = [ "1080", "720", "non-vid" ])]
        vid_opt: Option<String>,
    },
}

pub fn args_parser() {
    // Arguments parser
    let args = Cli::parse();

    fn arg_check(args: Cli) {
        if args.check {
            parse::feed_check(settings::get_url(), settings::get_wl());
        } else {
            default_logic();
        }
    }
    match args.subcommand {
        Some(Subcommands::Tui {
            settings,
            watchlist,
        }) => {
            if settings && !watchlist {
                tui::arg_tui("set");
            } else if !settings && watchlist {
                tui::arg_tui("wle");
            } else {
                tui::main_tui();
            }
        }
        Some(Subcommands::Download { links, file }) => {
            if let Some(urls) = links {
                parse::arg_dl(urls);
            }
            if let Some(name) = file {
                let filename = File::open(name).expect("Failed to open file.");
                let buf = BufReader::new(filename);
                let links: Vec<String> = buf
                    .lines()
                    .map(|l| l.expect("Failed to read line"))
                    .collect();

                parse::arg_dl(links);
            }
        }
        Some(Subcommands::Parse {
            feed,
            item,
            vid_opt,
        }) => {
            if let Some(url) = feed {
                item_parse(url, item, vid_opt);
            } else {
                let url = settings::get_url();
                item_parse(url, item, vid_opt);
            }
            fn item_parse(url: String, item_p: Option<String>, vid_opt_p: Option<String>) {
                if let Some(title) = item_p {
                    if let Some(opt) = vid_opt_p {
                        let wl = settings::wl_builder(0, title, opt);
                        parse::feed_parser(url, wl);
                    } else {
                        println!("An option is required. (Ex. '1080', 'non-vid')")
                    }
                } else {
                    println!("Please provide an item to parse.");
                }
            }
        }
        None => arg_check(args),
    }
}

fn default_logic() {
    debug!("Nyaadle started normally.");
    let url = settings::get_url();
    let wl = settings::get_wl();
    parse::feed_parser(url, wl);
}

/*
fn arg_t(sub_m: &ArgMatches) {
    if sub_m.is_present("settings") {
        tui::arg_tui("set");
    } else if sub_m.is_present("watchlist") {
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
                let temp_id = 0;
                let wl = settings::wl_builder(temp_id, item, opt);
                info!("Nyaadle started in parse mode. Feed used: \"{}\"", &url);
                parse::feed_parser(url, wl);
            }
        } else {
            println!("Item to be parsed not provided.");
            println!("Check 'nyaadle parse --help' for more details.")
        }
    }
}
fn arg_s(sub_m: &ArgMatches) {
    if sub_m.is_present("dl-dir") {
        let dir = sub_m
            .value_of("dl-dir")
            .expect("Unable to read given value.");
        settings::arg_set("dl-dir", dir);
    } else if sub_m.is_present("ar-dir") {
        let dir = sub_m
            .value_of("ar-dir")
            .expect("Unable to read given value.");
        settings::arg_set("ar-dir", dir);
    } else if sub_m.is_present("url") {
        let dir = sub_m.value_of("url").expect("Unable to read given value.");
        settings::arg_set("url", dir);
    } else if sub_m.is_present("log") {
        let dir = sub_m.value_of("log").expect("Unable to read given value.");
        settings::arg_set("log", dir);
    } else if sub_m.is_present("get-url")
        || sub_m.is_present("get-ar")
        || sub_m.is_present("get-dl")
        || sub_m.is_present("get-log")
    {
        get_set(&sub_m);
    } else {
        tui::arg_tui("set")
    }
    fn get_set(sub_m: &ArgMatches) {
        if sub_m.is_present("get-dl") {
            settings::arg_get_set("dl-dir");
        }
        if sub_m.is_present("get-ar") {
            settings::arg_get_set("ar-dir");
        }
        if sub_m.is_present("get-url") {
            settings::arg_get_set("url");
        }
        if sub_m.is_present("get-log") {
            settings::arg_get_set("log");
        }
    }
}
fn arg_w(sub_m: &ArgMatches) {
    let add_bool = sub_m.is_present("add");
    let edit_bool = sub_m.is_present("edit");
    let del_bool = sub_m.is_present("delete");
    let print_bool = sub_m.is_present("print");
    let set_path = settings::settings_dir();

    if add_bool && (!edit_bool || !del_bool || !print_bool) {
        add_item(&set_path, sub_m);
    } else if edit_bool && (!add_bool || !del_bool || !print_bool) {
        edit_item(&set_path, sub_m);
    } else if del_bool && (!add_bool || !edit_bool || !print_bool) {
        del_item(&set_path, sub_m);
    } else if print_bool && (!add_bool || !edit_bool || !del_bool) {
        print_wl(&set_path);
    } else {
        tui::arg_tui("wle");
    }

    fn item_builder(sub_m: &ArgMatches) -> (&str, &str) {
        let value = sub_m
            .value_of("value")
            .expect("Unable to read given value.");
        let opt = sub_m
            .value_of("option")
            .expect("Unable to read given value.");

        (&value, &opt)
    }

    fn add_item(set_path: &str, sub_m: &ArgMatches) {
        if sub_m.is_present("value") && sub_m.is_present("option") {
            let item = item_builder(sub_m);
            settings::db_write_wl(&set_path, &item.0, &item.1)
                .expect("Unable to write to the database.");
            println!("Added \"{} | {}\" to the watchlist.", &item.0, &item.1);
        } else {
            println!(
                "Please provide both an item name and an item option. See help for more details."
            );
        }
    }
    fn edit_item(set_path: &str, sub_m: &ArgMatches) {
        if sub_m.is_present("item") {
            if sub_m.is_present("value") && sub_m.is_present("option") {
                let id = sub_m.value_of("item").expect("Failed to get the item id.");
                let item = item_builder(sub_m);
                settings::update_wl(set_path, &item.0, &item.1, &id)
                    .expect("Unable to update item.");
                println!("Updated {} to \"{} | {}\".", id, &item.0, &item.1);
            }
        } else {
            println!("Please select an item to edit.");
        }
    }
    fn del_item(set_path: &str, sub_m: &ArgMatches) {
        if sub_m.is_present("item") {
            let ids: Vec<&str> = sub_m
                .values_of("item")
                .expect("Failed to get item id.")
                .collect();
            for id in ids {
                settings::db_delete_wl(set_path, id).expect("Failed to delete item.");
            }
            println!("Item deleted.");
        } else {
            println!("Please select an item to delete.");
        }
    }
    fn print_wl(set_path: &str) {
        let wl = settings::read_watch_list(&set_path).expect("Failed to unpack watchlist.");
        println!("ID | Item Title | Option");
        for item in wl {
            let id = item.id;
            let title = item.title;
            let opt = item.option;
            println!("{} | {} | {}", id, title, opt);
        }
    }
}
*/
