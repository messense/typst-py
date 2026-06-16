use typst_kit::downloader::{Downloader, SystemDownloader};

/// Returns a new downloader.
pub fn downloader() -> impl Downloader {
    let user_agent = concat!("typst-py/", env!("CARGO_PKG_VERSION"));
    SystemDownloader::new(user_agent)
}
