// Parts of this code was adapted from "The Rust Cookbook" 
// which can be found at : https://rust-lang-nursery.github.io/rust-cookbook/


use std::io::copy;
use std::fs::File;
use rss::Channel;

#[macro_use]
extern crate error_chain;
extern crate reqwest;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

/// Function that takes in a link and downloads it to the specified path. 
/// Returns either an `Ok` or an `Err`.
fn download(target: &str) -> Result<()> {
    let tmp_dir = String::from("/home/elskiee/Transmission/torrent-ingest/");
    let mut response = reqwest::get(target)?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        println!("file to download: '{}'", fname);
        let fname = format!("{}{}", tmp_dir, fname);
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
fn feed_parser() {
    let channel = Channel::from_url("https://nyaa.si/?page=rss").unwrap();
    let items = channel.into_items();

    for item in items {
        if item.title().unwrap() == "[Erai-raws] Rifle Is Beautiful - 10 [1080p].mkv" {
            let link = item.link();
            let target = match link {
                Some(link) => link,
                _ => continue
            };
            let result = download(target);
            match result {
                Ok(_) => println!("Download Success!"),
                Err(_) => println!("An Error Occurred.")
            }
       } 
    }
}

/// The main function of the program.
fn main() {
    feed_parser();
}