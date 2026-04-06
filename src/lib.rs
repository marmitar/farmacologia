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
        .unwrap()
        .wait()
        .await
        .unwrap();

    if !status.success() {
        panic!("'ffmpeg' problem: exit code {:?}", status.code())
    }
}

mod cookies {
    use std::sync::OnceLock;
    use tokio::fs;

    static COOKIES: OnceLock<String> = OnceLock::new();

    #[inline]
    pub fn set_cookies(cookies: String) {
        COOKIES.set(cookies).unwrap();
    }

    #[inline]
    pub fn cookies() -> &'static str {
        COOKIES.get().expect("GLOBAL COOKIES UNSET")
    }

    #[inline]
    pub async fn read_cookies_txt() {
        let cookies = fs::read_to_string("cookies.txt").await.expect("Missing 'cookies.txt'");

        set_cookies(cookies)
    }
}
