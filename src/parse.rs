// Parts of this code was adapted from "The Rust Cookbook"
// which can be found at: https://rust-lang-nursery.github.io/rust-cookbook/

use crate::settings;
use crate::settings::Watchlist;
use anyhow::Result;
use rss::Channel;
use rusqlite::Connection;
use std::fs::File;
use std::io::copy;
use std::io::Cursor;
use std::path::Path;

#[cfg(feature = "discord")]
use serenity::builder::ExecuteWebhook;
#[cfg(feature = "discord")]
use serenity::http::Http;
#[cfg(feature = "discord")]
use serenity::model::webhook::Webhook;

/// Extracts the filename from a URL string without making a network request.
fn filename_from_url(url: &str) -> &str {
    url.split('/')
        .last()
        .filter(|s| !s.is_empty())
        .unwrap_or("tmp.bin")
}

/// Checks if the target has already been downloaded and archived.
/// Returns 0 if found (skip), 1 if not found (proceed).
async fn archive_check(
    target: &str,
    archive_dir: &str,
    force: bool,
    conn: &Connection,
) -> Result<u8> {
    if force {
        warn!("Force option enabled.");
        let _ = settings::write_log(conn, "WARN", "Force option enabled");
        return Ok(1);
    }
    let path = Path::new(archive_dir).join(filename_from_url(target));
    Ok(if path.exists() { 0 } else { 1 })
}

/// Downloads the target URL to dl-dir and copies it to ar-dir for dedup tracking.
async fn downloader(conn: &Connection, target: &str, title: &str, force: bool) -> Result<u8> {
    debug!("Reached Downloader");
    let dl_dir = settings::get_settings(conn, "dl-dir")?;
    let archive_dir = settings::get_settings(conn, "ar-dir")?;

    #[cfg(feature = "discord")]
    let wbhk_url = settings::get_settings(conn, "webhk_url").expect("No webhook url set");
    #[cfg(feature = "discord")]
    let http = Http::new("");
    #[cfg(feature = "discord")]
    let wbhk = Webhook::from_url(&http, &wbhk_url)
        .await
        .expect("Failed to get webhook url.");

    if !Path::new(&dl_dir).exists() {
        std::fs::create_dir_all(&dl_dir).expect("Failed to create download directory");
    }
    if !Path::new(&archive_dir).exists() {
        std::fs::create_dir_all(&archive_dir).expect("Failed to create archive directory");
    }

    match archive_check(target, &archive_dir, force, conn).await? {
        1 => {
            let response = reqwest::get(target).await?;

            let fname = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.bin");

            println!("file to download: '{}'", fname);

            let dest_name = format!("{}/{}", dl_dir, fname);
            let archive_name = format!("{}/{}", archive_dir, fname);

            println!("will be located under: '{}'", dest_name);

            let mut dest = File::create(&dest_name)?;
            let mut content = Cursor::new(response.bytes().await?);
            copy(&mut content, &mut dest)?;

            // Copy to archive so future cron runs can detect it by filename
            let mut dest2 = File::open(&dest_name)?;
            let mut archive = File::create(archive_name)?;
            copy(&mut dest2, &mut archive)?;

            info!("Downloaded {}", title);
            let _ = settings::write_log(conn, "INFO", &format!("Downloaded {}", title));

            #[cfg(feature = "discord")]
            {
                let content = format!("Downloaded {}", title);
                let builder = ExecuteWebhook::new().content(content).username("Nyaadle");
                wbhk.execute(&http, false, builder)
                    .await
                    .expect("Failed to execute webhook.");
            }

            Ok(1)
        }
        _ => {
            println!("File Found. Skipping Download.");
            Ok(0)
        }
    }
}

/// Downloads a list of URLs directly, bypassing the watchlist/feed logic.
pub async fn arg_dl(conn: &Connection, links: Vec<String>) -> Result<()> {
    info!("Nyaadle started in download mode.");
    let _ = settings::write_log(conn, "INFO", "Nyaadle started in download mode.");
    let mut num_dl = 0;

    for link in links.iter() {
        if link.is_empty() || link == "\n" {
            break;
        }
        // Pass 0 as the default feed_id here
        if !tracking_check(conn, link.to_string(), link, true, 0) {
            if link.contains("magnet:") {
                match opener::open(link) {
                    Ok(_) => {
                        println!("Opening magnet link...");
                        num_dl += 1;
                    }
                    Err(_) => println!("Error. Path not found."),
                }
                info!("Downloaded magnet link.");
                let _ = settings::write_log(conn, "INFO", "Downloaded magnet link.");
            } else {
                match downloader(conn, link, link, true).await? {
                    1 => num_dl += 1,
                    _ => {}
                }
            }
        }
    }

    if num_dl == 0 {
        debug!("No items downloaded. Nyaadle closed.");
    } else {
        info!("{} items downloaded. Nyaadle closed.", num_dl);
        let _ = settings::write_log(
            conn,
            "INFO",
            &format!("{} items downloaded. Nyaadle closed.", num_dl),
        );
    }
    Ok(())
}

/// Checks the tracking table and updates it. Returns true if already downloaded.
fn tracking_check(conn: &Connection, item: String, wl_title: &str, force: bool, feed_id: i32) -> bool {
    let trck = settings::get_tracking(conn, wl_title, feed_id).expect("Failed to get tracking.");
    if trck == item && !force {
        println!("Item already downloaded. Skipping...");
        true
    } else {
        let _ = settings::update_tracking(conn, wl_title, &item, feed_id).is_ok();
        false
    }
}

/// Resolves the link from an RSS item and dispatches to downloader.
async fn download_logic(
    conn: &Connection,
    item: &rss::Item,
    wl_title: &str,
    force: bool,
    feed_id: i32,
) -> Result<u8> {
    let title = item.title().expect("Failed to extract title");

    if tracking_check(conn, title.to_string(), wl_title, force, feed_id) {
        return Ok(0);
    }
    println!("Downloading {}", title);

    let target = match item.link() {
        Some(link) => link,
        None => return Ok(0),
    };

    if target.contains("magnet:") {
        info!("Downloaded {}", title);
        let _ = settings::write_log(conn, "INFO", &format!("Downloaded {}", title));
        match opener::open(target) {
            Ok(_) => Ok(1),
            Err(_) => Ok(0),
        }
    } else {
        match downloader(conn, target, title, force).await? {
            1 => Ok(1),
            _ => Ok(0),
        }
    }
}

pub async fn arg_parse(
    conn: &Connection,
    force: bool,
    feed: Option<String>,
    item: Option<String>,
    vid_opt: Option<String>,
) -> Result<()> {
    if let (Some(title), Some(opt)) = (item, vid_opt) {
        println!("Parsing for: '{}' with option '{}'", &title, &opt);
        feed_parser(conn, false, force, feed, Some(title), Some(opt)).await?;
        return Ok(());
    }

    feed_parser(conn, false, force, feed, None, None).await?;
    Ok(())
}

/// Fetches and parses the RSS feed then runs the main logic.
/// Pass `check = true` to print matches without downloading.
pub async fn feed_parser(conn: &Connection, check: bool, force: bool, feed_url: Option<String>, item_title: Option<String>, vid_opt: Option<String>) -> Result<()> {
    if check {
        info!("Nyaadle started in checking mode.");
        let _ = settings::write_log(conn, "INFO", "Nyaadle started in checking mode.");
    }

    let master_watchlist = if let (Some(t), Some(o)) = (item_title, vid_opt) {
        // Create a temporary "virtual" watchlist entry for the command line args
        vec![settings::Watchlist {
            id: -1, 
            title: t,
            option: o,
            feed_id: -1,
        }]
    } else {
        settings::read_watch_list(conn).unwrap_or_default()
    };
    if let Some(url) = feed_url {
        println!("Parsing external feed: {}", url);
        let content = reqwest::get(&url).await?.bytes().await?;
        let channel = Channel::read_from(&content[..])?;
        
        // Use the entire watchlist for the external feed
        nyaadle_logic(conn, channel.into_items(), master_watchlist, check, force).await?;
        return Ok(());
    }
    let feeds = settings::read_feeds(conn).unwrap_or_default();
    let mut total_dl: u32 = 0;

    if master_watchlist.is_empty() || master_watchlist.iter().all(|w| w.title.is_empty()) {
        warn!("Watch-list not found.");
        let _ = settings::write_log(conn, "WARN", "Watch-list not found.");
        println!("Please set a watch-list by running 'nyaadle wle --add'");
        return Ok(());
    }

    for feed in feeds {
        let local_watchlist: Vec<Watchlist> = master_watchlist
            .iter()
            .filter(|item| item.feed_id == feed.id || item.id == -1)
            .cloned()
            .collect();

        if local_watchlist.is_empty() {
            continue;
        }

        println!("Connecting to feed channel: {} ({})", feed.name, feed.url);

        let content = match reqwest::get(&feed.url).await {
            Ok(res) => match res.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => {
                    eprintln!(
                        "Failed to stream data from channel '{}'. Skipping...",
                        feed.name
                    );
                    continue;
                }
            },
            Err(_) => {
                eprintln!("Unable to connect to '{}'. Skipping channel...", feed.name);
                continue;
            }
        };

        let channel = match Channel::read_from(&content[..]) {
            Ok(ch) => ch,
            Err(_) => {
                eprintln!(
                    "Failed to parse XML for channel '{}'. Skipping...",
                    feed.name
                );
                continue;
            }
        };

        match nyaadle_logic(conn, channel.into_items(), local_watchlist, check, force).await {
            Ok(count) => total_dl += count as u32,
            Err(e) => error!("Error evaluating channel logic for '{}': {}", feed.name, e),
        }
    }

    if total_dl == 0 {
        debug!("No items downloaded. Nyaadle closed.");
    } else {
        info!("{} total items downloaded. Nyaadle closed.", total_dl);
        let _ = settings::write_log(
            conn,
            "INFO",
            &format!("{} total items downloaded. Nyaadle closed.", total_dl),
        );
    }

    Ok(())
}

/// Iterates the watchlist against RSS feed items and downloads matches.
///
/// Download options:
/// - A resolution string (`1080`, `720`, `480`) for video items.
/// - `non-vid` for non-video items such as books, software, or audio.
pub async fn nyaadle_logic(
    conn: &Connection,
    items: Vec<rss::Item>,
    watch_list: Vec<Watchlist>,
    check: bool,
    force: bool,
) -> Result<u8> {
    let non_opt = "non-vid";
    let mut num_dl: u8 = 0;
    println!("Checking watch-list...\n");

    for anime in &watch_list {
        if anime.option.is_empty() {
            warn!("Download option not found for \"{}\".", anime.title);
            let _ = settings::write_log(
                conn,
                "WARN",
                &format!("Download option not found for \"{}\".", anime.title),
            );
            println!("Please set a download option using 'nyaadle wle --edit'");
            continue;
        }

        println!("Checking for {}", &anime.title);

        // Collect all matching items before acting on any of them
        let matches: Vec<&rss::Item> = items
            .iter()
            .filter(|item| {
                let title = match item.title() {
                    Some(t) => t,
                    None => return false,
                };
                if !title.contains(&anime.title) {
                    return false;
                }
                if anime.option == non_opt {
                    true
                } else {
                    title.contains(&anime.option)
                }
            })
            .collect();

        if matches.is_empty() {
            println!("No matches found for {}\n", &anime.title);
            continue;
        }

        for item in matches {
            let title = item.title().unwrap_or("unknown");
            if check {
                println!("Found {}\n", title);
            } else {
                match download_logic(conn, item, &anime.title, force, anime.feed_id).await? {
                    1 => num_dl += 1,
                    _ => {}
                }
            }
        }
    }

    if num_dl == 0 {
        debug!("No items downloaded. Nyaadle closed.");
    } else {
        info!("{} items downloaded. Nyaadle closed.", num_dl);
        let _ = settings::write_log(
            conn,
            "INFO",
            &format!("{} items downloaded. Nyaadle closed.", num_dl),
        );
    }

    Ok(num_dl)
}
