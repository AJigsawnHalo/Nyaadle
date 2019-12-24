// Parts of this code was adapted from "The Rust Cookbook" 
// which can be found at: https://rust-lang-nursery.github.io/rust-cookbook/

use std::io::copy;
use std::fs::File;
use rss::Channel;
use config::Config;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

/// Function that takes in a link and downloads it to the specified path. 
/// Returns either an `Ok` or an `Err`.
fn download(target: &str) -> Result<()> {
    // Get the download dir from the Settings.toml file
    let mut settings = Config::new();
    settings.merge(config::File::with_name("Settings")).unwrap();
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
    let channel = Channel::from_url("https://nyaa.si/?page=rss").unwrap();
    let items = channel.into_items();

    // Read the watch-list from the Settings.toml
    let mut settings = Config::new();
    settings.merge(config::File::with_name("Settings")).unwrap();
    // Transform the watch-list into an array.
    let watch_list = settings.get_array("watch-list").unwrap();

    // Main logic for the function
    // The function iterates on the array 'watch_list' and compares it to the 'items' returned by the website.

    // Iterate in the array 'watch_list'
    for anime in watch_list{
        // Transform anime into a string so it would be usable in the comparison.
        let title = anime.into_str().unwrap();
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