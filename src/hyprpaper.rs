use tokio::process::Command;

pub struct Hyprpaper;

impl Hyprpaper {
    const DISPATCHER: &str = "hyprpaper";
    const PRELOAD_CMD: &str = "preload";
    const RELOAD_CMD: &str = "reload";
    const SET_CMD: &str = "wallpaper";

    pub async fn reload_wallpaper(path: &str) -> Result<(), std::io::Error> {
        let result = Command::new("hyprctl")
            .arg(Self::DISPATCHER)
            .arg(Self::RELOAD_CMD)
            .arg(path)
            .output()
            .await?;
        if result.status.success() {
            log::info!("Reloaded wallpaper: {}, result: {:?}", path, result);
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&result.stderr);
            log::error!("Failed to reload wallpaper: {}: {}", path, err_msg);
            Err(std::io::Error::new(std::io::ErrorKind::Other, err_msg))
        }
    }

    pub async fn preload_wallpaper(path: &str) -> Result<(), std::io::Error> {
        let result = Command::new("hyprctl")
            .arg(Self::DISPATCHER)
            .arg(Self::PRELOAD_CMD)
            .arg(path)
            .output()
            .await?;
        if result.status.success() {
            log::info!("Preloaded wallpaper: {}, result: {:?}", path, result);
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&result.stderr);
            log::error!("Failed to preload wallpaper: {}: {}", path, err_msg);
            Err(std::io::Error::new(std::io::ErrorKind::Other, err_msg))
        }
    }

    pub async fn set_wallpaper(path: &str) -> Result<(), std::io::Error> {
        let result = Command::new("hyprctl")
            .arg(Self::DISPATCHER)
            .arg(Self::SET_CMD)
            .arg(format!(",{}", path))
            .output()
            .await?;
        if result.status.success() {
            log::info!("Set wallpaper: {}, result: {:?}", path, result);
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&result.stderr);
            log::error!("Failed to set wallpaper: {}: {}", path, err_msg);
            Err(std::io::Error::new(std::io::ErrorKind::Other, err_msg))
        }
    }

    pub async fn preload_all_wallpapers(paths: &[String]) -> Result<(), std::io::Error> {
        for path in paths {
            Self::preload_wallpaper(path).await?;
            Self::reload_wallpaper(path).await?;
        }
        Ok(())
    }
}
