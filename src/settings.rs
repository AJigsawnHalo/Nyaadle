use rusqlite::{named_params, params, Connection};
use std::fs::File;
use std::path::Path;
use time::format_description;
use time::OffsetDateTime;

pub const CURRENT_DB_VERSION: &str = "3.0";

/// Settings Struct
struct Settings {
    dl_key: String,
    dl_val: String,
    ar_key: String,
    ar_val: String,
    url_val: String,
    log_key: String,
    log_val: String,
    ver_key: String,
    ver_val: String,
    whkurl_key: String,
    whkurl_val: String,
}

/// Public Watchlist Struct
#[derive(Clone, Debug)]
pub struct Watchlist {
    pub id: i32,
    pub title: String,
    pub option: String,
    pub feed_id: i32,
}

///Public Log Struct
#[derive(Clone, Debug)]
pub struct Log {
    pub id: i32,
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct Feed {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub is_default: bool,
}

impl Settings {
    fn default() -> Settings {
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
            url_val: String::from("https://nyaa.si/?page=rss"),
            log_key: String::from("log"),
            log_val: log_path,
            ver_key: String::from("db-ver"),
            ver_val: String::from("2.0"),
            whkurl_key: String::from("webhk_url"),
            whkurl_val: String::from(""),
        }
    }
}

impl Watchlist {
    pub fn new() -> Watchlist {
        Watchlist {
            id: 0,
            title: String::from(""),
            option: String::from(""),
            feed_id: 0,
        }
    }

    pub fn build(mut self, id: i32, title: String, option: String, feed_id: i32) -> Watchlist {
        self.id = id;
        self.title = title;
        self.option = option;
        self.feed_id = feed_id;
        self
    }

    fn default() -> Watchlist {
        Watchlist {
            id: 0,
            title: String::from(""),
            option: String::from("non-vid"),
            feed_id: 0,
        }
    }
}

/// Returns the path to the nyaadle database.
pub fn settings_dir() -> String {
    let mut set_dir = dirs::config_dir().unwrap();
    set_dir.push("nyaadle");
    set_dir.push("nyaadle");
    set_dir.set_extension("db");
    String::from(set_dir.to_str().unwrap())
}

/// Opens a connection to the nyaadle database.
/// Used by main to create the shared connection, and by tui.rs for
/// its interactive callbacks which cannot hold a borrowed reference.
pub fn open_conn() -> rusqlite::Result<Connection> {
    let conn = Connection::open(settings_dir())?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    Ok(conn)
}

/// Creates the database tables if they don't already exist.
fn db_create(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS directories (
            option TEXT PRIMARY KEY,
            path   TEXT NOT NULL UNIQUE)",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS feeds (
            id         INTEGER PRIMARY KEY,
            name       TEXT NOT NULL UNIQUE,
            url        TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0)",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS watchlist (
            id      INTEGER PRIMARY KEY,
            name    TEXT NOT NULL,
            option  TEXT NOT NULL,
            feed_id INTEGER NOT NULL REFERENCES feeds(id),
            UNIQUE(name, feed_id))", // Allows tracking the same item name across different feeds
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS item_tracker (
            id     INTEGER PRIMARY KEY,
            item   BLOB NOT NULL UNIQUE,
            latest BLOB NOT NULL UNIQUE)",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id        INTEGER PRIMARY KEY,
            timestamp TEXT NOT NULL,
            level     TEXT NOT NULL,
            message   TEXT NOT NULL)",
        [],
    )?;
    Ok(())
}

/// Checks if the config directory and database exist; creates them if not.
/// Opens its own connection since this runs before the shared conn exists.
pub fn write_settings() {
    let default_set = Settings::default();
    let default_wl = Watchlist::default();
    let set_file = settings_dir();

    let mut directory = dirs::config_dir().unwrap();
    directory.push("nyaadle");
    let directory = String::from(directory.to_str().unwrap());

    if !Path::new(&set_file).exists() {
        println!("nyaadle.db not found. Creating it right now.");

        if !Path::new(&directory).exists() {
            std::fs::create_dir(&directory).expect("Unable to create directory");
        }

        let conn = Connection::open(&set_file).expect("Failed to create database.");

        let db_conn = db_create(&conn);
        let db_ar_write = db_write_dir(&conn, &default_set.ar_key, &default_set.ar_val);
        let db_dl_write = db_write_dir(&conn, &default_set.dl_key, &default_set.dl_val);
        let db_log_write = db_write_dir(&conn, &default_set.log_key, &default_set.log_val);
        let db_whk_write = db_write_dir(&conn, &default_set.whkurl_key, &default_set.whkurl_val);
        let db_ver_write = db_write_dir(&conn, &default_set.ver_key, CURRENT_DB_VERSION);
        let db_feed_write = db_write_feed(&conn, "Default", &default_set.url_val, true);
        let db_wl_write = match get_default_feed_id(&conn) {
            Ok(feed_id) => db_write_wl(&conn, &default_wl.title, &default_wl.option, feed_id),
            Err(e) => Err(e),
        };

        match File::create(&default_set.log_val) {
            Ok(_) => println!("Created log file."),
            Err(_) => println!("Failed to create log file."),
        }

        let base_ok = db_conn == Ok(())
            && db_ar_write == Ok(())
            && db_dl_write == Ok(())
            && db_wl_write == Ok(())
            && db_feed_write == Ok(())
            && db_ver_write == Ok(())
            && db_log_write == Ok(());

        #[cfg(feature = "discord")]
        let base_ok = base_ok && db_whk_write == Ok(());
        #[cfg(not(feature = "discord"))]
        let _ = db_whk_write;

        if base_ok {
            println!("nyaadle.db created.");
            println!("You can change settings by running 'nyaadle set' or 'nyaadle tui'.");
        } else {
            println!("Failed to create nyaadle.db");
        }
    }
}

/// Ensures the database exists, creating it with defaults if not.
/// Called at program start before the shared connection is opened.
pub fn set_check() {
    if !Path::new(&settings_dir()).exists() {
        write_settings();
    }
}

// ── Hot-path functions: take &Connection ────────────────────────────────────
// These are called from the cron-triggered feed_parser path and share the
// single connection opened in main(). Do not open new connections here.

/// Reads the value for `key` from the directories table.
pub fn get_settings(conn: &Connection, key: &str) -> rusqlite::Result<String> {
    let mut stmt = conn.prepare("SELECT path FROM directories WHERE option = :name")?;
    let rows = stmt.query_map(named_params! { ":name": key }, |row| row.get(0))?;
    let mut dir = String::new();
    for dir_result in rows {
        dir = dir_result.unwrap_or_default();
    }
    Ok(dir)
}

/// Returns the RSS feed URL, falling back to the nyaa.si default.
pub fn get_url(conn: &Connection) -> String {
    conn.query_row(
        "SELECT url FROM feeds WHERE is_default = 1 LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_else(|_| String::from("https://nyaa.si/?page=rss"))
}

/// Returns the log file path, falling back to the default location.
pub fn get_log(conn: &Connection) -> String {
    get_settings(conn, "log")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let mut p = dirs::config_dir().unwrap();
            p.push("nyaadle");
            p.push("nyaadle");
            p.set_extension("log");
            String::from(p.to_str().unwrap())
        })
}

/// Returns the full watchlist from the database.
pub fn get_wl(conn: &Connection) -> Vec<Watchlist> {
    read_watch_list(conn).expect("Failed to read watchlist")
}

/// Builds a single-entry watchlist vector, used for one-shot parses.
pub fn wl_builder(id: i32, item: String, opt: String) -> Vec<Watchlist> {
    vec![Watchlist::new().build(id, item, opt, 0)]
}

/// Returns the values inside the watchlist table.
pub fn read_watch_list(conn: &Connection) -> rusqlite::Result<Vec<Watchlist>> {
    let mut stmt = conn.prepare("SELECT * FROM watchlist")?;
    let stored = stmt.query_map([], |row| {
        Ok(Watchlist::new().build(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    let mut watch_list = Vec::new();
    for item in stored {
        watch_list.push(item?);
    }
    Ok(watch_list)
}

pub fn read_feeds(conn: &Connection) -> rusqlite::Result<Vec<Feed>> {
    let mut stmt = conn.prepare("SELECT id, name, url, is_default FROM feeds")?;
    let stored = stmt.query_map([], |row| {
        Ok(Feed {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            is_default: row.get::<_, i32>(3)? != 0,
        })
    })?;
    let mut feeds = Vec::new();
    for feed in stored {
        feeds.push(feed?);
    }
    Ok(feeds)
}

pub fn get_default_feed_id(conn: &Connection) -> rusqlite::Result<i32> {
    conn.query_row(
        "SELECT id FROM feeds WHERE is_default = 1 LIMIT 1",
        [],
        |row| row.get(0),
    )
}

pub fn db_write_feed(
    conn: &Connection,
    name: &str,
    url: &str,
    is_default: bool,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO feeds (name, url, is_default) VALUES (?1, ?2, ?3)",
        params![name, url, is_default as i32],
    )?;
    Ok(())
}

pub fn update_default_feed_url(conn: &Connection, new_url: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE feeds SET url = ?1 WHERE is_default = 1",
        params![new_url],
    )?;
    Ok(())
}

/// Writes a directory key/value pair to the directories table.
pub fn db_write_dir(conn: &Connection, dir_key: &str, dir_val: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO directories (option, path) VALUES (?1, ?2)",
        params![dir_key, dir_val],
    )?;
    Ok(())
}

/// Updates an existing directory entry, or inserts it if not present.
pub fn update_write_dir(conn: &Connection, dir_key: &str, dir_val: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO directories (option, path) VALUES (?1, ?2)
         ON CONFLICT(option) DO UPDATE SET path = excluded.path",
        params![dir_key, dir_val],
    )?;
    Ok(())
}

/// Writes a new entry to the watchlist table.
pub fn db_write_wl(
    conn: &Connection,
    wl_key: &str,
    wl_val: &str,
    feed_id: i32,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO watchlist (name, option, feed_id) VALUES (?1, ?2, ?3)",
        params![wl_key, wl_val, feed_id],
    )?;
    Ok(())
}

/// Updates an existing watchlist entry by ID, or inserts it if not present.
pub fn update_wl(
    conn: &Connection,
    wl_new_key: &str,
    wl_opt: &str,
    id: &str,
) -> rusqlite::Result<()> {
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM watchlist WHERE id = ?1",
        params![id],
        |row| row.get::<_, i64>(0),
    )? > 0;

    if exists {
        // Keep item_tracker in sync when the watchlist title changes
        let old_title: String = conn.query_row(
            "SELECT name FROM watchlist WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        conn.execute(
            "UPDATE item_tracker SET item = ?2 WHERE item = ?1",
            params![old_title, wl_new_key],
        )?;
        conn.execute(
            "UPDATE watchlist SET name = ?2, option = ?3 WHERE id = ?1",
            params![id, wl_new_key, wl_opt],
        )?;
    } else {
        let feed_id = get_default_feed_id(conn)?;
        conn.execute(
            "INSERT INTO watchlist (name, option, feed_id) VALUES (?1, ?2, ?3)",
            params![wl_new_key, wl_opt, feed_id],
        )?;
    }
    Ok(())
}

/// Deletes a watchlist entry by ID.
pub fn db_delete_wl(conn: &Connection, wl_key: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM watchlist WHERE id = ?1", params![wl_key])?;
    Ok(())
}

/// Returns the last-seen item for a given watchlist title.
pub fn get_tracking(conn: &Connection, key: &str) -> rusqlite::Result<String> {
    let mut stmt = conn.prepare("SELECT latest FROM item_tracker WHERE item = :name")?;
    let rows = stmt.query_map(named_params! { ":name": key }, |row| row.get(0))?;
    let mut trck = String::new();
    for trck_result in rows {
        trck = trck_result.unwrap();
    }
    Ok(trck)
}

/// Updates the last-seen item for a watchlist title, inserting if not present.
pub fn update_tracking(conn: &Connection, trck_key: &str, trck_val: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO item_tracker (item, latest) VALUES (?1, ?2)
         ON CONFLICT(item) DO UPDATE SET latest = excluded.latest",
        params![trck_key, trck_val],
    )?;
    Ok(())
}

/// Sets a key in the directories table from the command line.
pub fn arg_set(conn: &Connection, key: &str, value: &str) {
    match key {
        "url" => {
            update_default_feed_url(conn, value).expect("Failed to write to database.");
            println!("Updated RSS Feed URL to \"{}\"", value);
        }
        "dl-dir" => {
            update_write_dir(conn, key, value).expect("Failed to write to database.");
            println!("Updated Download directory to \"{}\"", value);
        }
        "ar-dir" => {
            update_write_dir(conn, key, value).expect("Failed to write to database.");
            println!("Updated Archive directory to \"{}\"", value);
        }
        "log" => {
            update_write_dir(conn, key, value).expect("Failed to write to database.");
            println!("Updated log file location to \"{}\"", value);
        }
        #[cfg(feature = "discord")]
        "webhk_url" => {
            update_write_dir(conn, key, value).expect("Failed to write to database.");
            println!("Updated Discord webhook URL to \"{}\"", value);
        }
        _ => println!("Unknown key."),
    }
}

/// Prints a key's current value from the directories table.
pub fn arg_get_set(conn: &Connection, key: &str) {
    match key {
        "url" => println!("RSS Feed URL: {}", get_url(conn)),
        "dl-dir" => println!(
            "Download Directory: {}",
            get_settings(conn, key).unwrap_or_default()
        ),
        "ar-dir" => println!(
            "Archive Directory: {}",
            get_settings(conn, key).unwrap_or_default()
        ),
        "log" => println!(
            "Log File Path: {}",
            get_settings(conn, key).unwrap_or_default()
        ),
        "db-ver" => println!(
            "Database version: {}",
            get_settings(conn, key).unwrap_or_default()
        ),
        "webhk_url" => {
            let value = get_settings(conn, key).unwrap_or_default();
            #[cfg(feature = "discord")]
            println!("Discord Webhook URL: {}", value);
            #[cfg(not(feature = "discord"))]
            println!("Discord Webhook URL: {} (Discord feature disabled)", value);
        }
        _ => unreachable!("Setting not found."),
    }
}

/// Ensures the db-ver entry exists, then migrates up to CURRENT_DB_VERSION
/// if needed.
pub fn get_db_ver(conn: &Connection) -> rusqlite::Result<()> {
    let default = Settings::default();
    conn.execute(
        "INSERT OR IGNORE INTO directories (option, path) VALUES (?1, ?2)",
        params![default.ver_key, default.ver_val],
    )?;

    let version = get_settings(conn, "db-ver").unwrap_or_default();
    if version == CURRENT_DB_VERSION {
        // Current version, ensure all tables exist
        db_create(conn)?;
        // Pre-release "3.0" databases created before multi-feed landed
        // (e.g. from the earlier logging-refactor session) will already
        // say db-ver = "3.0" but won't have feed_id on watchlist. Since
        // 3.0 was never actually released, patch these in place instead
        // of introducing a new version number.
        if !watchlist_has_feed_id(conn)? {
            backfill_feed_id(conn)?;
        }
    } else {
        migrate(conn)?;
    }
    Ok(())
}

/// Runs database migrations, stepping through versions one at a time
/// until the database is at CURRENT_DB_VERSION or the version is
/// unrecognized. Add new match arms here as new versions are released.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    // Create a backup of the live database file right before migrating
    let db_path = settings_dir();
    let path = Path::new(&db_path);
    if path.exists() {
        let backup_path = format!("{}.bak", db_path);
        println!(
            "Backing up database to {} before running migrations...",
            backup_path
        );
        if let Err(e) = std::fs::copy(path, &backup_path) {
            println!(
                "Warning: Pre-migration backup failed: {}. Attempting migration anyway.",
                e
            );
        }
    }

    loop {
        let version = get_settings(conn, "db-ver").unwrap_or_default();
        match version.as_str() {
            "2.0" => {
                db_create(conn)?;
                if !watchlist_has_feed_id(conn)? {
                    backfill_feed_id(conn)?;
                }
                update_write_dir(conn, "db-ver", CURRENT_DB_VERSION)?;
                println!("Migrated database to {}", CURRENT_DB_VERSION);
            }
            v if v == CURRENT_DB_VERSION => break,
            _ => {
                db_create(conn)?;
                break;
            }
        }
    }
    Ok(())
}

/// Returns true if the watchlist table already has a feed_id column.
fn watchlist_has_feed_id(conn: &Connection) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare("PRAGMA table_info(watchlist)")?;
    let cols = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for col in cols {
        if col? == "feed_id" {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Adds feed_id to an existing watchlist table and backfills every row
/// to a "Default" feed, seeding that feed from the legacy `url` entry
/// in `directories` if one doesn't already exist. Used both for the
/// 2.0 -> 3.0 migration and for patching pre-multi-feed 3.0 dev databases.
fn backfill_feed_id(conn: &Connection) -> rusqlite::Result<()> {
    // Ensure the feeds table exists.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS feeds (
            id         INTEGER PRIMARY KEY,
            name       TEXT NOT NULL UNIQUE,
            url        TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0)",
        [],
    )?;

    // Reuse the existing default feed if one is already there (e.g. this
    // is a re-run), otherwise seed one from the legacy directories.url.
    let default_feed_id = match get_default_feed_id(conn) {
        Ok(id) => id,
        Err(_) => {
            let old_url = get_settings(conn, "url")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| String::from("https://nyaa.si/?page=rss"));
            db_write_feed(conn, "Default", &old_url, true)?;
            get_default_feed_id(conn)?
        }
    };

    // SQLite requires a constant DEFAULT for a NOT NULL ADD COLUMN, so
    // default to 0 and immediately overwrite with the real id.
    conn.execute(
        "ALTER TABLE watchlist ADD COLUMN feed_id INTEGER NOT NULL DEFAULT 0",
        [],
    )?;
    conn.execute(
        "UPDATE watchlist SET feed_id = ?1",
        params![default_feed_id],
    )?;

    // `url` no longer lives in directories.
    conn.execute("DELETE FROM directories WHERE option = 'url'", [])?;

    Ok(())
}

/// Writes a log entry to the logs table.
pub fn write_log(conn: &Connection, level: &str, message: &str) -> rusqlite::Result<()> {
    let format = format_description::parse(
        "[year]-[month repr:short]-[day] [weekday repr:short] [hour]:[minute]:[second]",
    )
    .unwrap();
    let timestamp = OffsetDateTime::now_local()
        .unwrap_or(OffsetDateTime::now_utc())
        .format(&format)
        .unwrap_or_else(|_| String::from("unknown"));

    conn.execute(
        "INSERT INTO logs (timestamp, level, message) VALUES (?1, ?2, ?3)",
        params![timestamp, level, message],
    )?;
    Ok(())
}

/// Reads log entries from thelog table.
pub fn read_logs(conn: &Connection) -> rusqlite::Result<Vec<Log>> {
    let mut stmt =
        conn.prepare("SELECT id, timestamp, level, message FROM logs ORDER BY id DESC LIMIT 500")?;
    let stored = stmt.query_map([], |row| {
        Ok(Log {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            level: row.get(2)?,
            message: row.get(3)?,
        })
    })?;
    let mut logs = Vec::new();
    for entry in stored {
        logs.push(entry?);
    }
    Ok(logs)
}
/// Updates the URL of a specific feed by its unique name.
pub fn update_feed_url(conn: &Connection, name: &str, new_url: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE feeds SET url = ?1 WHERE name = ?2",
        params![new_url, name],
    )?;
    Ok(())
}

/// Renames an existing feed tracking channel.
pub fn rename_feed(conn: &Connection, old_name: &str, new_name: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE feeds SET name = ?1 WHERE name = ?2",
        params![new_name, old_name],
    )?;
    Ok(())
}

/// Shifts the global default feed flag safely to a designated target feed.
pub fn set_default_feed(conn: &Connection, name: &str) -> rusqlite::Result<()> {
    // Clear out old defaults first
    conn.execute("UPDATE feeds SET is_default = 0", [])?;
    conn.execute(
        "UPDATE feeds SET is_default = 1 WHERE name = ?1",
        params![name],
    )?;
    Ok(())
}

/// Modifies a watchlist item's core details along with its target feed association.
pub fn update_wl_with_feed(
    conn: &Connection,
    wl_new_key: &str,
    wl_opt: &str,
    feed_id: i32,
    id: &str,
) -> rusqlite::Result<()> {
    let old_title: String = conn.query_row(
        "SELECT name FROM watchlist WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;

    if old_title != wl_new_key {
        conn.execute(
            "UPDATE item_tracker SET item = ?2 WHERE item = ?1",
            params![old_title, wl_new_key],
        )?;
    }

    conn.execute(
        "UPDATE watchlist SET name = ?1, option = ?2, feed_id = ?3 WHERE id = ?4",
        params![wl_new_key, wl_opt, feed_id, id],
    )?;
    Ok(())
}

/// Safely purges a feed, re-routing default configurations and reassigning or purging dependent trackers.
pub fn db_delete_feed(
    conn: &Connection,
    name: &str,
    replacement_default: Option<&str>,
    reassign_feed_name: Option<&str>,
) -> rusqlite::Result<()> {
    let target_id: i32 = conn.query_row(
        "SELECT id FROM feeds WHERE name = ?1",
        params![name],
        |row| row.get(0),
    )?;

    // 1. If we are deleting the current default feed, apply the replacement default flag first
    if let Some(rep_name) = replacement_default {
        conn.execute("UPDATE feeds SET is_default = 0", [])?;
        conn.execute(
            "UPDATE feeds SET is_default = 1 WHERE name = ?1",
            params![rep_name],
        )?;
    }

    // 2. Clear or assign dependent tracking items belonging to this feed channel
    if let Some(reassign_name) = reassign_feed_name {
        let reassign_id: i32 = conn.query_row(
            "SELECT id FROM feeds WHERE name = ?1",
            params![reassign_name],
            |row| row.get(0),
        )?;
        conn.execute(
            "UPDATE watchlist SET feed_id = ?1 WHERE feed_id = ?2",
            params![reassign_id, target_id],
        )?;
    } else {
        // Drop items linked to this specific feed if no alternative routing is requested
        conn.execute(
            "DELETE FROM watchlist WHERE feed_id = ?1",
            params![target_id],
        )?;
    }

    // 3. Purge the target record cleanly
    conn.execute("DELETE FROM feeds WHERE id = ?1", params![target_id])?;
    Ok(())
}
