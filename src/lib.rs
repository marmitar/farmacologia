pub mod decrypt;
pub mod request;
pub mod hotmart;
pub mod videos;
pub use cookies::{cookies, set_cookies, read_cookies_txt};

use tokio::process::Command;
use std::path::PathBuf;


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
        .await
        .unwrap();

    if !status.success() {
        panic!("'ffmpeg' problem: exit code {:?}", status.code())
    }
}


mod cookies {
    use std::sync::Once;
    use tokio::fs;

    static mut COOKIES: String = String::new();
    static GUARD: Once = Once::new();

    #[inline]
    pub fn set_cookies(cookies: String) {
        unsafe {
            GUARD.call_once(|| {
                COOKIES = cookies;
            })
        }
    }

    #[inline]
    pub fn cookies() -> &'static str {
        if !GUARD.is_completed() {
            panic!("GLOBAL COOKIES UNSET")
        }

        unsafe {
            &COOKIES
        }
    }

    #[inline]
    pub async fn read_cookies_txt() {
        let cookies = fs::read_to_string("cookies.txt").await
            .expect("Missing 'cookies.txt'");

            set_cookies(cookies)
    }
}
