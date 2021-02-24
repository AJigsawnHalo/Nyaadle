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
    Title,
    Option,
}

/// Implement the Watch-list Editor Table
impl TableViewItem<WatchColumn> for Watchlist {
    fn to_column(&self, column: WatchColumn) -> String {
        match column {
            WatchColumn::Title => self.title.to_string(),
            WatchColumn::Option => self.option.to_string(),
        }
    }
    fn cmp(&self, other: &Self, column: WatchColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            WatchColumn::Title => self.title.cmp(&other.title),
            WatchColumn::Option => self.option.cmp(&other.option),
        }
    }
}

/// The main tui of Nyaadle
pub fn main_tui() {
    let mut siv = cursive::default();

    main_tui_layer(&mut siv);
    // Runs the Cursive Root
    siv.run();
}

/// Matches the &str passed and points to the correct tui
fn on_submit_main(s: &mut Cursive, item: &str) {
    // removes the previous layer
    // Matches the item passed by the main_tui
    match item {
        "wle" => wle_tui(s),
        "set" => set_tui(s),
        _ => unreachable!("Not in item list"),
    };
}

/// Function for setting up the main tui
fn main_tui_layer(s: &mut Cursive) {
    // Setup the Main TUI
    let select = SelectView::<String>::new()
        .item("Watch-list Editor", String::from("wle"))
        .item("Settings", String::from("set"))
        .on_submit(on_submit_main)
        .with_name("select")
        .fixed_size((75, 10));

    // Removes the previous layer
    s.pop_layer();

    // Adds the Main TUI to the Cursive Root
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

/// Function that sets up a Cursive TUI when started from
/// a command-line argument
pub fn arg_tui(item: &str) {
    // Create a blank CUrsive Root
    let mut siv = cursive::default();
    // Add a DummyView to the blank root
    siv.add_layer(DummyView);
    // Matches the arguments passed
    match item {
        "wle" => wle_tui(&mut siv),
        "set" => set_tui(&mut siv),
        _ => unreachable!("Item not in list"),
    };
    // Runs the Cursive Root
    siv.run();
}

/// The Watch-list Editor TUI
fn wle_tui(s: &mut Cursive) {
    println!("Watch List");
    // Removes the previous layer
    s.pop_layer();
    // get the settings dir
    let set_path = settings::settings_dir().to_string();
    // read the items from the database
    let items = settings::read_watch_list(&set_path).expect("Failed to unpack vectors");

    // Set-up the Watch-list Editor TableView
    let mut table = TableView::<Watchlist, WatchColumn>::new()
        .column(WatchColumn::Title, "Item Name", |c| c.width(60))
        .column(WatchColumn::Option, "Option", |c| c.width(10))
        .default_column(WatchColumn::Title);

    // Inserts the items into the table
    table.set_items(items);

    // Buttons at the left side of the TUI
    // Comprised of the Add and Delete Buttons
    let buttons_left = LinearLayout::horizontal()
        .child(Button::new("Add", add_item))
        .child(DummyView)
        .child(Button::new("Delete", delete_item));
    // Buttons at the right side of the TUI
    // Comprised of Navigation Buttons (Back and Quit)
    let buttons_right = LinearLayout::horizontal()
        .child(Button::new("Back", |s| main_tui_layer(s)))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    // Sets up the buttons into a horizontal layer
    let button_layer = LinearLayout::horizontal()
        .child(PaddedView::lrtb(0, 47, 0, 0, buttons_left))
        .child(buttons_right);
    // Adds the views to create the WLE
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(table.with_name("watch-list").min_size((70, 18)))
                .child(button_layer),
        )
        .title("Watch-list Editor"),
    );
}

/// Adds an item in the watch-list
fn add_item(s: &mut Cursive) {
    // Set-up the EditViews
    let edit_title = EditView::new().with_name("title_edit").fixed_width(50);

    let edit_option = EditView::new().with_name("opt_edit").fixed_width(10);

    let title_text = TextView::new("Title:");
    let option_text = TextView::new("Option:");

    // Function runs when the <Ok> button is pressed
    fn ok(s: &mut Cursive, value: String, opt: String) {
        if &value != "" && &opt != "" {
            let set_path = settings::settings_dir().to_string();
            let list = Watchlist {
                title: value,
                option: opt,
            };
            settings::db_write_wl(&set_path, &list.title, &list.option)
                .expect("Failed to write into database");
            s.call_on_name(
                "watch-list",
                |wl: &mut TableView<Watchlist, WatchColumn>| {
                    wl.insert_item(list);
                },
            );
            s.pop_layer();
        } else {
            s.pop_layer();
        }
    }

    // Sets up the Add Item Dialog
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

/// Deletes the currently selected item in the watch-list
fn delete_item(s: &mut Cursive) {
    // Get the settings dir
    let set_path = settings::settings_dir().to_string();
    // retrieve the WLE table
    let mut table = s
        .find_name::<TableView<Watchlist, WatchColumn>>("watch-list")
        .unwrap();
    // Matches the given item index
    match table.item() {
        None => s.add_layer(Dialog::info("No item to delete")),
        // If there's a value, delete it
        Some(index) => {
            let value = table.borrow_item(index).unwrap();
            settings::db_delete_wl(&set_path, &value.title).expect("Failed to delete item");
            table.remove_item(index);
        }
    };
}

/// The Settings Editor TUI
fn set_tui(s: &mut Cursive) {
    println!("Settings");
    // Remove the previous layer
    s.pop_layer();
    // Set-up the Settings SelectView
    let select = SelectView::<String>::new()
        .item("Download directory", String::from("dl-dir"))
        .item("Archive directory", String::from("ar-dir"))
        .item("RSS Feed URL", String::from("url"))
        .item("Log File Path", String::from("log"))
        .on_submit(on_submit_set)
        .with_name("set_select")
        .fixed_size((50, 10));

    // Set-up the navigation buttons
    let buttons = LinearLayout::horizontal()
        .child(Button::new("Back", |s| main_tui_layer(s)))
        .child(Button::new("Quit", Cursive::quit));

    // Create the Settings Editor Dialog
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
    // get the settings dir
    let set_path = settings::settings_dir().to_string();
    // get the current archive directory
    let ar_dir = settings::get_settings(&String::from(item)).unwrap();
    // transform the &str item to a String
    let key = String::from(item);
    // Set up the Archive Dir EditVIew
    let edit = EditView::new()
        .content(ar_dir)
        .with_name("ar_edit")
        .fixed_width(70);

    // Create the Archive Dir Dialog
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
            ok(s, &set_path, &key, value);
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit Archive Directory")
        .fixed_size((70, 10)),
    );

    // Function that runs when <Ok> is pressed
    fn ok(s: &mut Cursive, set_path: &String, dir_key: &String, value: String) {
        settings::update_write_dir(&set_path, &dir_key, &value)
            .expect("Failed to write to database");
        s.pop_layer();
    }
}

/// Dialog box to edit the Downloads Directory
fn dl_edit(s: &mut Cursive, item: &str) {
    // get the settings dir
    let set_path = settings::settings_dir().to_string();
    // get the current download dir
    let dl_dir = settings::get_settings(&String::from(item)).unwrap();
    // transform the &str item to String
    let key = String::from(item);
    // Set-up the Download Dir EditView
    let edit = EditView::new()
        .content(dl_dir)
        .with_name("dl_edit")
        .fixed_width(70);

    // Create the Download Dir Dialog
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
            ok(s, &set_path, &key, value);
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit Download Directory")
        .fixed_size((70, 10)),
    );

    // Function that runs when the <Ok> button is pressed
    fn ok(s: &mut Cursive, set_path: &String, dir_key: &String, value: String) {
        settings::update_write_dir(&set_path, &dir_key, &value)
            .expect("Failed to write to database");
        s.pop_layer();
    }
}

/// Dialog box to edit the Downloads Directory
fn url_edit(s: &mut Cursive, item: &str) {
    // get the settings dir
    let set_path = settings::settings_dir().to_string();
    // get the current url
    let url = settings::get_settings(&String::from(item)).unwrap();
    // transform the &str item to String
    let key = String::from(item);
    // Set-up the URL EditView
    let edit = EditView::new()
        .content(url)
        .with_name("url_edit")
        .fixed_width(70);

    // Create the URL Dialog
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Set the rss feed to parse"))
                .child(edit),
        )
        .button("Ok", move |s| {
            let value = s
                .call_on_name("url_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .expect("Failed to get value");
            ok(s, &set_path, &key, value);
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit RSS Feed URL")
        .fixed_size((70, 10)),
    );

    // Function that runs when the <Ok> button is pressed
    fn ok(s: &mut Cursive, set_path: &String, dir_key: &String, value: String) {
        let mut val = value;
        if val == "" {
            val = String::from("https://nyaa.si/?page=rss");
        }
        settings::update_write_dir(&set_path, &dir_key, &val).expect("Failed to write to database");
        s.pop_layer();
    }
}
/// Dialog box to edit the Downloads Directory
fn log_edit(s: &mut Cursive, item: &str) {
    // get the settings dir
    let set_path = settings::settings_dir().to_string();
    // get the current url
    let url = settings::get_settings(&String::from(item)).unwrap();
    // transform the &str item to String
    let key = String::from(item);
    // Set-up the URL EditView
    let edit = EditView::new()
        .content(url)
        .with_name("log_edit")
        .fixed_width(70);

    // Create the URL Dialog
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Set the log path to write to"))
                .child(edit),
        )
        .button("Ok", move |s| {
            let value = s
                .call_on_name("log_edit", |view: &mut EditView| {
                    view.get_content().to_string()
                })
                .expect("Failed to get value");
            ok(s, &set_path, &key, value);
        })
        .button("Cancel", |s| set_tui(s))
        .title("Edit Log file path")
        .fixed_size((70, 10)),
    );

    // Function that runs when the <Ok> button is pressed
    fn ok(s: &mut Cursive, set_path: &String, dir_key: &String, value: String) {
        let mut log_path_default = dirs::config_dir().unwrap();
        log_path_default.push("nyaadle");
        log_path_default.push("nyaadle");
        log_path_default.set_extension("log");
        let mut val = value;
        if val == "" {
            val = String::from(log_path_default.to_str().unwrap());
        }
        settings::update_write_dir(&set_path, &dir_key, &val).expect("Failed to write to database");
        s.pop_layer();
    }
}
