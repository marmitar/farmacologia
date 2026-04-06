#![allow(
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "no documentation"
)]

use hotmart::hotmart::{Hotmart, Playlist};
use hotmart::videos::Video;
use hotmart::{ffmpeg, read_cookies_txt};

use std::path::PathBuf;

#[tokio::main]
#[inline]
async fn main() {
    let cookies = read_cookies_txt();
    let (url, key, output, resolution, tmpdir) = args();

    cookies.await;
    let playlist = match resolution {
        Some(resolution) => Playlist::get(&url, &resolution).await,
        None => Playlist::get_max(&url).await,
    };
    let hotmart = Hotmart::get(playlist, &key).await;
    let video = Video::new(hotmart, tmpdir.as_deref());

    let input = video.build().await;
    video.download().await;

    ffmpeg(input, output).await;
}

#[inline]
fn args() -> (String, String, PathBuf, Option<String>, Option<String>) {
    let mut args = std::env::args().skip(1);

    let (Some(url), Some(key), Some(video)) = (args.next(), args.next(), args.next()) else {
        panic!("not enough arguments\nUSAGE: cargo run URL KEY OUTPUT [RESOLUTION] [TMPDIR]")
    };
    let resolution = args.next().filter(|s| s != "-");
    let tmpdir = args.next().filter(|s| s != "-");

    let path = std::env::current_dir().expect("no PWD").join(video);
    (url, key, path, resolution, tmpdir)
}
