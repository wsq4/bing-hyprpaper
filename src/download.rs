use std::path::PathBuf;
use futures_util::TryStreamExt;
use thiserror::Error;
use tokio_util::io::StreamReader;

use crate::model::{ApiResponse, Image, ImageStoreItem, Query};

pub struct Downloader {
    pub width: u32,
    pub height: u32,
    pub storage_path: PathBuf,
    pub download_counts: u8,
}

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("HTTP request error: {0}")]
    RequestError(reqwest::Error),
    #[error("Reqwest JSON error: {0}")]
    JsonError(reqwest::Error),
    #[error("IO error: {0}")]
    IoError(tokio::io::Error),
}

impl Downloader {
    const BING_BASE_URL: &str = "https://www.bing.com";
    const BING_WALLPAPER_URL: &str = "https://www.bing.com/HPImageArchive.aspx";
    const RESPONSE_FORMAT: &str = "js";
    const MARKET: &str = "en-US";
    const DOWNLOAD_COUNTS: u8 = 10;

    pub fn new(width: u32, height: u32, storage_path: PathBuf, download_counts: u8) -> Self {
        Downloader { width, height, storage_path, download_counts: u8::min(download_counts, Self::DOWNLOAD_COUNTS) }
    }

    pub async fn fetch_image_list(&self) -> Result<Vec<Image>, DownloadError> {
        let query = Query {
            format: Self::RESPONSE_FORMAT,
            idx: 0,
            n: self.download_counts,
            mkt: Self::MARKET,
            uhd: 1,
            uhdwidth: self.width,
            uhdheight: self.height,
        };

        let client = reqwest::Client::new();
        let response = client
            .get(Self::BING_WALLPAPER_URL)
            .query(&query)
            .send()
            .await
            .map_err(DownloadError::RequestError)?;
        
        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(DownloadError::JsonError)?;

        Ok(api_response.images)
    }

    pub async fn download(&self, image: &Image) -> Result<Option<ImageStoreItem>, DownloadError> {
        let image_url = format!("{}{}", Self::BING_BASE_URL, image.url);
        let file_name = format!("{}_{}.jpg", image.startdate, image.hsh);
        let file_path = self.storage_path.join(file_name);
        if file_path.exists() {
            log::info!("Image already exists: {:?}", file_path);
            return Ok(None);
        }
        log::info!("Downloading image from: {}", image_url);
        let response = reqwest::get(&image_url).await.map_err(DownloadError::RequestError)?;
        let byte_stream = response.bytes_stream();

        let mut file = tokio::fs::File::create(&file_path).await.map_err(DownloadError::IoError)?;
        let byte_stream = byte_stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
        let mut reader = StreamReader::new(byte_stream);

        tokio::io::copy(&mut reader, &mut file).await.map_err(DownloadError::IoError)?;

        log::info!("Image saved to: {:?}", file_path);

        Ok(Some(ImageStoreItem {
            path: file_path,
            file_created: std::time::SystemTime::now(),
        }))
    }
}