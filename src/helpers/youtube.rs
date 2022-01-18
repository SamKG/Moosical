use std::{
    error::Error,
    path::PathBuf,
    process::{Command, Stdio},
};
use youtube_dl::{SearchOptions, SingleVideo, YoutubeDl, YoutubeDlOutput};

const YT_DLP_PATH: &str = "/data/samyakg/anaconda3/bin/yt-dlp";
const AUDIO_DOWNLOAD_PATH: &str = "/dev/shm";


pub(crate) async fn search_for(
    query: &str,
    num_results: usize,
) -> Result<Vec<SingleVideo>, Box<dyn Error + Send + Sync>> {
    let results = YoutubeDl::search_for(&SearchOptions::youtube(query).with_count(num_results))
        .youtube_dl_path(YT_DLP_PATH)
        .flat_playlist(true)
        .run()?;
    match results {
        YoutubeDlOutput::Playlist(results) => Ok(results.entries.unwrap()),
        _ => {
            panic!("Unhandled result type!");
        }
    }
}

pub(crate) async fn download_audio(url: String) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let stdout = Stdio::piped();
    let output = Command::new(YT_DLP_PATH)
        .arg(url)
        .arg("--extract-audio")
        .arg(format!("-P {AUDIO_DOWNLOAD_PATH}"))
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
    use super::{download_audio, search_for};
    use std::error::Error;
    use tokio::runtime::Runtime;

    #[test]
    fn test_download_audio() -> Result<(), Box<dyn Error + Sync + Send>> {
        let handle = Runtime::new()?;
        let fut = download_audio("dQw4w9WgXcQ".to_string());
        let fpath = handle.block_on(fut)?;
        println!("download to {fpath:?}");
        Ok(())
    }

    #[test]
    fn test_query_videos() -> Result<(), Box<dyn Error + Sync + Send>> {
        let handle = Runtime::new()?;
        let fut = search_for("Rick astley", 5);
        let results = handle.block_on(fut)?;
        assert_eq!(results.len(), 5);
        Ok(())
    }
}
