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
        after_help = "ALIAS:\n     t\n\nEXAMPLE:\n     nyaadle tui\n     nyaadle t --settings\n     nyaadle t -w\n     nyaadle t -f"
    )]
    Tui {
        #[clap(short, long, help = "Opens the settings TUI.")]
        settings: bool,

        #[clap(short, long, help = "Opens the watch-list editor.")]
        watchlist: bool,

        #[clap(short, long, help = "Opens the feeds configuration editor.")]
        feeds: bool,

        #[clap(short, long, help = "Opens the log viewer.")]
        log: bool,
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

        #[clap(
            short,
            long,
            help = "Feed name to link this item to. Falls back to default.",
            value_name = "FEED_NAME"
        )]
        feed: Option<String>,

        #[clap(short, long)]
        print: bool,
    },

    #[clap(visible_alias = "fds", about = "Subcommand to configure RSS Feeds.")]
    Feeds {
        #[clap(short, long, help = "Add a new feed.")]
        add: bool,

        #[clap(short, long, help = "Edit an existing feed's URL.")]
        edit: bool,

        #[clap(short, long, help = "Rename an existing feed.")]
        rename: bool,

        #[clap(short, long, help = "Delete a feed and reassign or drop items.")]
        delete: bool,

        #[clap(long = "set-default", help = "Set a feed as the global default feed.")]
        set_default: bool,

        #[clap(short, long, help = "Print active feed names and URLs.")]
        print: bool,

        #[clap(long, help = "Print all columns including IDs.")]
        all: bool,

        #[clap(short, long, help = "Name of the feed.", value_name = "NAME")]
        name: Option<String>,

        #[clap(
            long = "new-name",
            help = "New name for renaming a feed.",
            value_name = "NEW_NAME"
        )]
        new_name: Option<String>,

        #[clap(short, long, help = "URL of the RSS feed.", value_name = "URL")]
        url: Option<String>,
    },
    #[clap(about = "Opens the log viewer.")]
    Log,
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
            feeds,
            log,
        }) => {
            if settings && !watchlist && !feeds && !log{
                tui::arg_tui("set");
            } else if !settings && watchlist && !feeds && !log{
                tui::arg_tui("wle");
            } else if !settings && !watchlist && feeds && !log{
                tui::arg_tui("fds");
            } else if !settings && !watchlist && !feeds && log {
                tui::arg_tui("log");
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
            parse::arg_parse(conn, args.force, feed, item, vid_opt)
                .await
                .unwrap();
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

        Some(Subcommands::Feeds {
            add,
            edit,
            rename,
            delete,
            set_default,
            print,
            all,
            name,
            new_name,
            url,
        }) => {
            if add {
                let n = name.expect("Feed name is required to add a feed.");
                let u = url.expect("Feed URL is required to add a feed.");
                settings::db_write_feed(conn, &n, &u, false).expect("Failed to save feed.");
                println!("Successfully added feed \"{}\".", n);
            } else if edit {
                let n = name.expect("Feed name is required to edit a feed.");
                let u = url.expect("New feed URL is required.");
                settings::update_feed_url(conn, &n, &u).expect("Failed to update feed URL.");
                println!("Updated feed \"{}\" with new URL.", n);
            } else if rename {
                let n = name.expect("Current feed name is required.");
                let nn = new_name.expect("New feed name is required via --new-name.");
                settings::rename_feed(conn, &n, &nn).expect("Failed to rename feed.");
                println!("Renamed feed \"{}\" to \"{}\".", n, nn);
            } else if set_default {
                let n = name.expect("Feed name is required to set default.");
                settings::set_default_feed(conn, &n).expect("Failed to set default feed.");
                println!("\"{}\" is now the default feed.", n);
            } else if delete {
                let n = name.expect("Feed name is required for deletion.");
                let feeds = settings::read_feeds(conn).expect("Failed to read feeds.");

                if feeds.len() <= 1 {
                    println!("Error: Refusing deletion. Cannot delete the last remaining feed.");
                    return;
                }

                let target_feed = feeds
                    .iter()
                    .find(|f| f.name == n)
                    .expect("Specified feed not found.");

                let mut replacement_default_name = None;
                if target_feed.is_default {
                    println!("'{}' is currently the default feed.", n);
                    println!("Enter the name of the replacement default feed:");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let def_input = input.trim().to_string();
                    if !feeds.iter().any(|f| f.name == def_input && f.name != n) {
                        println!("Invalid replacement feed chosen. Aborting.");
                        return;
                    }
                    replacement_default_name = Some(def_input);
                }

                println!("Enter feed name to reassign dependent watchlist items (leave empty to delete items):");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let reassign_input = input.trim();
                let reassign_name = if !reassign_input.is_empty() {
                    if !feeds
                        .iter()
                        .any(|f| f.name == reassign_input && f.name != n)
                    {
                        println!("Invalid reassignment feed chosen. Aborting.");
                        return;
                    }
                    Some(reassign_input.to_string())
                } else {
                    None
                };

                settings::db_delete_feed(
                    conn,
                    &n,
                    replacement_default_name.as_deref(),
                    reassign_name.as_deref(),
                )
                .expect("Failed to delete feed securely.");
                println!("Feed '{}' successfully purged.", n);
            } else if print {
                let feeds = settings::read_feeds(conn).expect("Failed to read feeds.");
                if all {
                    println!("ID | Default | Feed Name | URL");
                    for f in feeds {
                        let def_marker = if f.is_default { "*" } else { " " };
                        println!("{} |    {}    | {} | {}", f.id, def_marker, f.name, f.url);
                    }
                } else {
                    println!("Feed Name | URL");
                    for f in feeds {
                        let display_name = if f.is_default {
                            format!("{} [default]", f.name)
                        } else {
                            f.name.clone()
                        };
                        println!("{} | {}", display_name, f.url);
                    }
                }
            } else {
                tui::arg_tui("fds");
            }
        }

        Some(Subcommands::WatchlistEditor {
            add,
            delete,
            edit,
            item,
            value,
            option,
            feed,
            print,
        }) => {
            if add && (!delete || !edit || !print) {
                let tgt = item_builder(value, option);
                let feed_id = if let Some(f_name) = feed {
                    let feeds = settings::read_feeds(conn).expect("Failed to read feeds.");
                    feeds
                        .iter()
                        .find(|f| f.name == f_name)
                        .map(|f| f.id)
                        .unwrap_or_else(|| {
                            println!(
                                "Feed '{}' not found! Falling back to system default.",
                                f_name
                            );
                            settings::get_default_feed_id(conn)
                                .expect("No default feed configured.")
                        })
                } else {
                    settings::get_default_feed_id(conn)
                        .expect("No default feed set. Run 'nyaadle feeds --add' first.")
                };
                settings::db_write_wl(conn, &tgt.0, &tgt.1, feed_id)
                    .expect("Failed to write to the database.");
                println!("Added \"{} | {}\" to the watchlist.", &tgt.0, &tgt.1);
            } else if edit && (!add || !delete || !print) {
                let tgt = item_builder(value, option);
                if let Some(ids) = item {
                    for id in ids {
                        if let Some(f_name) = &feed {
                            let feeds = settings::read_feeds(conn).expect("Failed to read feeds.");
                            if let Some(f) = feeds.iter().find(|f| f.name == *f_name) {
                                settings::update_wl_with_feed(conn, &tgt.0, &tgt.1, f.id, &id)
                                    .expect("Failed to update watchlist item feed relation.");
                            } else {
                                println!("Feed '{}' not found. Skipping relation shift.", f_name);
                            }
                        } else {
                            settings::update_wl(conn, &tgt.0, &tgt.1, &id)
                                .expect("Failed to write to the database.");
                        }
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
                let feeds = settings::read_feeds(conn).expect("Failed to read feeds.");

                println!("ID | Item Title | Option | Feed Name");
                for item in wl {
                    let feed_name = feeds
                        .iter()
                        .find(|f| f.id == item.feed_id)
                        .map(|f| f.name.as_str())
                        .unwrap_or("Unknown");

                    println!(
                        "{} | {} | {} | {}",
                        item.id, item.title, item.option, feed_name
                    );
                }
            } else {
                tui::arg_tui("wle");
            }
        }

        None => {
            if args.check {
                parse::feed_parser(conn, args.check, args.force, None, None, None)
                    .await
                    .unwrap();
            } else {
                default_logic(conn, args.force).await;
            }
        }
        Some(Subcommands::Log) => {
            tui::arg_tui("log");
        }
    }
}

async fn default_logic(conn: &Connection, force: bool) {
    if force {
        debug!("Force flag set");
    } else {
        debug!("Nyaadle started normally.");
    }
    parse::feed_parser(conn, false, force, None, None, None).await.unwrap();
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
