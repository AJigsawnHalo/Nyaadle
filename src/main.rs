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

fn main() {
    let target = "https://www.rust-lang.org/logos/rust-logo-512x512.png";
    let _result = download(target); 
}