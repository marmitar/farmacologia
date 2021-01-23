use reqwest::{Client as Inner, Url, Response, Result};
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use crate::cookies;


#[derive(Clone)]
pub struct Client {
    client: Inner,
    media: HeaderMap,
    segment: HeaderMap
}

impl Client {
    #[inline]
    pub fn new() -> Self {
        Self::with_cookies(cookies())
    }

    #[inline]
    fn with_cookies(cookies: &str) -> Self {
        let default = headers(&[
            (header::ACCEPT, "*/*"),
            (header::ACCEPT_ENCODING, "gzip, deflate, br"),
            (header::ACCEPT_LANGUAGE, "en,en-US;q=0.7,pt-BR;q=0.3"),
            (header::CONNECTION, "keep-alive"),
            (header::REFERER, "https://player.hotmart.com/"),
            (header::COOKIE, cookies)
        ]);

        let client = Inner::builder()
            .default_headers(default)
            .gzip(true)
            .referer(false)
            .build().unwrap();

        let media = headers(&[
            (header::HOST, "player.hotmart.com"),
            (header::TE, "Trailers")
        ]);
        let segment = headers(&[
            (header::HOST, "contentplayer.hotmart.com"),
            (header::ORIGIN, "https://player.hotmart.com")
        ]);

        Self { client, media, segment }
    }

    #[inline]
    async fn request(&self, url: impl AsUrl, headers: &HeaderMap) -> Result<Response> {
        let req = self.client.get(url.as_url());
        req.headers(headers.clone()).send().await
    }

    #[inline]
    pub async fn request_media(&self, url: impl AsUrl) -> Result<Response> {
        self.request(url, &self.media).await
    }

    #[inline]
    pub async fn request_segment(&self, url: impl AsUrl) -> Result<Response> {
        self.request(url, &self.segment).await
    }
}


#[inline]
fn headers<'a>(iter: impl IntoIterator<Item = &'a(HeaderName, &'a str)> + 'a) -> HeaderMap {
    iter.into_iter()
        .map(|(name, value)| {
            (name.clone(), HeaderValue::from_str(value).unwrap())
        })
        .collect()
}

pub trait AsUrl {
    fn as_url(self) -> Url;
}

impl AsUrl for Url {
    #[inline]
    fn as_url(self) -> Url {
        self
    }
}
impl<'a> AsUrl for &'a str {
    #[inline]
    fn as_url(self) -> Url {
        match self.parse() {
            Ok(url) => url,
            Err(err) =>
                panic!("Url parsing error for '{}' := {}", self, err)
        }
    }
}
