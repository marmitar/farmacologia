use hotmart::hotmart::{Playlist, Hotmart};
use hotmart::videos::Video;
use hotmart::{ffmpeg, set_cookies};

use std::io::stdin;
use std::path::PathBuf;


#[tokio::main]
#[inline]
async fn main() {
    let (url, key, output) = args();
    read_cookies();
    println!("Downloading...");

    let playlist = Playlist::get_max(&url).await;
    let hotmart = Hotmart::get(playlist, &key).await;
    let video = Video::new(hotmart, None);

    let input = video.build().await;
    video.download().await;

    ffmpeg(input, output).await
}

#[inline]
fn args() -> (String, String, PathBuf) {
    let mut args = std::env::args().skip(1);

    match (args.next(), args.next(), args.next()) {
        (Some(url), Some(key), Some(video)) => {
            let path = std::env::current_dir().unwrap().join(video);
            (url, key, path)
        },
        _ => {
            panic!("Not enough arguments\nUSAGE: cargo run URL KEY OUTPUT")
        }
    }
}

#[inline]
fn read_cookies() {
    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();
    let text = line.trim_end();

    let cookies = match text.strip_prefix("Cookie: ") {
        Some(text) => text,
        None => text
    };

    set_cookies(String::from(cookies))
}
