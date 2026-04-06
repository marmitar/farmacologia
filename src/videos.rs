use crate::hotmart::Hotmart;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use std::sync::Arc;
use tokio::{fs, task, try_join};

enum Dir {
    Temp(TempDir),
    Fix(PathBuf),
}

impl Dir {
    #[inline]
    pub fn temp() -> Self {
        Self::Temp(TempDir::new().expect("failed to create temporary directory"))
    }

    #[inline]
    pub fn fix(path: &Path) -> Self {
        Self::build(path);
        Self::Fix(path.to_path_buf())
    }

    #[inline]
    pub fn path(&self) -> &Path {
        match self {
            Self::Temp(dir) => dir.path(),
            Self::Fix(path) => path,
        }
    }

    fn build(path: &Path) {
        use std::fs::create_dir_all;
        use std::io::ErrorKind::AlreadyExists;

        match create_dir_all(path) {
            Err(err) if err.kind() != AlreadyExists => panic!("{err}"),
            _ => (),
        }
    }
}

struct Hls {
    dir: Dir,
    hotmart: Hotmart,
}

impl Hls {
    #[inline]
    pub fn new(hotmart: Hotmart, dir: Option<&Path>) -> Self {
        let dir = dir.map_or_else(Dir::temp, Dir::fix);
        Self { dir, hotmart }
    }

    #[inline]
    async fn build_playlist(&self) -> PathBuf {
        let path = self.dir.path().join(Self::playlist());

        let contents = [Hotmart::start(), self.hotmart.playlist_info(), Self::video()];

        fs::write(&path, contents.join("\n"))
            .await
            .expect("could not save playlist");
        path
    }

    #[inline]
    fn build_segments(&self) -> Vec<String> {
        let segments = self.hotmart.segments();
        let mut segs = Vec::with_capacity(2 * segments.len());

        for (i, seg) in segments.iter().enumerate() {
            segs.push(seg.info.clone());
            segs.push(Self::segment(i));
        }
        segs
    }

    #[inline]
    async fn build_video(&self) {
        let path = self.dir.path().join(Self::video());

        let info = self.hotmart.info().join("\n");
        let segs = self.build_segments().join("\n");

        let contents = [info.as_ref(), segs.as_ref(), Hotmart::end_list()];

        fs::write(path, contents.join("\n"))
            .await
            .expect("could not save video part");
    }

    #[inline]
    pub async fn build(self: Arc<Self>) -> PathBuf {
        let (first, second) = (Arc::clone(&self), Arc::clone(&self));

        try_join!(
            task::spawn(async move { first.build_playlist().await }),
            task::spawn(async move { second.build_video().await }),
            task::spawn(async move {
                let path = self.dir.path().join(Self::segments());
                fs::create_dir(path).await.expect("failed to create output directory");
            })
        )
        .map(|(path, (), ())| path)
        .expect("failed to set up output directories")
    }

    #[inline]
    async fn download_segment(&self, url: &str, segment: usize) {
        let resp = self.hotmart.request(url).await;

        let path = self.dir.path().join(Self::segment(segment));
        fs::write(path, resp).await.expect("failed to save segment");
    }

    #[inline]
    pub async fn download(self: &Arc<Self>) {
        let handles = self.hotmart.segments().iter().enumerate().map(|(i, seg)| {
            let clone = Arc::clone(self);
            let url = seg.url.clone();

            task::spawn(async move { clone.download_segment(&url, i).await })
        });

        for (i, handle) in handles.into_iter().enumerate() {
            if let Err(err) = handle.await {
                panic!("Error at segment {i}: {err}")
            }
        }
    }

    #[inline]
    pub const fn playlist() -> &'static str {
        "playlist.m3u8"
    }
    #[inline]
    pub const fn video() -> &'static str {
        "video.m3u8"
    }
    #[inline]
    pub const fn segments() -> &'static str {
        "segs"
    }
    #[inline]
    pub fn segment(n: usize) -> String {
        let segs = Self::segments();
        format!("{segs}/{n}.ts")
    }
}

#[derive(Clone)]
pub struct Video(Arc<Hls>);

impl Video {
    #[inline]
    pub fn new(hotmart: Hotmart, dir: Option<&str>) -> Self {
        Self(Arc::new(Hls::new(hotmart, dir.as_ref().map(AsRef::as_ref))))
    }

    #[inline]
    pub async fn build(&self) -> PathBuf {
        Hls::build(Arc::clone(&self.0)).await
    }

    #[inline]
    pub async fn download(&self) {
        Hls::download(&self.0).await;
    }
}
