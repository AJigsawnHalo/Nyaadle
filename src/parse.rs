// Parts of this code was adapted from "The Rust Cookbook"
// which can be found at: https://rust-lang-nursery.github.io/rust-cookbook/

use crate::settings;
use crate::settings::Watchlist;
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
fn downloader(target: &str) -> Result<()> {
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

pub fn arg_dl(links: Vec<String>) {
    for link in links.iter(){

        if link == "" {
            println!("No link found. Exiting...");
            return
        } else if link == "\n" {
            return
        } else {
            let result = downloader(&link);
            match result {
                Ok(_) => println!("Success.\n"),
                Err(_) => println!("An Error Occurred.\n"),
            }
        }
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
    let _empty_url = String::from("");
    let url = {
        if &settings::get_settings(&String::from("url")).unwrap() == "" {
            String::from("https://nyaa.si/?page=rss")
        } else {
            settings::get_settings(&String::from("url")).unwrap()
        }
    };

    let channel = Channel::from_url(&url).unwrap_or_else(|_e| {
        println!("Unable to connect to website. Exiting...");
        std::process::exit(0)
    });
    let items = channel.into_items();

    // Read the watchlist from the database
    let set_dir = settings::settings_dir();
    let watch_list = settings::read_watch_list(&set_dir).expect("Failed to unpack vectors");

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
                    let opt2 = &option;
                    if option == non_opt {
                        download_logic(item);
                    } else if option == String::from("") {
                        println!(
                            "Please set download option in the config file: {}",
                            &set_dir
                        );
                    } else if check.contains(&option) {
                        println!("Selecting {}p version", &opt2);
                        download_logic(item);
                        items.iter();
                    }
                }
            }
        }
    }
}
