#![allow(
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "no documentation"
)]

use hotmart::hotmart::Playlist;
use hotmart::read_cookies_txt;
use std::path::PathBuf;
use tokio::{fs, task, try_join};

#[tokio::main]
#[inline]
async fn main() {
    let cookies = read_cookies_txt();
    let (url, outdir) = args();
    cookies.await;

    let (text, playlists) = Playlist::get_all(&url).await;
    let path = outdir.join("context.m3u8");

    let handles: Vec<_> = playlists
        .map(move |plt| {
            let path = outdir.join(plt.resolution() + ".m3u8");

            task::spawn(async move {
                fs::write(path, plt.text().await)
                    .await
                    .expect("failed to save playlist");
            })
        })
        .collect();

    try_join!(
        task::spawn(async move { fs::write(path, text).await.expect("failed to save list") }),
        task::spawn(async move {
            for (i, handle) in handles.into_iter().enumerate() {
                if let Err(err) = handle.await {
                    panic!("error at segment {i}: {err}")
                }
            }
        })
    )
    .expect("failed to save parts");
}

#[inline]
fn args() -> (String, PathBuf) {
    let mut args = std::env::args().skip(1);
    let curdir = std::env::current_dir().expect("no PWD");

    match (args.next(), args.next()) {
        (Some(url), None) => (url, curdir),
        (Some(url), Some(dir)) => (url, curdir.join(dir)),
        _ => panic!("not enough arguments\nUSAGE: cargo run URL [OUTDIR]"),
    }
}
