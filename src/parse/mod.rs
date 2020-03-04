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
    
    let mut ar_dir = dirs::home_dir().unwrap();
    ar_dir.push("Transmission");
    ar_dir.push("torrent-ingest");
    ar_dir.push("archive");
    let ar_dir = String::from(ar_dir.to_str().unwrap());
    // Settings Struct
    struct Settings {
        dl_key: String,
        dl_val: String,
        ar_key: String,
        ar_val: String,
        wl_key: String,
        wl_val: String,
        nf_key: String,
        nf_val: String
    }
    // Default Settings
    let default_set = Settings {
        dl_key: String::from("dl-dir"),
        dl_val: dl_dir,
        ar_key: String::from("ar-dir"),
        ar_val: ar_dir,
        wl_key: String::from("watch-list"),
        wl_val: String::from(""),
        nf_key: String::from("non-fhd"),
        nf_val: String::from("")
    };

    let set_file = settings_dir();
    let mut directory = dirs::config_dir().unwrap();
    directory.push("nyaadle");

    let directory = String::from(directory.to_str().unwrap());
    // If the settings file doesn't exist, create it.
    if Path::new(&set_file).exists() {
        return
    } else {
        println!("Settings.toml not found. Creating it right now.");
        // create directory
        std::fs::create_dir(&directory).expect("Unable to create directory");
        // Create Settings.toml and add dl-dir
        let dl = format!("{} = \"{}\"\n", default_set.dl_key, default_set.dl_val);
        write(&set_file, dl).expect("Unable to write file");
        // Append watch-list to Settings.toml
        let mut file = OpenOptions::new().append(true).open(&set_file).unwrap();
        let ar = format!("{} = \"{}\"\n", default_set.ar_key, default_set.ar_val);
        file.write_all(ar.as_bytes()).expect("Unable to append file");
        let wl = format!("{} = [ \n \"{}\", \n]\n", default_set.wl_key, default_set.wl_val);
        file.write_all(wl.as_bytes()).expect("Unable to append file");
        let nf = format!("{} = [ \n \"{}\", \n]\n", default_set.nf_key, default_set.nf_val);
        file.write_all(nf.as_bytes()).expect("Unable to append file");
        
        println!("Settings.toml created.");
        println!("You can change settings by editing the config file in {}", &set_file); 
    }
}

/// Sets the settings directory using User Variables.
pub fn settings_dir() -> String {
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

fn archive_check(target: &str) -> &str {
    let settings = get_settings();
    let mut archive_dir = settings.get_str("ar-dir").unwrap();
    let response = reqwest::get(target).unwrap();
    let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");
    archive_dir.push_str(fname);
    let path = Path::new(&archive_dir);
    match path.exists() {
        true => "Found",
        false => "Empty"
    }
}

/// Function that takes in a link and downloads it to the specified path. 
/// Returns either an `Ok` or an `Err`.
fn download(target: &str) -> Result<()> {
    // Get the download dir from the Settings.toml file
    let settings = get_settings();

    let dl_dir = settings.get_str("dl-dir").unwrap();
    let archive_dir = settings.get_str("ar-dir").unwrap();
    let check = archive_check(&target);
    if check == "Found" {
        println!("File Found. Skipping Download");
        return Ok(())
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

/// Function that parses the nyaa.si website then compares it against a 
/// file containing the watch list of anime to download. 
/// 
/// If an item title matches the watch list, it invokes the `download` function.
pub fn feed_parser() {
    // Create a channel for the rss feed and return a vector of items.
    let channel = Channel::from_url("https://nyaa.si/?page=rss").expect("Unable to connect to website");
    let items = channel.into_items();
    let items2 = items.clone();

    // Read the watc-list from the Settings.toml
    let set_dir = settings_dir();
    let set_dir2 = set_dir.clone();
    let settings = get_settings();
    // Transform the watch-list into an array.
    let watch_list = settings.get_array("watch-list").unwrap();
    let non_fhd_list = settings.get_array("non-fhd").unwrap();
    nyaadle_logic_fhd(items, watch_list, set_dir);
    nyaadle_logic(items2, non_fhd_list, set_dir2);
}
/// Main logic for the function. Used on 1080p versions. This is the default option.
/// The function iterates on the array 'watch_list' and compares it to the 'items' returned by the website.
/// Iterate in the array 'watch_list'
pub fn nyaadle_logic_fhd(items: Vec<rss::Item>, watch_list: Vec<config::Value>, set_dir: String) {
    println!("Checking 1080p versions...");
    for anime in watch_list {
        // Transform anime into a string so it would be usable in the comparison.
        let title = anime.into_str().unwrap();
        if &title == "" {
            println!("Please set a watch-list in the config file in: {}", set_dir);
        } else if &title == "Skip" {
            println!("Skipping 1080p check.\n");
            continue
        } else {
            println!("Checking for {}", &title);
            // Iterate in the array items
            for item in &items {
                // Compare the 'title' and the 'item' to see if it's in the watch-list
                let check = item.title().unwrap();
                if check.contains(&title) {
                    if check.contains("[1080p]"){
                        // Get the link of the item
                        let link = item.link();
                        let target = match link {
                            Some(link) => link,
                            _ => continue
                        };
                        // Download the given link
                        let result = download(target);
                        match result {
                            Ok(_) => println!("Success.\n"),
                            Err(_) => println!("An Error Occurred.\n")
                        }
                    }
                }
            }
        }
    }
}

/// Main logic for the function. Used on items that has no choice of resolution. 
/// The function iterates on the array 'watch_list' and compares it to the 'items' returned by the website.
/// Iterate in the array 'watch_list'
// FIXME: Currently downloads all resolutions if there's different versions found
pub fn nyaadle_logic(items: Vec<rss::Item>, watch_list: Vec<config::Value>, set_dir: String) {
    println!("\nChecking non-1080p versions...");
    for anime in watch_list {
        // Transform anime into a string so it would be usable in the comparison.
        let title = anime.into_str().unwrap();
        if &title == "" {
            println!("Please set a watch-list in the config file in: {}", set_dir);
        } else if &title == "Skip"{
            println!("Skipping non-1080p check.\n");
            continue
        } else {
            println!("Checking for {}", &title);
            // Iterate in the array items
            for item in &items {
                // Compare the 'title' and the 'item' to see if it's in the watch-list
                let check = item.title().unwrap();
                if check.contains("1080"){
                    continue   
                } else if check.contains(&title) {
                        // Get the link of the item
                        let link = item.link();
                        let target = match link {
                            Some(link) => link,
                            _ => continue
                        };
                        // Download the given link
                        let result = download(target);
                        match result {
                            Ok(_) => println!("Success.\n"),
                            Err(_) => println!("An Error Occurred.\n")
                        }
                }
            }
        }
    }
}
