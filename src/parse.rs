// Parts of this code was adapted from "The Rust Cookbook"
// which can be found at: https://rust-lang-nursery.github.io/rust-cookbook/

use crate::settings;
use crate::settings::Watchlist;
use opener;
use rss::Channel;
use std::fs::File;
use std::io::copy;
use std::path::Path;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
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
fn downloader(target: &str, wl_title: &String) -> Result<u8> {
    // Get the download dir from the Settings.toml file
    let dl_dir = settings::get_settings(&String::from("dl-dir")).unwrap();
    let archive_dir = settings::get_settings(&String::from("ar-dir")).unwrap();

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
        return Ok(0);
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
        info!("Downloaded {}", wl_title);

        Ok(1)
    }
}

pub fn arg_dl(links: Vec<String>) {
    info!("Nyaadle started in download mode.");
    let mut num_dl = 0;
    for link in links.iter() {
        if link == "" {
            println!("No link found. Exiting...");
            return;
        } else if link == "\n" {
            break;
        } else if link.contains("magnet:") {
            match opener::open(&link) {
                Ok(_) => {
                    println!("Opening magnet link...");
                    num_dl = num_dl + 1;
                }
                Err(_) => println!("Error. Path not found."),
            };
        } else {
            let result = downloader(&link, &link.to_string());
            match result {
                Ok(r) => r,
                Err(_) => 0,
            };
            num_dl = num_dl + result.unwrap();
        }
    }
    if num_dl == 0 {
        info!("No items downloaded. Nyaadle closed.");
    } else {
        info!("{} items downloaded. Nyaadle closed.", num_dl);
    }
}

/// Initializes the download function then passes on the target link
/// to the downloader function
fn download_logic(item: &rss::Item, wl_title: String) -> u8 {
    // Get the link of the item
    let e: u8 = 0;
    let title = item.title().expect("Failed to extract title");
    println!("Downloading {}", title);
    let link = item.link();
    let target = match link {
        Some(link) => link,
        _ => return 0,
    };
    //FIXME: There's currently no way of checking if the item has already been downloaded.
    if target.contains("magnet:") {
        match opener::open(&target) {
            Ok(_) => 1,
            Err(_) => e,
        }
    } else {
        // Download the given link
        let result = downloader(target, &wl_title);
        match result {
            Ok(r) => r,
            Err(_) => e,
        }
    }
}

/// Function that parses the nyaa.si website then compares it against a
/// file containing the watch list of anime to download.
///
/// If an item title matches the watch list, it invokes the `download` function.
pub fn feed_parser(url: String, watch_list: Vec<Watchlist>) {
    // Create a channel for the rss feed and return a vector of items.
    let channel = Channel::from_url(&url).unwrap_or_else(|_e| {
        println!("Unable to connect to website. Exiting...");
        error!("Unable to connect to website. Nyaadle closed.");
        std::process::exit(0)
    });
    let items = channel.into_items();

    // Execute the main logic
    nyaadle_logic(items, watch_list, false);
}

pub fn feed_check(url: String, watch_list: Vec<Watchlist>) {
    // Create a channel for the rss feed and return a vector of items.
    info!("Nyaadle started in checking mode.");
    let channel = Channel::from_url(&url).unwrap_or_else(|_e| {
        println!("Unable to connect to website. Exiting...");
        error!("Unable to connect to website. Nyaadle closed.");
        std::process::exit(0)
    });
    let items = channel.into_items();

    // Execute the main logic
    nyaadle_logic(items, watch_list, true);
}
/// Main logic for the function.
/// The function iterates on the Vector `watch_list` and compares it to the `items` returned by the website.
/// This function also checks for the download option that is set by the user.
/// There can be two download options:
/// - A resolution number. This is used for video items.
///     Example: `1080p`, `720p`, `480p`
/// - `non-vid`. This is used for other items such as Books, Software, or Audio.
pub fn nyaadle_logic(items: Vec<rss::Item>, watch_list: Vec<Watchlist>, check: bool) {
    let chk = check;
    let non_opt = String::from("non-vid");
    let mut num_dl: u8 = 0;
    println!("Checking watch-list...\n");
    for anime in watch_list {
        // Transform anime into a string so it would be usable in the comparison.
        let title = anime.title;
        let option = anime.option;
        if &title == "" {
            warn!("Watch-list not found.");
            println!("Please set a watch-list by running 'nyaadle tui --watch-list'");
            break;
        } else {
            println!("Checking for {}", &title);
            // Iterate in the array items
            for item in &items {
                // Compare the 'title' and the 'item' to see if it's in the watch-list
                let check = item.title().expect("Failed to extract Post title");
                if check.contains(&title) {
                    let opt2 = &option;
                    if option == non_opt {
                        if chk == true {
                            println!("Found {}\n", &check);
                            continue;
                        } else {
                            let result = download_logic(item, (&check).to_string());
                            num_dl = num_dl + result;
                        }
                    } else if option == String::from("") {
                        warn!("Download Option not found.");
                        println!("Please set a download option using 'nyaadle tui --watch-list'");
                        break;
                    } else if check.contains(&option) {
                        if chk == true {
                            println!("Found {}\n", &check);
                            continue;
                        } else {
                            println!("Selecting {}p version", &opt2);
                            let result = download_logic(item, (&check).to_string());
                            num_dl = num_dl + result;
                        }
                    }
                }
            }
        }
    }
    if num_dl == 0 {
        info!("No items downloaded. Nyaadle closed.");
    } else {
        info!("{} items downloaded. Nyaadle closed.", num_dl);
    }
}
