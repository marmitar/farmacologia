#![allow(
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "no documentation"
)]

pub mod decrypt;
pub mod hotmart;
pub mod request;
pub mod videos;
pub use cookies::{cookies, read_cookies_txt, set_cookies};

use std::path::PathBuf;
use tokio::process::Command;

#[inline]
pub async fn ffmpeg(input: PathBuf, output: PathBuf) {
    let status = Command::new("ffmpeg")
        .arg("-allowed_extensions")
        .arg("ALL")
        .arg("-protocol_whitelist")
        .arg("file,http,https,tcp,tls,crypto")
        .arg("-i")
        .arg(input)
        .arg("-c")
        .arg("copy")
        .arg("-bsf:a")
        .arg("aac_adtstoasc")
        .arg(output)
        .spawn()
        .expect("ffmpeg could not be started")
        .wait()
        .await
        .expect("ffmpeg failed");

    assert!(status.success(), "'ffmpeg' problem: exit code {:?}", status.code());
}

mod cookies {
    use std::sync::OnceLock;
    use tokio::fs;

    static COOKIES: OnceLock<String> = OnceLock::new();

    #[inline]
    pub fn set_cookies(cookies: String) {
        COOKIES.set(cookies).expect("could not update");
    }

    #[inline]
    pub fn cookies() -> &'static str {
        COOKIES.get().expect("global cookies unset")
    }

    #[inline]
    pub async fn read_cookies_txt() {
        let cookies = fs::read_to_string("cookies.txt").await.expect("missing 'cookies.txt'");

        set_cookies(cookies);
    }
}
