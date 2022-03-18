use crate::parse;
use crate::settings;
use crate::tui;
use clap::{Parser, Subcommand};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

#[derive(Parser)]
#[clap(
    author,
    version,
    about,
    long_about = None,
    after_help = "EXAMPLE:\n    nyaadle\n    nyaadle tui\n    nyaadle dl -l https://foo.bar/bar.file"
)]
struct Cli {
    #[clap(
        short,
        long,
        help = "Parses the RSS Feed normally but does not download anything."
    )]
    check: bool,

    #[clap(subcommand)]
    subcommand: Option<Subcommands>,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    #[clap(
        visible_alias = "t",
        about = "Opens a terminal interface to adjust watch-lists and settings.",
        after_help = "ALIAS:\n     t\n\nEXAMPLE:\n     nyaadle tui\n     nyaadle t --settings\n     nyaadle t -w"
    )]
    Tui {
        #[clap(short, long, help = "Opens the settings TUI.")]
        settings: bool,

        #[clap(short, long, help = "Opens the watch-list editor.")]
        watchlist: bool,
    },

    #[clap(
        visible_alias = "dl",
        about = "Downloads the given URL to the set downloads directory.",
        after_help = "ALIAS:\n     dl\n\nEXAMPLE:\n    nyaadle download -l https://foo.com/bar.torrent\n    nyaadle dl -f input.file\n    nyaadle dl -l https://foo.com/bar1.file https://foo.com/bar2.file",
        arg_required_else_help = true
    )]
    Download {
        #[clap(
            short,
            long,
            multiple_values = true,
            help = "Used for parsing URLs to download from the command-line.",
            value_name = "URLs"
        )]
        links: Option<Vec<String>>,

        #[clap(
            short,
            long = "from-file",
            help = "Used for parsing URLs to download from a given file.",
            value_name = "FILE"
        )]
        file: Option<String>,
    },

    #[clap(
        visible_alias = "p",
        about = "Parses the specified URL or Item.",
        arg_required_else_help = true,
        after_help = "ALIAS:\n     p\n\nEXAMPLE:\n    nyaadle parse -f https://foo.com/bar.rss\n    nyaadle p -i \"Item Title\" -o 720\n    nyaadle p -f https://foo.com/bar1.rss -i \"Item title\" -o non-vid"
    )]
    Parse {
        #[clap(
            short,
            long,
            help = "Parses the given RSS feed instead of the one in the database.",
            value_name = "URL"
        )]
        feed: Option<String>,

        #[clap(
            short = 't',
            long = "title",
            value_name = "TITLE",
            help = "Parses the RSS Feed for the given item. Must be used with `--option`."
        )]
        item: Option<String>,

        #[clap(
            short = 'o',
            long = "option", 
            possible_values = [ "1080", "720", "non-vid" ],
            help = "Used with `--item`. This sets the option value for the item."
        )]
        vid_opt: Option<String>,
    },

    #[clap(visible_alias = "set", about = "Subcommand to configure settings.")]
    Settings {
        #[clap(
            short,
            long = "set-dl-dir",
            value_name = "PATH",
            help = "Sets the value of the Download directory."
        )]
        dl_dir: Option<String>,

        #[clap(
            short,
            long = "set-ar-dir",
            value_name = "PATH",
            help = "Sets the value of the Archive directory."
        )]
        ar_dir: Option<String>,

        #[clap(
            short,
            long = "set-feed-url",
            value_name = "URL",
            help = "Sets the value of the RSS Feed URL."
        )]
        url: Option<String>,

        #[clap(
            short,
            long = "set-log-file",
            value_name = "PATH",
            help = "Sets the log file location."
        )]
        log: Option<String>,

        #[clap(long = "get-dl-dir", help = "Returns the Download Directory.")]
        get_dl: bool,

        #[clap(long = "get-ar-dir", help = "Returns the Archive Directory.")]
        get_ar: bool,

        #[clap(long = "get-feed-url", help = "Returns the RSS Feed URL.")]
        get_url: bool,

        #[clap(long = "get-log-path", help = "Returns the log file location.")]
        get_log: bool,

        #[clap(short, long, help = "Displays the current Settings.")]
        print: bool,
    },

    #[clap(
        visible_alias = "wle",
        about = "Subcommand to configure the Watchlist."
    )]
    WatchlistEditor {
        #[clap(short, long, help = "Add an item.")]
        add: bool,

        #[clap(short, long, help = "Delete an item.")]
        delete: bool,

        #[clap(short, long, help = "Edit an item.")]
        edit: bool,

        #[clap(
            short,
            long,
            help = "Item ID to edit or delete.",
            multiple_values = true,
            value_name = "ID"
        )]
        item: Option<Vec<String>>,

        #[clap(
            short = 't',
            long = "title",
            help = "Name or title of the item.",
            value_name = "TITLE"
        )]
        value: Option<String>,

        #[clap(
            short,
            long,
            help = "Item Option.",
            possible_values = [ "1080", "720", "non-vid" ],
        )]
        option: Option<String>,

        #[clap(short, long)]
        print: bool,
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
                if item == None && vid_opt == None {
                    let wl = settings::get_wl();
                    parse::feed_parser(url, wl);
                } else {
                    item_parse(url, item, vid_opt);
                }
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
        Some(Subcommands::Settings {
            dl_dir,
            ar_dir,
            url,
            log,
            get_dl,
            get_ar,
            get_url,
            get_log,
            print,
        }) => {
            if let Some(dl) = dl_dir {
                settings::arg_set("dl-dir", &dl);
            } else if let Some(ar) = ar_dir {
                settings::arg_set("ar-dir", &ar);
            } else if let Some(url) = url {
                settings::arg_set("url", &url)
            } else if let Some(log) = log {
                settings::arg_set("log", &log);
            } else if get_dl || get_ar || get_url || get_log || print {
                get_set(get_dl, get_ar, get_url, get_log, print);
            } else {
                tui::arg_tui("set");
            }

            fn get_set(dl: bool, ar: bool, url: bool, log: bool, print: bool) {
                if dl {
                    settings::arg_get_set("dl-dir");
                }
                if ar {
                    settings::arg_get_set("ar-dir");
                }
                if url {
                    settings::arg_get_set("url");
                }
                if log {
                    settings::arg_get_set("log");
                }
                if print {
                    settings::arg_get_set("dl-dir");
                    settings::arg_get_set("ar-dir");
                    settings::arg_get_set("url");
                    settings::arg_get_set("log");
                }
            }
        }
        Some(Subcommands::WatchlistEditor {
            add,
            delete,
            edit,
            item,
            value,
            option,
            print,
        }) => {
            let set_path = settings::settings_dir();
            if add && (!delete || !edit || !print) {
                let tgt = item_builder(value, option);
                add_item(&set_path, tgt);
            } else if edit && (!add || !delete || !print) {
                let tgt = item_builder(value, option);
                if let Some(ids) = item {
                    for id in ids {
                        edit_item(&set_path, &tgt, id);
                    }
                }
            } else if delete && (!add || !edit || !print) {
                if let Some(id) = item {
                    del_item(&set_path, id);
                }
            } else if print && (!add || !edit || !delete) {
                print_wl(&set_path);
            } else {
                tui::arg_tui("wle");
            }

            fn item_builder(val: Option<String>, opt: Option<String>) -> (String, String) {
                if let Some(value) = val {
                    let item_val = value;
                    if let Some(option) = opt {
                        let item_opt = option;
                        (item_val, item_opt)
                    } else {
                        println!("Please provide both an item name and an item option.");
                        std::process::exit(0);
                    }
                } else {
                    println!("Please provide both an item name and an item option.");
                    std::process::exit(0);
                }
            }

            fn add_item(set_path: &str, item: (String, String)) {
                settings::db_write_wl(&set_path, &item.0, &item.1)
                    .expect("Failed to write to the database.");
                println!("Added \"{} | {}\" to the watchlist.", &item.0, &item.1);
            }

            fn edit_item(set_path: &str, item: &(String, String), id: String) {
                settings::update_wl(set_path, &item.0, &item.1, &id)
                    .expect("Failed to write to the database.");
                println!("Updated {} to \"{} | {}\".", id, &item.0, &item.1);
            }

            fn del_item(set_path: &str, ids: Vec<String>) {
                for id in ids {
                    settings::db_delete_wl(set_path, &id).expect("Failed to delete item");
                    println!("Item deleted.");
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
        None => arg_check(args),
    }
}

fn default_logic() {
    debug!("Nyaadle started normally.");
    let url = settings::get_url();
    let wl = settings::get_wl();
    parse::feed_parser(url, wl);
}
