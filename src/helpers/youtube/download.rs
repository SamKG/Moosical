use crate::state::YoutubeConfig;
use std::{
    error::Error,
    path::PathBuf,
    process::{Command, Stdio},
};

pub(crate) async fn download_audio(
    url: String,
    config: &YoutubeConfig,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let stdout = Stdio::piped();
    let audio_download_path = &config.audio_download_path;
    let output = Command::new(&config.yt_dlp_path)
        .arg(url)
        .arg("--extract-audio")
        .arg(format!("-P {audio_download_path}"))
        .stdout(stdout)
        .output()?;

    match output.status.success() {
        true => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let dest_line = output_str
                .split('\n')
                .find(|x| x.contains("Destination:"))
                .unwrap();
            let dest_path = dest_line.strip_prefix("[download] Destination: ").unwrap();
            Ok(PathBuf::from(dest_path))
        }
        false => panic!("{}", String::from_utf8_lossy(&output.stderr)),
    }
}

#[cfg(test)]
mod tests {
    use crate::state::ApplicationConfig;

    use super::download_audio;
    use std::error::Error;
    use tokio::runtime::Runtime;

    #[test]
    fn test_download_audio() -> Result<(), Box<dyn Error + Sync + Send>> {
        let config_string = std::fs::read_to_string("config.toml")?;
        let config: ApplicationConfig = toml::from_str(&config_string)?;
        let handle = Runtime::new()?;
        let fut = download_audio("dQw4w9WgXcQ".to_string(), &config.youtube);
        let fpath = handle.block_on(fut)?;
        println!("download to {fpath:?}");
        Ok(())
    }
}
