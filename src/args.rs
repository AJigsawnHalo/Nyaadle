use crate::parse;
use crate::settings;
use crate::tui;
use clap::{Parser, Subcommand};
use rusqlite::Connection;
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

    #[clap(
        short,
        long,
        help = "Force downloading of file even if it has been downloaded already."
    )]
    force: bool,

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
            num_args = 1..,
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
            help = "Used with `--item`. This sets the option value for the item."
        )]
        vid_opt: Option<String>,
    },

    #[clap(visible_alias = "set", about = "Subcommand to configure settings.")]
    Settings {
        #[clap(short, long, help = "Displays the current Settings.")]
        print: bool,

        #[clap(
            short,
            long = "set-dl-dir",
            value_name = "PATH",
            help = "Sets the value of the Download directory."
        )]
        dl_dir: Option<String>,

        #[clap(long = "get-dl-dir", help = "Returns the Download Directory.")]
        get_dl: bool,

        #[clap(
            short,
            long = "set-ar-dir",
            value_name = "PATH",
            help = "Sets the value of the Archive directory."
        )]
        ar_dir: Option<String>,

        #[clap(long = "get-ar-dir", help = "Returns the Archive Directory.")]
        get_ar: bool,

        #[clap(
            short,
            long = "set-feed-url",
            value_name = "URL",
            help = "Sets the value of the RSS Feed URL."
        )]
        url: Option<String>,

        #[clap(long = "get-feed-url", help = "Returns the RSS Feed URL.")]
        get_url: bool,

        #[clap(
            short,
            long = "set-log-file",
            value_name = "PATH",
            help = "Sets the log file location."
        )]
        log: Option<String>,

        #[clap(long = "get-log-path", help = "Returns the log file location.")]
        get_log: bool,

        #[clap(
            short,
            long = "set-discord-webhook",
            value_name = "URL",
            help = "Set the Discord Webhook URL. Requires the \"discord\" feature to be enabled."
        )]
        webhk_url: Option<String>,

        #[clap(
            long = "get-webhook_url",
            help = "Returns the Discord webhook url. Requires the \"discord\" feature to be enabled."
        )]
        get_wbhk: bool,

        #[clap(long = "get-db-ver", help = "Returns the Database version.")]
        get_ver: bool,
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
            num_args = 1..,
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

        #[clap(short, long, help = "Item Option.")]
        option: Option<String>,

        #[clap(short, long)]
        print: bool,
    },
}

pub async fn args_parser(conn: &Connection) {
    let args = Cli::parse();

    if args.force {
        println!("Forcing downloads.");
    }

    match args.subcommand {
        // TUI subcommands open their own connections internally via open_conn()
        // because cursive callbacks require 'static and cannot hold a borrow.
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
                parse::arg_dl(conn, urls).await.unwrap();
            }
            if let Some(name) = file {
                let filename = File::open(name).expect("Failed to open file.");
                let buf = BufReader::new(filename);
                let links: Vec<String> = buf
                    .lines()
                    .map(|l| l.expect("Failed to read line"))
                    .collect();
                parse::arg_dl(conn, links).await.unwrap();
            }
        }

        Some(Subcommands::Parse {
            feed,
            item,
            vid_opt,
        }) => {
            let url = feed.unwrap_or_else(|| settings::get_url(conn));
            match (item, vid_opt) {
                (Some(title), Some(opt)) => {
                    println!("Parsing for: '{}' with option '{}'", &title, &opt);
                    let wl = settings::wl_builder(0, title, opt);
                    parse::feed_parser(conn, url, wl, false, args.force)
                        .await
                        .unwrap();
                }
                (None, None) => {
                    let wl = settings::get_wl(conn);
                    parse::feed_parser(conn, url, wl, false, args.force)
                        .await
                        .unwrap();
                }
                _ => println!("Both --title and --option are required together."),
            }
        }

        Some(Subcommands::Settings {
            dl_dir,
            ar_dir,
            url,
            log,
            webhk_url,
            get_dl,
            get_ar,
            get_url,
            get_log,
            get_wbhk,
            print,
            get_ver,
        }) => {
            if let Some(dl) = dl_dir {
                settings::arg_set(conn, "dl-dir", &dl);
            } else if let Some(ar) = ar_dir {
                settings::arg_set(conn, "ar-dir", &ar);
            } else if let Some(url) = url {
                settings::arg_set(conn, "url", &url);
            } else if let Some(log) = log {
                settings::arg_set(conn, "log", &log);
            } else if let Some(webhk_url) = webhk_url {
                settings::arg_set(conn, "webhk_url", &webhk_url);
            } else if get_dl || get_ar || get_url || get_log || print || get_ver || get_wbhk {
                if get_dl {
                    settings::arg_get_set(conn, "dl-dir");
                }
                if get_ar {
                    settings::arg_get_set(conn, "ar-dir");
                }
                if get_url {
                    settings::arg_get_set(conn, "url");
                }
                if get_log {
                    settings::arg_get_set(conn, "log");
                }
                if get_wbhk {
                    settings::arg_get_set(conn, "webhk_url");
                }
                if get_ver {
                    settings::arg_get_set(conn, "db-ver");
                }
                if print {
                    settings::arg_get_set(conn, "dl-dir");
                    settings::arg_get_set(conn, "ar-dir");
                    settings::arg_get_set(conn, "url");
                    settings::arg_get_set(conn, "log");
                    settings::arg_get_set(conn, "webhk_url");
                    settings::arg_get_set(conn, "db-ver");
                }
            } else {
                tui::arg_tui("set");
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
            if add && (!delete || !edit || !print) {
                let tgt = item_builder(value, option);
                settings::db_write_wl(conn, &tgt.0, &tgt.1)
                    .expect("Failed to write to the database.");
                println!("Added \"{} | {}\" to the watchlist.", &tgt.0, &tgt.1);
            } else if edit && (!add || !delete || !print) {
                let tgt = item_builder(value, option);
                if let Some(ids) = item {
                    for id in ids {
                        settings::update_wl(conn, &tgt.0, &tgt.1, &id)
                            .expect("Failed to write to the database.");
                        println!("Updated {} to \"{} | {}\".", id, &tgt.0, &tgt.1);
                    }
                }
            } else if delete && (!add || !edit || !print) {
                if let Some(ids) = item {
                    for id in ids {
                        settings::db_delete_wl(conn, &id).expect("Failed to delete item");
                        println!("Item deleted.");
                    }
                }
            } else if print && (!add || !edit || !delete) {
                let wl = settings::read_watch_list(conn).expect("Failed to unpack watchlist.");
                println!("ID | Item Title | Option");
                for item in wl {
                    println!("{} | {} | {}", item.id, item.title, item.option);
                }
            } else {
                tui::arg_tui("wle");
            }
        }

        None => {
            if args.check {
                parse::feed_parser(
                    conn,
                    settings::get_url(conn),
                    settings::get_wl(conn),
                    true,
                    false,
                )
                .await
                .unwrap();
            } else {
                default_logic(conn, args.force).await;
            }
        }
    }
}

async fn default_logic(conn: &Connection, force: bool) {
    if force {
        debug!("Force flag set");
    } else {
        debug!("Nyaadle started normally.");
    }
    parse::feed_parser(
        conn,
        settings::get_url(conn),
        settings::get_wl(conn),
        false,
        force,
    )
    .await
    .unwrap();
}

fn item_builder(val: Option<String>, opt: Option<String>) -> (String, String) {
    match (val, opt) {
        (Some(v), Some(o)) => (v, o),
        _ => {
            println!("Please provide both an item name and an item option.");
            std::process::exit(0);
        }
    }
}
