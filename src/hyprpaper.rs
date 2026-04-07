use tokio::process::Command;

pub struct Hyprpaper;

impl Hyprpaper {
    const PROGRAM_NAME: &str = "/etc/profiles/per-user/wsq/bin/hyprctl";
    const DISPATCHER: &str = "hyprpaper";
    const SET_CMD: &str = "wallpaper";


    pub async fn set_wallpaper(path: &str) -> Result<(), std::io::Error> {
        let result = Command::new(Self::PROGRAM_NAME)
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
}
