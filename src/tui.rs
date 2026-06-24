use crate::settings;
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
}

/// Implement the Watch-list Editor Table
impl TableViewItem<WatchColumn> for Watchlist {
    fn to_column(&self, column: WatchColumn) -> String {
        match column {
            WatchColumn::Id => self.id.to_string(),
            WatchColumn::Title => self.title.to_string(),
            WatchColumn::Option => self.option.to_string(),
        }
    }
    fn cmp(&self, other: &Self, column: WatchColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            WatchColumn::Id => self.id.cmp(&other.id),
            WatchColumn::Title => self.title.cmp(&other.title),
            WatchColumn::Option => self.option.cmp(&other.option),
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
        "set" => set_tui(s),
        _ => unreachable!("Not in item list"),
    };
}

/// Function for setting up the main TUI
fn main_tui_layer(s: &mut Cursive) {
    let select = SelectView::<String>::new()
        .item("Watch-list Editor", String::from("wle"))
        .item("Settings", String::from("set"))
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
        _ => unreachable!("Item not in list"),
    };
    siv.run();
}

/// The Watch-list Editor TUI
fn wle_tui(s: &mut Cursive) {
    s.pop_layer();

    // TUI callbacks are 'static so they open their own connections
    let conn = settings::open_conn().expect("Failed to open database.");
    let items = settings::read_watch_list(&conn).expect("Failed to unpack vectors");

    let mut table = TableView::<Watchlist, WatchColumn>::new()
        .column(WatchColumn::Id, "ID", |c| c.width(5))
        .column(WatchColumn::Title, "Item Name", |c| c.width(55))
        .column(WatchColumn::Option, "Option", |c| c.width(10))
        .default_column(WatchColumn::Id);

    table.set_items(items);

    let buttons_left = LinearLayout::horizontal()
        .child(Button::new("Add", add_item))
        .child(Button::new("Edit", edit_item))
        .child(Button::new("Delete", delete_item));

    let buttons_right = LinearLayout::horizontal()
        .child(Button::new("Back", |s| main_tui_layer(s)))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    let button_layer = LinearLayout::horizontal()
        .child(PaddedView::lrtb(0, 47, 0, 0, buttons_left))
        .child(buttons_right);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(table.with_name("watch-list").min_size((70, 18)))
                .child(button_layer),
        )
        .title("Watch-list Editor"),
    );
}

/// Adds an item to the watch-list
fn add_item(s: &mut Cursive) {
    let edit_title = EditView::new().with_name("title_edit").fixed_width(50);
    let edit_option = EditView::new().with_name("opt_edit").fixed_width(10);
    let title_text = TextView::new("Title:");
    let option_text = TextView::new("Option:");

    fn ok(s: &mut Cursive, value: String, opt: String) {
        if !value.is_empty() && !opt.is_empty() {
            let conn = settings::open_conn().expect("Failed to open database.");
            let list = Watchlist {
                id: 0,
                title: value,
                option: opt,
            };
            settings::db_write_wl(&conn, &list.title, &list.option)
                .expect("Failed to write into database");
            s.call_on_name(
                "watch-list",
                |wl: &mut TableView<Watchlist, WatchColumn>| {
                    wl.insert_item(list);
                },
            );
        }
        s.pop_layer();
        wle_tui(s);
    }

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(title_text)
                .child(edit_title)
                .child(option_text)
                .child(edit_option),
        )
        .button("Ok", |s| {
            let value = s
                .call_on_name("title_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .unwrap();
            let opt = s
                .call_on_name("opt_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .unwrap();
            ok(s, value, opt);
        }),
    )
}

/// Edits the selected item in the watch-list
fn edit_item(s: &mut Cursive) {
    let table = s
        .find_name::<TableView<Watchlist, WatchColumn>>("watch-list")
        .unwrap();
    let index = table.item().unwrap();
    let item = table.borrow_item(index).expect("No Item Selected");
    let id = item.id;
    let old_title = item.title.clone();
    let old_opt = item.option.clone();

    let edit_title = EditView::new()
        .content(&old_title)
        .with_name("title_edit")
        .fixed_width(50);
    let edit_option = EditView::new()
        .content(&old_opt)
        .with_name("opt_edit")
        .fixed_width(10);
    let title_text = TextView::new("Title:");
    let option_text = TextView::new("Option:");

    fn ok(s: &mut Cursive, value: &str, opt: String, id: i32) {
        if !value.is_empty() && !opt.is_empty() {
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::update_wl(&conn, value, &opt, &id.to_string())
                .expect("Failed to write to database.");
        }
        s.pop_layer();
        wle_tui(s);
    }

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(title_text)
                .child(edit_title)
                .child(option_text)
                .child(edit_option),
        )
        .button("Ok", move |s| {
            let value = s
                .call_on_name("title_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .unwrap();
            let opt = s
                .call_on_name("opt_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .unwrap();
            ok(s, &value, opt, id);
        }),
    )
}

/// Deletes the currently selected item in the watch-list
fn delete_item(s: &mut Cursive) {
    let mut table = s
        .find_name::<TableView<Watchlist, WatchColumn>>("watch-list")
        .unwrap();
    match table.item() {
        None => s.add_layer(Dialog::info("No item to delete")),
        Some(index) => {
            let value = table.borrow_item(index).unwrap();
            let conn = settings::open_conn().expect("Failed to open database.");
            settings::db_delete_wl(&conn, &value.id.to_string()).expect("Failed to delete item");
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
        .item("Log File Path", String::from("log"))
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
