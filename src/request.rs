use crate::cookies;
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client as Inner, Response, Result, Url};

#[derive(Clone)]
pub struct Client {
    inner: Inner,
    media: HeaderMap,
    segment: HeaderMap,
}

impl Client {
    #[inline]
    #[must_use]
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
            (header::COOKIE, cookies),
        ]);

        let client = Inner::builder()
            .default_headers(default)
            .gzip(true)
            .referer(false)
            .build()
            .expect("invalid HTTP client");

        let media = headers(&[(header::HOST, "player.hotmart.com"), (header::TE, "Trailers")]);
        let segment = headers(&[
            (header::HOST, "contentplayer.hotmart.com"),
            (header::ORIGIN, "https://player.hotmart.com"),
        ]);

        Self {
            inner: client,
            media,
            segment,
        }
    }

    #[inline]
    async fn request(&self, url: impl AsUrl, headers: &HeaderMap) -> Result<Response> {
        let req = self.inner.get(url.to_url());
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

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn headers<'a>(iter: impl IntoIterator<Item = &'a (HeaderName, &'a str)> + 'a) -> HeaderMap {
    iter.into_iter()
        .map(|(name, value)| (name.clone(), HeaderValue::from_str(value).expect("could not create HTTP header")))
        .collect()
}

pub trait AsUrl {
    fn to_url(self) -> Url;
}

impl AsUrl for Url {
    #[inline]
    fn to_url(self) -> Url {
        self
    }
}
impl AsUrl for &str {
    #[inline]
    fn to_url(self) -> Url {
        match self.parse() {
            Ok(url) => url,
            Err(err) => panic!("url parsing error for '{self}' := {err}"),
        }
    }
}
