use dirs;
use rusqlite::{named_params, params, Connection, NO_PARAMS, };
use std::path::Path;

/// Settings Struct
struct Settings {
    dl_key: String,
    dl_val: String,
    ar_key: String,
    ar_val: String,
    url_key: String,
    url_val: String,
}
/// Public Watchlist Struct
#[derive(Clone, Debug)]
pub struct Watchlist {
    pub title: String,
    pub option: String,
}

/// Function that returns the values inside the watchlist table
pub fn read_watch_list(set_path: &String) -> rusqlite::Result<Vec<Watchlist>> {
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
        url_key: String::from("url"),
        url_val: String::from("https://nyaa.si/?page=rss",) 
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
        let db_ar_write = db_write_dir(&set_file, &default_set.ar_key, &default_set.ar_val);
        let db_dl_write = db_write_dir(&set_file, &default_set.dl_key, &default_set.dl_val);
        let db_url_write = db_write_dir(&set_file, &default_set.url_key, &default_set.url_val);

        // Append watch-list to nyaadle.db
        let db_wl_write = db_write_wl(&set_file, &default_wl.title, &default_wl.option);
        if db_conn == Ok(())
            && db_ar_write == Ok(())
            && db_dl_write == Ok(())
            && db_wl_write == Ok(())
            && db_url_write == Ok(())
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
pub fn db_write_dir(set_path: &String, dir_key: &String, dir_val: &String) -> rusqlite::Result<()> {
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
pub fn update_write_dir(
    set_path: &String,
    dir_key: &String,
    dir_val: &String,
) -> rusqlite::Result<()> {
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
        println!("num_match = '{}'", &num_match);
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
        conn.execute("insert into directories (option, path) values (?1, ?2)", &[&dir_key.to_string(), &dir_val.to_string()])?; 
    
    }
    // return an Ok value
    Ok(())
}
/// Function to write the watchlist values to the watchlist table
pub fn db_write_wl(set_path: &String, wl_key: &String, wl_val: &String) -> rusqlite::Result<()> {
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
pub fn db_delete_wl(set_path: &String, wl_key: &String) -> rusqlite::Result<()> {
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

pub fn set_check()  {
    let set_path = settings_dir();
    if Path::new(&set_path).exists(){
        return
    } else {
        write_settings();
        return
    }
}