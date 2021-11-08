use rusqlite::{named_params, params, Connection, NO_PARAMS};
use std::fs::File;
use std::path::Path;

/// Settings Struct
struct Settings {
    dl_key: String,
    dl_val: String,
    ar_key: String,
    ar_val: String,
    url_key: String,
    url_val: String,
    log_key: String,
    log_val: String,
}
/// Public Watchlist Struct
#[derive(Clone, Debug)]
pub struct Watchlist {
    pub id: i32,
    pub title: String,
    pub option: String,
}

impl Settings {
    // Default Settings
    fn default() -> Settings {
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

        let mut log_path = dirs::config_dir().unwrap();
        log_path.push("nyaadle");
        log_path.push("nyaadle");
        log_path.set_extension("log");
        let log_path = String::from(log_path.to_str().unwrap());

        Settings {
            dl_key: String::from("dl-dir"),
            dl_val: dl_dir,
            ar_key: String::from("ar-dir"),
            ar_val: ar_dir,
            url_key: String::from("url"),
            url_val: String::from("https://nyaa.si/?page=rss"),
            log_key: String::from("log"),
            log_val: log_path,
        }
    }
}

impl Watchlist {
    fn new() -> Watchlist {
        Watchlist {
            id: 0,
            title: String::from(""),
            option: String::from(""),
        }
    }
    fn build(mut self, id: i32, title: String, option: String) -> Watchlist {
        self.id = id;
        self.title = title;
        self.option = option;
        self
    }

    // Default Watchlist
    fn default() -> Watchlist {
        Watchlist {
            id: 0,
            title: String::from(""),
            option: String::from("non-vid"),
        }
    }
}

/// Function that returns the values inside the watchlist table
pub fn read_watch_list(set_path: &str) -> rusqlite::Result<Vec<Watchlist>> {
    // Open the database
    let conn = Connection::open(set_path)?;

    // Prepare the query for the watchlist
    let mut stmt = conn.prepare("SELECT * FROM watchlist")?;
    // Execute the query. Returns the values into a Watchlist Struct
    let stored_watch_list = stmt.query_map(NO_PARAMS, |row| {
        Ok(Watchlist::new().build(row.get(0)?, row.get(1)?, row.get(2)?))
    })?;
    // Push the returned values into a Vector
    let mut watch_list = Vec::new();
    for item in stored_watch_list {
        watch_list.push(item?)
    }
    // Return the watchlist
    Ok(watch_list)
}

/// Checks if the config directory exists and then creates it if it's not found.
pub fn write_settings() {
    let default_set = Settings::default();
    let default_wl = Watchlist::default();
    let set_file = settings_dir();

    let mut directory = dirs::config_dir().unwrap();
    directory.push("nyaadle");

    let directory = String::from(directory.to_str().unwrap());

    // If the settings file doesn't exist, create it.
    if !Path::new(&set_file).exists() {
        println!("nyaadle.db not found. Creating it right now.");
        // create directory
        if !Path::new(&directory).exists() {
            std::fs::create_dir(&directory).expect("Unable to create directory");
        }
        // Create nyaadle.db and add dl-dir
        let db_conn = db_create(&set_file);
        let db_ar_write = db_write_dir(&set_file, &default_set.ar_key, &default_set.ar_val);
        let db_dl_write = db_write_dir(&set_file, &default_set.dl_key, &default_set.dl_val);
        let db_url_write = db_write_dir(&set_file, &default_set.url_key, &default_set.url_val);
        let db_log_write = db_write_dir(&set_file, &default_set.log_key, &default_set.log_val);
        let db_log_file = File::create(&default_set.log_val);
        match db_log_file {
            Ok(_) => println!("Created log file."),
            Err(_) => println!("Failed to create log file"),
        }

        // Append watch-list to nyaadle.db
        let db_wl_write = db_write_wl(&set_file, &default_wl.title, &default_wl.option);
        if db_conn == Ok(())
            && db_ar_write == Ok(())
            && db_dl_write == Ok(())
            && db_wl_write == Ok(())
            && db_url_write == Ok(())
            && db_log_write == Ok(())
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
fn db_create(set_path: &str) -> rusqlite::Result<()> {
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
    // Create the item_tracker table
    conn.execute(
        "create table if not exists item_tracker (
            id integer primary key,
            item blob not null unique,
            latest blob not null unique)
            ",
        NO_PARAMS,
    )?;

    Ok(())
}

/// Funtion to write the directory values to the directories table
pub fn db_write_dir(set_path: &str, dir_key: &str, dir_val: &str) -> rusqlite::Result<()> {
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

/// Function that updates the directory in the database
pub fn update_write_dir(set_path: &str, dir_key: &str, dir_val: &str) -> rusqlite::Result<()> {
    // Collect the directory values
    let mut dir = std::collections::HashMap::new();
    dir.insert(dir_key, dir_val);

    // Establish a connection to the database
    let conn = Connection::open(&set_path)?;

    let mut stmt = conn.prepare("select path from directories where option = (?1)")?;
    let mut rows = stmt.query(params![&dir_key])?;

    let mut num_match = 0;

    while let Some(_rows) = rows.next()? {
        num_match += 1;
    }
    if num_match != 0 {
        // Insert the values into the table
        for (key, val) in &dir {
            conn.execute(
                "update directories set path = (?2)
                where option = (?1)",
                &[&key.to_string(), &val.to_string()],
            )?;
        }
    } else if num_match == 0 {
        conn.execute(
            "insert into directories (option, path) values (?1, ?2)",
            &[&dir_key.to_string(), &dir_val.to_string()],
        )?;
    }
    // return an Ok value
    Ok(())
}
/// Function to write the watchlist values to the watchlist table
pub fn db_write_wl(set_path: &str, wl_key: &str, wl_val: &str) -> rusqlite::Result<()> {
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

/// Deletes the item in the database
pub fn db_delete_wl(set_path: &str, wl_key: &str) -> rusqlite::Result<()> {
    let conn = Connection::open(&set_path)?;

    conn.execute("delete from watchlist where name = (?1)", params![wl_key])?;
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
pub fn get_settings(key: &str) -> rusqlite::Result<String> {
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

pub fn get_url() -> String {
    let url: String;
    if get_settings(&String::from("url")).unwrap() == "" {
        url = String::from("https://nyaa.si/?page=rss");
    } else {
        url = get_settings(&String::from("url")).unwrap();
    }
    url
}

pub fn get_wl() -> Vec<Watchlist> {
    // Read the watchlist from the database
    let set_dir = settings_dir();
    read_watch_list(&set_dir).expect("Failed to unpack vectors")
}

pub fn wl_builder(id: i32, item: String, opt: String) -> Vec<Watchlist> {
    let wl_build = Watchlist::new().build(id, item, opt);
    let wl = vec![wl_build];
    wl
}

pub fn set_check() {
    let set_path = settings_dir();

    if !Path::new(&set_path).exists() {
        write_settings();
    }
}

pub fn get_log() -> String {
    let log_dir: String;
    let mut log_path_default = dirs::config_dir().unwrap();
    log_path_default.push("nyaadle");
    log_path_default.push("nyaadle");
    log_path_default.set_extension("log");

    if get_settings(&String::from("log")).unwrap() == "" {
        log_dir = String::from(log_path_default.to_str().unwrap());
    } else {
        log_dir = get_settings(&String::from("log")).unwrap();
    }
    log_dir
}
pub fn update_tracking(set_path: &str, trck_key: &str, trck_val: &str) -> rusqlite::Result<()> {
    // Collect the directory values
    let mut trck = std::collections::HashMap::new();
    trck.insert(trck_key, trck_val);

    // Establish a connection to the database
    let conn = Connection::open(&set_path)?;

    let mut stmt = conn.prepare("select latest from item_tracker where item = (?1)")?;
    let mut rows = stmt.query(params![&trck_key])?;

    let mut num_match = 0;

    while let Some(_rows) = rows.next()? {
        num_match += 1;
    }
    if num_match != 0 {
        // Insert the values into the table
        for (key, val) in &trck {
            conn.execute(
                "update item_tracker set latest = (?2)
                where item = (?1)",
                &[&key.to_string(), &val.to_string()],
            )?;
        }
    } else if num_match == 0 {
        conn.execute(
            "insert into item_tracker (item, latest) values (?1, ?2)",
            &[&trck_key.to_string(), &trck_val.to_string()],
        )?;
    }
    // return an Ok value
    Ok(())
}
pub fn update_wl(
    set_path: &str,
    wl_old_key: &str,
    wl_new_key: &str,
    wl_opt: &str,
) -> rusqlite::Result<()> {
    // Collect the directory values
    let mut item = std::collections::HashMap::new();
    item.insert(wl_old_key, wl_new_key);

    // Establish a connection to the database
    let conn = Connection::open(&set_path)?;

    let mut stmt = conn.prepare("select name from watchlist where name = (?1)")?;
    let mut rows = stmt.query(params![&wl_old_key])?;

    let mut num_match = 0;

    while let Some(_rows) = rows.next()? {
        num_match += 1;
    }
    if num_match != 0 {
        // Insert the values into the table
        for (_key, _val) in &item {
            conn.execute(
                "update watchlist set name = (?2), option = (?3)
                where name = (?1)",
                &[
                    &wl_old_key.to_string(),
                    &wl_new_key.to_string(),
                    &wl_opt.to_string(),
                ],
            )?;
            conn.execute(
                "update item_tracker set item = (?2) where item = (?1)",
                &[&wl_old_key.to_string(), &wl_new_key.to_string()],
            )?;
        }
    } else if num_match == 0 {
        conn.execute(
            "insert into watchlist (name, option) values (?1, ?2)",
            &[&wl_new_key.to_string(), &wl_opt.to_string()],
        )?;
    }
    // return an Ok value
    Ok(())
}
pub fn get_tracking(key: &str) -> rusqlite::Result<String> {
    // Get the settings path
    let set_dir = settings_dir();
    db_create(&set_dir)?;

    // Establish a connection to the database
    let conn = Connection::open(set_dir)?;
    // Prepare the query
    let mut stmt = conn.prepare("SELECT latest FROM item_tracker WHERE item = :name")?;
    // execute the query
    let rows = stmt.query_map_named(named_params! { ":name": &key }, |row| row.get(0))?;

    // push the returned value into a String
    let mut trck = String::new();
    for trck_result in rows {
        trck = trck_result.unwrap();
    }
    // Return the directory path
    Ok(trck)
}
pub fn arg_set(key: &str, value: &str) {
    let set_file = settings_dir();
    update_write_dir(&set_file, &key, &value).expect("Failed to write to database.");
    match key {
        "dl-dir" => println!("Updated Download directory to \"{}\"", &value),
        "ar-dir" => println!("Updated Archive directory to \"{}\"", &value),
        "url" => println!("Updated RSS Feed URL to \"{}\"", &value),
        "log" => println!("Updated log file location to \"{}\"", &value),
        _ => println!("Unknown key value."),
    };
}
pub fn arg_get_set(key: &str) {
    let value = get_settings(&key).expect("Unable to get specified setting.");
    match key {
        "dl-dir" => println!("Download Directory: {}", value),
        "ar-dir" => println!("Archive Directory: {}", value),
        "url" => println!("RSS Feed URL: {}", value),
        "log" => println!("Log File Path: {}", value),
        _ => println!("Setting not found."),
    }
}
