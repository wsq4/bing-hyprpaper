use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};

use futures::{StreamExt, future};
use thiserror::Error;
use tokio::{fs::{self, DirEntry}, sync::RwLock, time::Instant};
use tokio_stream::wrappers::ReadDirStream;

use crate::{args::Args, hyprpaper::Hyprpaper, model::ImageStoreItem};

pub struct App<'a> {
    pub args: &'a Args,
    pub file_store: Arc<RwLock<Vec<ImageStoreItem>>>,
    pub downloader: crate::download::Downloader,
    pub last_download: Arc<RwLock<tokio::time::Instant>>,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    #[error("Download error: {0}")]
    DownloadError(#[from] crate::download::DownloadError),
}

impl<'a> App<'a> {
    pub fn new(args: &'a Args) -> Result<Self, AppError> {
        let storage_path = PathBuf::from(&args.storage_path);
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).map_err(AppError::IoError)?;
        }

        let current_wallpaper_info_path = PathBuf::from(&args.current_wallpaper_info);
        if let Some(parent) = current_wallpaper_info_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(AppError::IoError)?;
            }
        }

        let file_store = Arc::new(RwLock::new(Vec::new()));
        let downloader = crate::download::Downloader::new(args.width, args.height, storage_path, args.max_images as u8);
        let last_download = Arc::new(RwLock::new(
            Instant::now() - Duration::from_secs(args.refresh_interval * 3600),
        ));

        Ok(App {
            args,
            file_store,
            downloader,
            last_download,
        })
    }

    async fn update_file_store(&self) -> Result<(), AppError> {
        let images = self.downloader.fetch_image_list().await?;
        log::info!("Fetched {} images", images.len());

        let tasks: Vec<_> = images
            .iter()
            .map(|img| {
                let file_store = self.file_store.clone();
                let downloader = &self.downloader;
                async move {
                    if let Ok(opt) = downloader.download(img).await {
                        if let Some(item) = opt {
                            let mut store = file_store.write().await;
                            store.push(item);
                        } else {
                            log::info!("Image already exists, skipped: {:?}", img.url);
                        }
                    } else {
                        log::error!("Error downloading image: {:?}", img.url);
                    }
                }
            })
            .collect();

        let _ = future::join_all(tasks).await;
        let mut last = self.last_download.write().await;
        *last = Instant::now();
        self.maintain_storage().await;

        Ok(())
    }

    async fn maintain_storage(&self) {
        let max_images = self.args.max_images;
        let mut store = self.file_store.write().await;
        store.sort_by_key(|item| item.file_created);
        if store.len() > max_images {
            let excess = store.len() - max_images;
            let to_remove: Vec<_> = store.drain(0..excess).collect();
            for item in to_remove {
                if let Err(e) = tokio::fs::remove_file(&item.path).await {
                    log::error!("Failed to remove file {:?}: {}", item.path, e);
                } else {
                    log::info!("Removed old image: {:?}", item.path);
                }

                let meta_path = item.path.with_extension("json");
                if let Err(e) = tokio::fs::remove_file(&meta_path).await {
                    log::error!("Failed to remove metadata file {:?}: {}", meta_path, e);
                } else {
                    log::info!("Removed old metadata: {:?}", meta_path);
                }
            }
        }

        log::info!("Maintained storage, current image count: {}", store.len());
    }

    async fn should_refresh(&self) -> bool {
        let last_download = self.last_download.read().await;
        last_download.elapsed() >= Duration::from_secs(self.args.refresh_interval * 3600)
    }

    async fn load_existing_images(&self) -> Result<(), AppError> {
        let storage_path = std::path::PathBuf::from(&self.args.storage_path);
        let entries = tokio::fs::read_dir(&storage_path)
            .await
            .map_err(AppError::IoError)?;

        let stream = ReadDirStream::new(entries);

        let mut entries: Vec<(DirEntry, Option<SystemTime>)> = stream
            .filter_map(|res| async {
                match res {
                    Ok(entry) => {
                        let metadata = entry.metadata().await.ok();
                        let created = metadata.and_then(|m| m.modified().ok());
                        Some((entry, created))
                    }
                    Err(_) => None,
                }
            })
            .collect()
            .await;

        entries.sort_by_key(|&(_, created)| created);

        let mut store = self.file_store.write().await;
        for (entry, created) in entries {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jpg") {
                store.push(ImageStoreItem {
                    path,
                    file_created: created.unwrap_or(SystemTime::now()),
                });
            }
        }
        log::info!("Loaded {} existing images from storage", store.len());
        Ok(())
    }

    pub async fn set_random_wallpaper(&self) -> Result<(), AppError> {
        let store = self.file_store.read().await;
        if store.is_empty() {
            log::warn!("No images available to set as wallpaper");
            return Ok(());
        }
        let idx = rand::random::<u64>() as usize % store.len();
        let selected = &store[idx];

        let path = fs::canonicalize(&selected.path).await.map_err(AppError::IoError)?;
        let meta_path = path.with_extension("json");
        
        let wallpaper_info_path = PathBuf::from(&self.args.current_wallpaper_info);
        _ = tokio::fs::remove_file(&wallpaper_info_path).await;
        tokio::fs::symlink(&meta_path, &wallpaper_info_path).await.map_err(AppError::IoError)?;
        let current_wallpaper_path = wallpaper_info_path.with_extension("jpg");
        _ = tokio::fs::remove_file(&current_wallpaper_path).await;
        tokio::fs::symlink(&path, &current_wallpaper_path).await.map_err(AppError::IoError)?;

        match Hyprpaper::set_wallpaper(&path.to_string_lossy().to_string()).await {
            Ok(_) => {
                log::info!("Set wallpaper to: {:?}", selected.path);
                Ok(())
            },
            Err(e) => {
                log::error!("Failed to set wallpaper: {}", e);
                Err(AppError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "Failed to set wallpaper")))
            },
        }
    }

    pub async fn run(&self) -> Result<(), AppError> {
        self.load_existing_images().await?;
        
        let mut ticker = tokio::time::interval(Duration::from_secs(self.args.interval));

        loop {
            ticker.tick().await;
            log::debug!("Tick at {:?}", tokio::time::Instant::now());

            if self.should_refresh().await {
                log::info!("Refreshing image list...");
                if let Err(e) = self.update_file_store().await {
                    log::error!("Error updating file store: {}", e);
                }
            }

            if let Err(e) = self.set_random_wallpaper().await {
                log::error!("Error setting wallpaper: {}", e);
            }
        }
    }
}
