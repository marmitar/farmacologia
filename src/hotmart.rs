use crate::request::Client;
use crate::decrypt::Decrypter;


pub struct Hotmart {
    client: Client,
    decrypter: Decrypter,

    playlist: String,
    infos: Vec<String>,
    urls: Vec<Segment>
}


impl Hotmart {
    #[inline]
    async fn load(client: &Client, url: &str) -> (Vec<String>, Vec<Segment>, String) {
        let text = get_text(client, url).await;

        let mut info = Vec::with_capacity(10);
        let mut urls = Vec::with_capacity(100);

        let mut crypto = None;
        let mut inf = None;
        let mut segments = false;

        for line in text.lines() {
            if !segments {
                if line.starts_with("#EXTINF") {
                    inf = Some(String::from(line));
                    segments = true

                } else if line.starts_with("#EXT-X-KEY") {
                    crypto = Some(String::from(line))

                } else {
                    info.push(String::from(line))
                }

            } else if let Some(info) = inf {
                let url = String::from(line);
                urls.push(Segment {info, url});
                inf = None

            } else if line.starts_with(Self::end_list()) {
                break

            } else {
                inf = Some(String::from(line))
            }
        }

        (info, urls, crypto.expect("'EXT-X-KEY' not found"))
    }

    #[inline]
    pub async fn get(playlist: Playlist, key: &str) -> Self {
        let Playlist { client, url, info } = playlist;

        let (mut infos, urls, crypto) = Self::load(&client, &url).await;
        infos.push(String::from("#EXT-X-KEY:METHOD=NONE"));

        let iv = crypto.split("IV=0x").nth(1).expect("Could not get IV from 'EXT-X-KEY'");
        let decrypter = Decrypter::new(key, iv);

        Self { client, decrypter, infos, urls, playlist: info }
    }

    #[inline]
    pub fn info(&self) -> &[String] {
        self.infos.as_ref()
    }

    #[inline]
    pub fn segments(&self) -> &[Segment] {
        self.urls.as_ref()
    }

    #[inline]
    pub fn start() -> &'static str {
        "#EXTM3U"
    }

    #[inline]
    pub fn end_list() -> &'static str {
        "#EXT-X-ENDLIST"
    }

    #[inline]
    pub fn playlist_info(&self) -> &str {
        self.playlist.as_ref()
    }

    #[inline]
    pub async fn request(&self, url: &str) -> Vec<u8> {
        let resp = self.client.request_segment(url).await.unwrap();
        let data = resp.bytes().await.unwrap();

        self.decrypter.decrypt(data.as_ref())
    }
}


pub struct Segment {
    pub info: String,
    pub url: String
}


pub struct Playlist {
    client: Client,
    url: String,
    info: String
}

impl Playlist {
    #[inline]
    pub fn resolution(&self) -> String {
        String::from(match self.info.rfind('=') {
            Some(i) => &self.info[i+1..],
            None => "UNKNOWN"
        })
    }

    #[inline]
    pub async fn text(&self) -> String {
        get_text(&self.client, &self.url).await
    }

    #[inline]
    fn get_playlists(text: String) -> Vec<(String, String)> {
        let mut iter = text.lines();
        let mut ans = Vec::new();

        loop {
            let info = match iter.next() {
                Some(info) if info.contains("RESOLUTION") =>
                    String::from(info),
                Some(_) => continue,
                None => break ans
            };

            let url = match iter.next() {
                Some(url) => String::from(url),
                None => {
                    let resolution = match info.rfind('=') {
                        Some(i) => &info[i+1..],
                        None => "UNKNOWN"
                    };
                    eprintln!("WARNING: Could not find URL for {}", resolution);
                    break ans
                }
            };

            ans.push((info, url))
        }
    }

    #[inline]
    pub async fn get(url: &str, resolution: &str) -> Self {
        let client = Client::new();
        let text = get_text(&client, url).await;

        for (info, url) in Self::get_playlists(text) {
            if info.contains(resolution) {
                return Self { client, url, info }
            }
        }

        panic!("Could not find RESOLUTION={}", resolution)
    }

    #[inline]
    pub async fn get_max(url: &str) -> Self {
        let client = Client::new();
        let text = get_text(&client, url).await;

        let (info, url) = Self::get_playlists(text)
            .into_iter()
            .max_by_key(|(info, _)| resolution(info))
            .expect("No Playlist found");

        Self { client, url, info }
    }

    #[inline]
    pub async fn get_all(url: &str) -> (String, impl Iterator<Item=Self>) {
        let client = Client::new();
        let text = get_text(&client, url).await;

        let iter = Self::get_playlists(text.clone())
            .into_iter()
            .map(move |(info, url)| Self {
                client: client.clone(),
                url, info
            });

        (text, iter)
    }
}

#[inline]
fn resolution(info: &str) -> usize {
    let res = match info.rfind('=') {
        Some(i) => &info[i+1..],
        None => return 0
    };

    let mut iter = res.split('x')
        .filter_map(|s| s.parse::<usize>().ok());

    match (iter.next(), iter.next()) {
        (Some(w), Some(h)) => w * h,
        _ => 0
    }
}

#[inline]
async fn get_text(client: &Client, url: &str) -> String {
    client.request_media(url)
        .await.unwrap()
        .text()
        .await.unwrap()
}
