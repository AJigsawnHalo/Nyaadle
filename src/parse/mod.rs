// Parts of this code was adapted from "The Rust Cookbook" 
// which can be found at: https://rust-lang-nursery.github.io/rust-cookbook/

use std::io::copy;
use std::io::Write;
use std::fs::File;
use std::fs::write;
use rss::Channel;
use config::Config;
use dirs;
use std::path::Path;
use std::fs::OpenOptions;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

/// Checks if the config directory exists and then creates it if it's not found.
pub fn write_settings() {
    // Gets the home directory
    let mut dl_dir = dirs::home_dir().unwrap();
    dl_dir.push("Transmission");
    dl_dir.push("torrent-ingest");
    let dl_dir = String::from(dl_dir.to_str().unwrap());
    
    // Settings Struct
    struct Settings {
        dl_key: String,
        dl_val: String,
        wl_key: String,
        wl_val: String
    }
    // Default Settings
    let default_set = Settings {
        dl_key: String::from("dl-dir"),
        dl_val: dl_dir,
        wl_key: String::from("watch-list"),
        wl_val: String::from("")
    };

    let set_file = settings_dir();
    let mut directory = dirs::config_dir().unwrap();
    directory.push("nyaadle");

    let directory = String::from(directory.to_str().unwrap());
    // If the settings file doesn't exist, create it.
    if Path::new(&set_file).exists() {
        return
    } else {
        // create directory
        std::fs::create_dir(&directory).expect("Unable to create directory");
        println!("{}", &set_file);
        // Create Settings.toml and add dl-dir
        let dl = format!("{} = \"{}\"\n", default_set.dl_key, default_set.dl_val);
        write(&set_file, dl).expect("Unable to write file");
        // Append watch-list to Settings.toml
        let mut file = OpenOptions::new().append(true).open(&set_file).unwrap();
        let wl = format!("{} = [ \n \"{}\", \n]\n", default_set.wl_key, default_set.wl_val);
        file.write_all(wl.as_bytes()).expect("Unable to append file");
    }
}

/// Sets the settings directory using User Variables.
fn settings_dir() -> String {
    let mut set_dir = dirs::config_dir().unwrap();
    set_dir.push("nyaadle");
    set_dir.push("Settings");
    set_dir.set_extension("toml");
    let set_dir = String::from(set_dir.to_str().unwrap());
    set_dir
}

/// Function that returns a `Config` struct from the crate Config. 
/// This allows us to read the settings set by the user.
pub fn get_settings() -> config::Config {
    let set_dir = settings_dir();

    let mut settings = Config::new();
    settings.merge(config::File::with_name(&set_dir)).unwrap();
    settings
}

/// Function that takes in a link and downloads it to the specified path. 
/// Returns either an `Ok` or an `Err`.
fn download(target: &str) -> Result<()> {
    // Get the download dir from the Settings.toml file
    let settings = get_settings();

    let dl_dir = settings.get_str("dl-dir").unwrap();

    let mut response = reqwest::get(target)?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        println!("file to download: '{}'", fname);
        let fname = format!("{}{}", dl_dir, fname);
        println!("will be located under: '{:?}'", fname);
        File::create(fname)?
    };
    copy(&mut response, &mut dest)?;
    Ok(())
}

/// Function that parses the nyaa.si website then compares it against a 
/// file containing the watch list of anime to download. 
/// 
/// If an item title matches the watch list, it invokes the `download` function.
pub fn feed_parser() {
    // Create a channel for the rss feed and return a vector of items.
    let channel = Channel::from_url("https://nyaa.si/?page=rss").expect("Unable to connect to website");
    let items = channel.into_items();

    // Read the watch-list from the Settings.toml
    let set_dir = settings_dir();
    let settings = get_settings();
    // Transform the watch-list into an array.
    let watch_list = settings.get_array("watch-list").unwrap();

    // Main logic for the function
    // The function iterates on the array 'watch_list' and compares it to the 'items' returned by the website.

    // Iterate in the array 'watch_list'
    for anime in watch_list{
        // Transform anime into a string so it would be usable in the comparison.
        let title = anime.into_str().unwrap();
        if &title == "" {
            println!("Please set a watch-list in the config file in: {}", set_dir);
        } else {
            println!("Checking for {}", &title);
            // Iterate in the array items
            for item in &items {
                // Compare the 'title' and the 'item' to see if it's in the watch-list
                if item.title().unwrap() == &title {
                    // Get the link of the item
                    let link = item.link();
                    let target = match link {
                        Some(link) => link,
                        _ => continue
                    };
                    // Download the given link
                    let result = download(target);
                    match result {
                        Ok(_) => println!("Download Success!"),
                        Err(_) => println!("An Error Occurred.")
                    }
                }
            }
        }
    }
}