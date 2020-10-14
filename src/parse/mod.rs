// Parts of this code was adapted from "The Rust Cookbook"
// which can be found at: https://rust-lang-nursery.github.io/rust-cookbook/

use dirs;
use rss::Channel;
use rusqlite::{named_params, Connection, NO_PARAMS};
use std::fs::File;
use std::io::copy;
use std::path::Path;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}
/// Settings Struct
struct Settings {
    dl_key: String,
    dl_val: String,
    ar_key: String,
    ar_val: String,
}
/// Public Watchlist Struct
pub struct Watchlist {
    title: String,
    option: String,
}

/// Checks if the config directory exists and then creates it if it's not found.
pub fn write_settings() {
    // Gets the home directory
    let mut dl_dir = dirs::home_dir().expect("Failed to extract home directory");
    dl_dir.push("Transmission");
    dl_dir.push("torrent-ingest");
    let dl_dir = String::from(dl_dir.to_str().unwrap());

    let mut ar_dir = dirs::home_dir().expect("Failed to extract home directory");
    ar_dir.push("Transmission");
    ar_dir.push("torrent-ingest");
    ar_dir.push("archive");
    let ar_dir = String::from(ar_dir.to_str().unwrap());

    // Default Settings
    let default_set = Settings {
        dl_key: String::from("dl-dir"),
        dl_val: dl_dir,
        ar_key: String::from("ar-dir"),
        ar_val: ar_dir,
    };

    // Default Watchlist
    let default_wl = Watchlist {
        title: String::from(""),
        option: String::from("non-vid"),
    };

    let set_file = settings_dir();
    let mut directory = dirs::config_dir().unwrap();
    directory.push("nyaadle");

    let directory = String::from(directory.to_str().unwrap());

    // If the settings file doesn't exist, create it.
    if Path::new(&set_file).exists() {
        return;
    } else {
        println!("nyaadle.db not found. Creating it right now.");
        // create directory
        if Path::new(&directory).exists() == false {
            std::fs::create_dir(&directory).expect("Unable to create directory");
        }
        // Create nyaadle.db and add dl-dir
        let db_conn = db_create(&set_file);
        let db_ar_write = db_write_dir(&set_file, default_set.ar_key, default_set.ar_val);
        let db_dl_write = db_write_dir(&set_file, default_set.dl_key, default_set.dl_val);

        // Append watch-list to nyaadle.db
        let db_wl_write = db_write_wl(&set_file, default_wl.title, default_wl.option);
        if db_conn == Ok(())
            && db_ar_write == Ok(())
            && db_dl_write == Ok(())
            && db_wl_write == Ok(())
        {
            println!("nyaadle.db created.");
            println!(
                "You can change settings by editing the config file in {}",
                &set_file
            );
        } else {
            println!("Failed to create nyaadle.db");
        }
    }
}

/// Function to create a database with the default tables
fn db_create(set_path: &String) -> rusqlite::Result<()> {
    let conn = Connection::open(&set_path)?;

    // Create the directories table
    conn.execute(
        "create table if not exists directories (
            option text primary key,
            path text not null unique)
            ",
        NO_PARAMS,
    )?;

    // Create the watchlist table
    conn.execute(
        "create table if not exists watchlist (
            id integer primary key,
            name text not null unique,
            option text not null)
            ",
        NO_PARAMS,
    )?;

    Ok(())
}

/// Funtion to write the directory values to the directories table
fn db_write_dir(set_path: &String, dir_key: String, dir_val: String) -> rusqlite::Result<()> {
    // Collect the directory values
    let mut dir = std::collections::HashMap::new();
    dir.insert(dir_key, dir_val);

    // Establish a connection to the database
    let conn = Connection::open(&set_path)?;

    // Insert the values into the table
    for (key, val) in &dir {
        conn.execute(
            "insert into directories
            (option, path) values (?1, ?2)",
            &[&key.to_string(), &val.to_string()],
        )?;
    }

    // return an Ok value
    Ok(())
}

/// Function to write the watchlist values to the watchlist table
fn db_write_wl(set_path: &String, wl_key: String, wl_val: String) -> rusqlite::Result<()> {
    // Collect the watchlist values
    let mut wl = std::collections::HashMap::new();
    wl.insert(wl_key, wl_val);

    // Establish a connection to the database
    let conn = Connection::open(&set_path)?;

    // Insert the values into the table
    for (key, val) in &wl {
        conn.execute(
            "insert into watchlist
            (name, option) values (?1, ?2)",
            &[&key.to_string(), &val.to_string()],
        )?;
    }

    // return an Ok value
    Ok(())
}

/// Sets the settings directory using User Variables.
pub fn settings_dir() -> String {
    // Get the config dir for the system
    let mut set_dir = dirs::config_dir().unwrap();

    // Push the needed values for nyaadle
    set_dir.push("nyaadle");
    set_dir.push("nyaadle");
    set_dir.set_extension("db");

    // Create the path string
    let set_dir = String::from(set_dir.to_str().unwrap());

    // Return the path
    set_dir
}

/// Function that returns the values for the directories.
/// This allows us to read the settings set by the user.
pub fn get_settings(key: &String) -> rusqlite::Result<String> {
    // Get the settings path
    let set_dir = settings_dir();

    // Establish a connection to the database
    let conn = Connection::open(set_dir)?;
    // Prepare the query
    let mut stmt = conn.prepare("SELECT path FROM directories WHERE option = :name")?;
    // execute the query
    let rows = stmt.query_map_named(named_params! { ":name": &key }, |row| row.get(0))?;

    // push the returned value into a String
    let mut dir = String::new();
    for dir_result in rows {
        dir = dir_result.unwrap();
    }
    // Return the directory path
    Ok(dir)
}

/// Function that returns the values inside the watchlist table
fn read_watch_list(set_path: &String) -> rusqlite::Result<Vec<Watchlist>> {
    // Open the database
    let conn = Connection::open(set_path)?;

    // Prepare the query for the watchlist
    let mut stmt = conn.prepare("SELECT * FROM watchlist")?;
    // Execute the query. Returns the values into a Watchlist Struct
    let stored_watch_list = stmt.query_map(NO_PARAMS, |row| {
        Ok(Watchlist {
            title: row.get(1)?,
            option: row.get(2)?,
        })
    })?;
    // Push the returned values into a Vector
    let mut watch_list = Vec::new();
    for item in stored_watch_list {
        watch_list.push(item?)
    }
    // Return the watchlist
    Ok(watch_list)
}

/// Checks if the `target` has been already downloaded and archived
/// Returns either `Found` or `Empty`
fn archive_check(target: &str, archive_dir: &String) -> String {
    let dir = archive_dir;
    let response = reqwest::get(target).unwrap();
    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");
    let fname = format!("{}/{}", dir, fname);
    let path = Path::new(&fname);
    match path.exists() {
        true => String::from("Found"),
        false => String::from("Empty"),
    }
}

/// Function that takes in a link and downloads it to the specified path.
/// Returns either an `Ok` or an `Err`.
fn downloader(target: &str) -> Result<()> {
    // Get the download dir from the Settings.toml file
    let dl_dir = get_settings(&String::from("dl-dir")).unwrap();
    let archive_dir = get_settings(&String::from("ar-dir")).unwrap();

    // Check if the download/archive location exists
    if Path::new(&dl_dir).exists() == false {
        std::fs::create_dir_all(Path::new(&dl_dir)).expect("Failed to create directory");
    }
    if Path::new(&archive_dir).exists() == false {
        std::fs::create_dir_all(Path::new(&archive_dir)).expect("Failed to create directory");
    }

    let check = archive_check(&target, &archive_dir);
    if check == "Found" {
        println!("File Found. Skipping Download");
        return Ok(());
    } else {
        // Normal download location
        let mut response = reqwest::get(target)?;
        let mut dest = {
            let fname = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.bin");

            println!("file to download: '{}'", fname);
            let fname = format!("{}/{}", dl_dir, fname);
            println!("will be located under: '{:?}'", fname);
            File::create(fname)?
        };
        copy(&mut response, &mut dest)?;

        // The archive function
        let mut response = reqwest::get(target)?;
        let mut archive = {
            let fname = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.bin");

            let fname = format!("{}/{}", archive_dir, fname);
            File::create(fname)?
        };
        copy(&mut response, &mut archive)?;

        Ok(())
    }
}

/// Initializes the download function then passes on the target link
/// to the downloader function
fn download_logic(item: &rss::Item) {
    // Get the link of the item
    let title = item.title().expect("Failed to extract title");
    println!("Downloading {}", title);
    let link = item.link();
    let target = match link {
        Some(link) => link,
        _ => return,
    };
    // Download the given link
    let result = downloader(target);
    match result {
        Ok(_) => println!("Success.\n"),
        Err(_) => println!("An Error Occurred.\n"),
    }
}

/// Function that parses the nyaa.si website then compares it against a
/// file containing the watch list of anime to download.
///
/// If an item title matches the watch list, it invokes the `download` function.
pub fn feed_parser() {
    // Create a channel for the rss feed and return a vector of items.
    let channel =
        Channel::from_url("https://nyaa.si/?page=rss").expect("Unable to connect to website");
    let items = channel.into_items();

    // Read the watchlist from the database
    let set_dir = settings_dir();
    let watch_list = read_watch_list(&set_dir).expect("Failed to unpack vectors");

    // Execute the main logic
    nyaadle_logic(items, watch_list, set_dir);
}

/// Main logic for the function.
/// The function iterates on the Vector `watch_list` and compares it to the `items` returned by the website.
/// This function also checks for the download option that is set by the user.
/// There can be two download options:
/// - A resolution number. This is used for video items.
///     Example: `1080p`, `720p`, `480p`
/// - `non-vid`. This is used for other items such as Books, Software, or Audio.
pub fn nyaadle_logic(items: Vec<rss::Item>, watch_list: Vec<Watchlist>, set_dir: String) {
    let non_opt = String::from("non-vid");
    println!("Checking watch-list...");
    for anime in watch_list {
        // Transform anime into a string so it would be usable in the comparison.
        let title = anime.title;
        let option = anime.option;
        if &title == "" {
            println!("Please set a watch-list in the config file in: {}", set_dir);
        } else if &title == "Skip" {
            println!("Skipping 1080p check.\n");
            continue;
        } else {
            println!("Checking for {}", &title);
            // Iterate in the array items
            for item in &items {
                // Compare the 'title' and the 'item' to see if it's in the watch-list
                let check = item.title().expect("Failed to extract Post title");
                if check.contains(&title) {
                    if option == non_opt {
                        download_logic(item);
                    } else if option == String::from("") {
                        println!(
                            "Please set download option in the config file: {}",
                            &set_dir
                        );
                    } else {
                        if check.contains(&option) {
                            println!("Selecting {}p version", &option);
                            download_logic(item);
                        } else {
                            println!("Invalid download option. Please set a valid option in the config file: {}", &set_dir);
                        }
                    }
                }
            }
        }
    }
}
