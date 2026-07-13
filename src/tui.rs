use crate::settings;
use crate::settings::Log;
use crate::settings::Watchlist;
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, PaddedView, SelectView, TextView,
};
use cursive::Cursive;
use cursive_table_view::*;
use std::cmp::Ordering;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
/// Columns for the Watch-list Editor
enum WatchColumn {
    Id,
    Title,
    Option,
    Feed,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
/// Columns for the Log viewer
enum LogColumn {
    Timestamp,
    Level,
    Message,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
/// Columns for the Feeds Editor
enum FeedColumn {
    Id,
    Name,
    Url,
    Default,
}

#[derive(Clone)]
/// UI-specific wrapper to pair a watchlist item with its human-readable feed name
struct TuiWatchlist {
    watchlist: Watchlist,
    feed_name: String,
}

// Manually compare the structural components so src/settings.rs doesn't need structural changes
impl PartialEq for TuiWatchlist {
    fn eq(&self, other: &Self) -> bool {
        self.watchlist.id == other.watchlist.id
            && self.watchlist.title == other.watchlist.title
            && self.watchlist.option == other.watchlist.option
            && self.watchlist.feed_id == other.watchlist.feed_id
            && self.feed_name == other.feed_name
    }
}

impl Eq for TuiWatchlist {}

/// Implement the Watch-list Editor Table using the wrapper struct
impl TableViewItem<WatchColumn> for TuiWatchlist {
    fn to_column(&self, column: WatchColumn) -> String {
        match column {
            WatchColumn::Id => self.watchlist.id.to_string(),
            WatchColumn::Title => self.watchlist.title.to_string(),
            WatchColumn::Option => self.watchlist.option.to_string(),
            WatchColumn::Feed => self.feed_name.clone(), // Displays the actual feed name string
        }
    }

    fn cmp(&self, other: &Self, column: WatchColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            WatchColumn::Id => self.watchlist.id.cmp(&other.watchlist.id),
            WatchColumn::Title => self.watchlist.title.cmp(&other.watchlist.title),
            WatchColumn::Option => self.watchlist.option.cmp(&other.watchlist.option),
            WatchColumn::Feed => self.feed_name.cmp(&other.feed_name),
        }
    }
}

/// Implement the Log Viewer Table
impl TableViewItem<LogColumn> for Log {
    fn to_column(&self, column: LogColumn) -> String {
        match column {
            LogColumn::Timestamp => self.timestamp.clone(),
            LogColumn::Level => self.level.clone(),
            LogColumn::Message => self.message.clone(),
        }
    }
    fn cmp(&self, other: &Self, column: LogColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            LogColumn::Timestamp => self.timestamp.cmp(&other.timestamp),
            LogColumn::Level => self.level.cmp(&other.level),
            LogColumn::Message => self.message.cmp(&other.message),
        }
    }
}

/// Implement the Feeds Editor Table
impl TableViewItem<FeedColumn> for settings::Feed {
    fn to_column(&self, column: FeedColumn) -> String {
        match column {
            FeedColumn::Id => self.id.to_string(),
            FeedColumn::Name => {
                if self.is_default {
                    format!("{} [Default]", self.name)
                } else {
                    self.name.clone()
                }
            }
            FeedColumn::Url => self.url.clone(),
            FeedColumn::Default => {
                if self.is_default {
                    String::from("Yes")
                } else {
                    String::from("No")
                }
            }
        }
    }

    fn cmp(&self, other: &Self, column: FeedColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            FeedColumn::Id => self.id.cmp(&other.id),
            FeedColumn::Name => self.name.cmp(&other.name),
            FeedColumn::Url => self.url.cmp(&other.url),
            FeedColumn::Default => self.is_default.cmp(&other.is_default),
        }
    }
}

/// The main TUI of Nyaadle
pub fn main_tui() {
    let mut siv = cursive::default();
    main_tui_layer(&mut siv);
    siv.run();
}

/// Matches the &str passed and points to the correct TUI
fn on_submit_main(s: &mut Cursive, item: &str) {
    match item {
        "wle" => wle_tui(s),
        "fds" => fds_tui(s),
        "set" => set_tui(s),
        "log" => log_tui(s),
        _ => unreachable!("Not in item list"),
    };
}

/// Function for setting up the main TUI
fn main_tui_layer(s: &mut Cursive) {
    let select = SelectView::<String>::new()
        .item("Watch-list Editor", String::from("wle"))
        .item("Feeds Editor", String::from("fds"))
        .item("Settings", String::from("set"))
        .item("Log Viewer", String::from("log"))
        .on_submit(on_submit_main)
        .with_name("select")
        .fixed_size((75, 10));

    s.pop_layer();

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Select an option"))
                .child(DummyView)
                .child(select)
                .child(DummyView)
                .child(Button::new("Quit", Cursive::quit)),
        )
        .title("Nyaadle"),
    );
}

/// Function that sets up a Cursive TUI when started from a command-line argument
pub fn arg_tui(item: &str) {
    let mut siv = cursive::default();
    siv.add_layer(DummyView);
    match item {
        "wle" => wle_tui(&mut siv),
        "set" => set_tui(&mut siv),
        "fds" => fds_tui(&mut siv),
        _ => unreachable!("Item not in list"),
    };
    siv.run();
}

/// The Watch-list Editor TUI
fn wle_tui(s: &mut Cursive) {
    s.pop_layer();

    let conn = settings::open_conn().expect("Failed to open database.");
    let items = settings::read_watch_list(&conn).expect("Failed to unpack vectors");
    let feeds = settings::read_feeds(&conn).expect("Failed to unpack tracking channels");

    let tui_items: Vec<TuiWatchlist> = items
        .into_iter()
        .map(|item| {
            let feed_name = feeds
                .iter()
                .find(|f| f.id == item.feed_id)
                .map(|f| f.name.clone())
                .unwrap_or_else(|| String::from("Unknown"));
            TuiWatchlist {
                watchlist: item,
                feed_name,
            }
        })
        .collect();

    let mut table = TableView::<TuiWatchlist, WatchColumn>::new()
        .column(WatchColumn::Id, "ID", |c| c.width(5))
        .column(WatchColumn::Title, "Item Name", |c| c.width(45))
        .column(WatchColumn::Option, "Option", |c| c.width(10))
        .column(WatchColumn::Feed, "Feed", |c| c.width(15))
        .default_column(WatchColumn::Id);

    table.set_items(tui_items);

    let buttons_left = LinearLayout::horizontal()
        .child(Button::new("Add", add_item))
        .child(Button::new("Edit", edit_item))
        .child(Button::new("Delete", delete_item));

    let buttons_right = LinearLayout::horizontal()
        .child(Button::new("Back", |s| main_tui_layer(s)))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    let button_layer = LinearLayout::horizontal()
        .child(PaddedView::lrtb(0, 42, 0, 0, buttons_left))
        .child(buttons_right);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(table.with_name("watch-list").min_size((75, 18)))
                .child(button_layer),
        )
        .title("Watch-list Editor"),
    );
}

/// Adds an item to the watch-list
fn add_item(s: &mut Cursive) {
    let edit_title = EditView::new().with_name("title_edit").fixed_width(50);
    let edit_option = EditView::new().with_name("opt_edit").fixed_width(10);

    let conn = settings::open_conn().expect("Failed to open database.");
    let feeds = settings::read_feeds(&conn).expect("Failed to load tracking channels.");

    let mut feed_select = SelectView::<i32>::new();
    let mut default_idx = 0;
    for (i, f) in feeds.iter().enumerate() {
        feed_select.add_item(&f.name, f.id);
        if f.is_default {
            default_idx = i;
        }
    }
    feed_select.set_selection(default_idx);

    fn ok(s: &mut Cursive, value: String, opt: String, feed_id: i32) {
        if !value.is_empty() && !opt.is_empty() {
            let conn = settings::open_conn().expect("Failed to open database.");

            let feeds = settings::read_feeds(&conn).expect("Failed to read feeds");
            let feed_name = feeds
                .iter()
                .find(|f| f.id == feed_id)
                .map(|f| f.name.clone())
                .unwrap_or_else(|| String::from("Unknown"));

            let list = Watchlist {
                id: 0,
                title: value,
                option: opt,
                feed_id,
            };
            settings::db_write_wl(&conn, &list.title, &list.option, list.feed_id)
                .expect("Failed to write into database");

            let tui_item = TuiWatchlist {
                watchlist: list,
                feed_name,
            };

            s.call_on_name(
                "watch-list",
                |wl: &mut TableView<TuiWatchlist, WatchColumn>| {
                    wl.insert_item(tui_item);
                },
            );
        }
        s.pop_layer();
        wle_tui(s);
    }

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Title:"))
                .child(edit_title)
                .child(TextView::new("Option:"))
                .child(edit_option)
                .child(TextView::new("Target Feed Channel:"))
                .child(feed_select.with_name("feed_select_box")),
        )
        .button("Ok", |s| {
            let value = s
                .call_on_name("title_edit", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            let opt = s
                .call_on_name("opt_edit", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            let feed_id = *s
                .call_on_name("feed_select_box", |v: &mut SelectView<i32>| {
                    v.selection().unwrap()
                })
                .unwrap();
            ok(s, value, opt, feed_id);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    )
}

/// Edits the selected item in the watch-list
fn edit_item(s: &mut Cursive) {
    let table = s
        .find_name::<TableView<TuiWatchlist, WatchColumn>>("watch-list")
        .unwrap();
    let index = match table.item() {
        Some(idx) => idx,
        None => {
            s.add_layer(Dialog::info("No item selected"));
            return;
        }
    };
    let item = table.borrow_item(index).expect("No Item Selected");
    let id = item.watchlist.id;
    let old_title = item.watchlist.title.clone();
    let old_opt = item.watchlist.option.clone();
    let current_feed_id = item.watchlist.feed_id;

    let edit_title = EditView::new()
        .content(&old_title)
        .with_name("title_edit")
        .fixed_width(50);
    let edit_option = EditView::new()
        .content(&old_opt)
        .with_name("opt_edit")
        .fixed_width(10);

    let conn = settings::open_conn().expect("Failed to open database.");
    let feeds = settings::read_feeds(&conn).expect("Failed to load tracking channels.");

    let mut feed_select = SelectView::<i32>::new();
    let mut selection_idx = 0;
    for (i, f) in feeds.iter().enumerate() {
        feed_select.add_item(&f.name, f.id);
        if f.id == current_feed_id {
            selection_idx = i;
        }
    }
    feed_select.set_selection(selection_idx);

    fn ok(s: &mut Cursive, value: &str, opt: String, id: i32, feed_id: i32) {
        if !value.is_empty() && !opt.is_empty() {
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::update_wl_with_feed(&conn, value, &opt, feed_id, &id.to_string())
                .expect("Failed to write to database.");
        }
        s.pop_layer();
        wle_tui(s);
    }

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Title:"))
                .child(edit_title)
                .child(TextView::new("Option:"))
                .child(edit_option)
                .child(TextView::new("Target Feed Channel:"))
                .child(feed_select.with_name("feed_select_box")),
        )
        .button("Ok", move |s| {
            let value = s
                .call_on_name("title_edit", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            let opt = s
                .call_on_name("opt_edit", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            let feed_id = *s
                .call_on_name("feed_select_box", |v: &mut SelectView<i32>| {
                    v.selection().unwrap()
                })
                .unwrap();
            ok(s, &value, opt, id, feed_id);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    )
}

/// Deletes the currently selected item in the watch-list
fn delete_item(s: &mut Cursive) {
    let mut table = s
        .find_name::<TableView<TuiWatchlist, WatchColumn>>("watch-list")
        .unwrap();
    match table.item() {
        None => s.add_layer(Dialog::info("No item to delete")),
        Some(index) => {
            let value = table.borrow_item(index).unwrap();
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::db_delete_wl(&conn, &value.watchlist.id.to_string())
                .expect("Failed to delete item");
            table.remove_item(index);
        }
    };
}

/// The Settings Editor TUI
fn set_tui(s: &mut Cursive) {
    s.pop_layer();

    let select = SelectView::<String>::new()
        .item("Download directory", String::from("dl-dir"))
        .item("Archive directory", String::from("ar-dir"))
        .item("RSS Feed URL", String::from("url"))
        .item("Log File Path", String::from("log"));

    #[cfg(feature = "discord")]
    let select = {
        let mut s = select;
        s.add_item("Discord Webhook URL", String::from("webhk_url"));
        s
    };

    let select = select
        .on_submit(on_submit_set)
        .with_name("set_select")
        .fixed_size((50, 10));

    let buttons = LinearLayout::horizontal()
        .child(Button::new("Back", |s| main_tui_layer(s)))
        .child(Button::new("Quit", Cursive::quit));

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Select a setting to change"))
                .child(select)
                .child(DummyView)
                .child(buttons),
        )
        .title("Settings Editor"),
    );
}

/// Matches then points to the correct settings dialog
fn on_submit_set(s: &mut Cursive, item: &str) {
    match item {
        "ar-dir" => ar_edit(s, item),
        "dl-dir" => dl_edit(s, item),
        "url" => url_edit(s, item),
        "log" => log_edit(s, item),
        #[cfg(feature = "discord")]
        "webhk_url" => webhk_edit(s, item),
        _ => unreachable!("Item not found in list"),
    };
}

/// Dialog box to edit the Archive Directory
fn ar_edit(s: &mut Cursive, item: &str) {
    let conn = settings::open_conn().expect("Failed to open database.");
    let ar_dir = settings::get_settings(&conn, item).unwrap();
    let key = String::from(item);

    let edit = EditView::new()
        .content(ar_dir)
        .with_name("ar_edit")
        .fixed_width(70);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Set a path where files would be archived"))
                .child(edit),
        )
        .button("Ok", move |s| {
            let value = s
                .call_on_name("ar_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .expect("Failed to get value");
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::update_write_dir(&conn, &key, &value).expect("Failed to write to database");
            s.pop_layer();
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit Archive Directory")
        .fixed_size((70, 10)),
    );
}

/// Dialog box to edit the Downloads Directory
fn dl_edit(s: &mut Cursive, item: &str) {
    let conn = settings::open_conn().expect("Failed to open database.");
    let dl_dir = settings::get_settings(&conn, item).unwrap();
    let key = String::from(item);

    let edit = EditView::new()
        .content(dl_dir)
        .with_name("dl_edit")
        .fixed_width(70);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Set a path where files would be downloaded"))
                .child(edit),
        )
        .button("Ok", move |s| {
            let value = s
                .call_on_name("dl_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .expect("Failed to get value");
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::update_write_dir(&conn, &key, &value).expect("Failed to write to database");
            s.pop_layer();
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit Download Directory")
        .fixed_size((70, 10)),
    );
}

/// Dialog box to edit the RSS Feed URL
fn url_edit(s: &mut Cursive, item: &str) {
    let conn = settings::open_conn().expect("Failed to open database.");
    let url = settings::get_settings(&conn, item).unwrap();
    let key = String::from(item);

    let edit = EditView::new()
        .content(url)
        .with_name("url_edit")
        .fixed_width(70);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Set the rss feed to parse"))
                .child(edit),
        )
        .button("Ok", move |s| {
            let mut value = s
                .call_on_name("url_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .expect("Failed to get value");
            if value.is_empty() {
                value = String::from("https://nyaa.si/?page=rss");
            }
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::update_write_dir(&conn, &key, &value).expect("Failed to write to database");
            s.pop_layer();
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit RSS Feed URL")
        .fixed_size((70, 10)),
    );
}

/// Dialog box to edit the Log File Path
fn log_edit(s: &mut Cursive, item: &str) {
    let conn = settings::open_conn().expect("Failed to open database.");
    let log_path = settings::get_settings(&conn, item).unwrap();
    let key = String::from(item);

    let edit = EditView::new()
        .content(log_path)
        .with_name("log_edit")
        .fixed_width(70);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Set the log path to write to"))
                .child(edit),
        )
        .button("Ok", move |s| {
            let mut value = s
                .call_on_name("log_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .expect("Failed to get value");
            if value.is_empty() {
                let mut default = dirs::config_dir().unwrap();
                default.push("nyaadle");
                default.push("nyaadle");
                default.set_extension("log");
                value = String::from(default.to_str().unwrap());
            }
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::update_write_dir(&conn, &key, &value).expect("Failed to write to database");
            s.pop_layer();
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit Log file path")
        .fixed_size((70, 10)),
    );
}

/// The Log Viewer TUI
fn log_tui(s: &mut Cursive) {
    s.pop_layer();

    let conn = settings::open_conn().expect("Failed to open database.");
    let items = settings::read_logs(&conn).expect("Failed to read logs.");

    let mut table = TableView::<Log, LogColumn>::new()
        .column(LogColumn::Timestamp, "Timestamp", |c| c.width(25))
        .column(LogColumn::Level, "Level", |c| c.width(8))
        .column(LogColumn::Message, "Message", |c| c.width(55))
        .default_column(LogColumn::Timestamp);

    table.set_items(items);

    let buttons = LinearLayout::horizontal()
        .child(Button::new("Back", |s| main_tui_layer(s)))
        .child(Button::new("Quit", Cursive::quit));

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(table.with_name("log-view").min_size((88, 18)))
                .child(buttons),
        )
        .title("Log Viewer"),
    );
}

#[cfg(feature = "discord")]
fn webhk_edit(s: &mut Cursive, item: &str) {
    let conn = settings::open_conn().expect("Failed to open database.");
    let webhk_url = settings::get_settings(&conn, item).unwrap();
    let key = String::from(item);

    let edit = EditView::new()
        .content(webhk_url)
        .with_name("webhk_edit")
        .fixed_width(70);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Set the Discord webhook URL"))
                .child(edit),
        )
        .button("Ok", move |s| {
            let value = s
                .call_on_name("webhk_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .expect("Failed to get value");
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::update_write_dir(&conn, &key, &value).expect("Failed to write to database");
            s.pop_layer();
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit Discord Webhook URL")
        .fixed_size((70, 10)),
    );
}

/// The Feeds Editor TUI view layer
fn fds_tui(s: &mut Cursive) {
    s.pop_layer();

    let conn = settings::open_conn().expect("Failed to open database.");
    let items = settings::read_feeds(&conn).expect("Failed to unpack feeds");

    let mut table = TableView::<settings::Feed, FeedColumn>::new()
        .column(FeedColumn::Id, "ID", |c| c.width(5))
        .column(FeedColumn::Name, "Feed Name", |c| c.width(20))
        .column(FeedColumn::Url, "URL", |c| c.width(40))
        .column(FeedColumn::Default, "Def", |c| c.width(5))
        .default_column(FeedColumn::Id);

    table.set_items(items);

    let buttons_left = LinearLayout::horizontal()
        .child(Button::new("Add", add_feed))
        .child(Button::new("Edit", edit_feed))
        .child(Button::new("Delete", delete_feed))
        .child(Button::new("Set Default", set_default_feed_ui));

    let buttons_right = LinearLayout::horizontal()
        .child(Button::new("Back", |s| main_tui_layer(s)))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    let button_layer = LinearLayout::horizontal()
        .child(PaddedView::lrtb(0, 25, 0, 0, buttons_left))
        .child(buttons_right);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(table.with_name("feeds-list").min_size((70, 18)))
                .child(button_layer),
        )
        .title("Feeds Configuration Editor"),
    );
}

/// Adds a new tracking feed channel
fn add_feed(s: &mut Cursive) {
    let edit_name = EditView::new().with_name("feed_name_edit").fixed_width(30);
    let edit_url = EditView::new().with_name("feed_url_edit").fixed_width(50);

    fn ok(s: &mut Cursive, name: String, url: String) {
        if !name.is_empty() && !url.is_empty() {
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::db_write_feed(&conn, &name, &url, false).expect("Failed to save channel");
        }
        s.pop_layer();
        fds_tui(s);
    }

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Feed Name Key:"))
                .child(edit_name)
                .child(TextView::new("RSS Stream URL:"))
                .child(edit_url),
        )
        .button("Ok", |s| {
            let name = s
                .call_on_name("feed_name_edit", |v: &mut EditView| {
                    v.get_content().to_string()
                })
                .unwrap();
            let url = s
                .call_on_name("feed_url_edit", |v: &mut EditView| {
                    v.get_content().to_string()
                })
                .unwrap();
            ok(s, name, url);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

/// Edits an existing tracking channel's name or endpoint reference
fn edit_feed(s: &mut Cursive) {
    let table = s
        .find_name::<TableView<settings::Feed, FeedColumn>>("feeds-list")
        .unwrap();
    let index = match table.item() {
        Some(idx) => idx,
        None => {
            s.add_layer(Dialog::info("Select a feed target row to edit."));
            return;
        }
    };
    let feed = table.borrow_item(index).unwrap().clone();

    let edit_name = EditView::new()
        .content(&feed.name)
        .with_name("feed_name_edit")
        .fixed_width(30);
    let edit_url = EditView::new()
        .content(&feed.url)
        .with_name("feed_url_edit")
        .fixed_width(50);

    fn ok(s: &mut Cursive, old_name: String, new_name: String, url: String) {
        if !new_name.is_empty() && !url.is_empty() {
            let conn = settings::open_conn().expect("Failed to open database.");
            if old_name != new_name {
                settings::rename_feed(&conn, &old_name, &new_name)
                    .expect("Failed to execute rename sequence");
            }
            settings::update_feed_url(&conn, &new_name, &url)
                .expect("Failed to update path routing");
        }
        s.pop_layer();
        fds_tui(s);
    }

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Feed Name:"))
                .child(edit_name)
                .child(TextView::new("RSS Stream URL:"))
                .child(edit_url),
        )
        .button("Ok", move |s| {
            let name = s
                .call_on_name("feed_name_edit", |v: &mut EditView| {
                    v.get_content().to_string()
                })
                .unwrap();
            let url = s
                .call_on_name("feed_url_edit", |v: &mut EditView| {
                    v.get_content().to_string()
                })
                .unwrap();
            ok(s, feed.name.clone(), name, url);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

/// Manually assigns the global fallback default target channel
fn set_default_feed_ui(s: &mut Cursive) {
    let table = s
        .find_name::<TableView<settings::Feed, FeedColumn>>("feeds-list")
        .unwrap();
    match table.item() {
        None => s.add_layer(Dialog::info("Please highlight a feed entry first.")),
        Some(idx) => {
            let feed = table.borrow_item(idx).unwrap();
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::set_default_feed(&conn, &feed.name)
                .expect("Failed to reallocate system default status");
            s.pop_layer();
            fds_tui(s);
        }
    }
}

/// Initiates the cascading defensive deletion verification sequence
fn delete_feed(s: &mut Cursive) {
    let table = s
        .find_name::<TableView<settings::Feed, FeedColumn>>("feeds-list")
        .unwrap();
    let index = match table.item() {
        Some(idx) => idx,
        None => {
            s.add_layer(Dialog::info("Please select a feed channel entry to clear."));
            return;
        }
    };
    let target_feed = table.borrow_item(index).unwrap().clone();

    let conn = settings::open_conn().expect("Failed to open database.");
    let feeds = settings::read_feeds(&conn).expect("Failed to unpack database entities.");

    if feeds.len() <= 1 {
        s.add_layer(Dialog::info(
            "Action Prohibited: Cannot delete the last remaining feed channel inside the engine.",
        ));
        return;
    }

    if target_feed.is_default {
        let mut def_select = SelectView::<String>::new();
        for f in feeds.iter().filter(|f| f.id != target_feed.id) {
            def_select.add_item(&f.name, f.name.clone());
        }

        s.add_layer(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(format!(
                        "'{}' is currently marked as your default. Choose its successor before dropping:",
                        target_feed.name
                    )))
                    .child(def_select.with_name("successor_def_select")),
            )
            .button("Next", move |s| {
                let replacement_default = s.call_on_name("successor_def_select", |v: &mut SelectView<String>| {
                    v.selection().unwrap().to_string()
                }).unwrap();
                s.pop_layer();
                prompt_tui_watchlist_reassignment(s, target_feed.clone(), Some(replacement_default));
            })
            .button("Cancel", |s| { s.pop_layer(); }),
        );
    } else {
        prompt_tui_watchlist_reassignment(s, target_feed, None);
    }
}

/// Prompt layer handling tracking target allocations or clean deletions
fn prompt_tui_watchlist_reassignment(
    s: &mut Cursive,
    target_feed: settings::Feed,
    replacement_default: Option<String>,
) {
    let conn = settings::open_conn().expect("Failed to open database.");
    let feeds = settings::read_feeds(&conn).expect("Failed to clear tracking frames.");

    let mut reassign_select = SelectView::<Option<String>>::new();
    reassign_select.add_item("[Drop / Purge All Associated Watchlist Items]", None);

    for f in feeds.iter().filter(|f| f.id != target_feed.id) {
        reassign_select.add_item(
            format!("Relocate active items to: {}", f.name),
            Some(f.name.clone()),
        );
    }

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!(
                    "Dependent items are currently tied to channel '{}'. Reassign them:",
                    target_feed.name
                )))
                .child(reassign_select.with_name("item_reallocation_select")),
        )
        .button("Confirm Changes", move |s| {
            let reassign_name = s
                .call_on_name(
                    "item_reallocation_select",
                    |v: &mut SelectView<Option<String>>| (*v.selection().unwrap()).clone(),
                )
                .unwrap();

            let conn = settings::open_conn().expect("Failed to open database.");
            settings::db_delete_feed(
                &conn,
                &target_feed.name,
                replacement_default.as_deref(),
                reassign_name.as_deref(),
            )
            .expect("Failed to drop database dependencies properly.");

            s.pop_layer();
            fds_tui(s);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}
