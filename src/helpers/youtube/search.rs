use serde::Deserialize;
use tokio::time::timeout;
use youtube_dl::{SearchOptions, YoutubeDl, YoutubeDlOutput};

use std::{error::Error, time::Duration};

use crate::state::YoutubeConfig;

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct VideoInfo {
    pub(crate) title: String,
    pub(crate) video_id: String,
    pub(crate) length: u64,
}

pub(crate) async fn search_for(
    query: &str,
    num_results: usize,
    config: &YoutubeConfig,
) -> Result<Vec<VideoInfo>, Box<dyn Error + Send + Sync>> {
    let t = timeout(Duration::from_secs(5), async {
        let results = YoutubeDl::search_for(&SearchOptions::youtube(query).with_count(num_results))
            .youtube_dl_path(config.yt_dlp_path.clone())
            .flat_playlist(true)
            .run()?;
        match results {
            YoutubeDlOutput::Playlist(results) => Ok(results
                .entries
                .unwrap()
                .iter()
                .map(|v| VideoInfo {
                    title: v.title.clone(),
                    video_id: v.id.clone(),
                    length: 0,
                })
                .collect()),
            _ => {
                panic!("Unhandled result type!");
            }
        }
    });
    t.await?
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use tokio::runtime::Runtime;

    use crate::{helpers::youtube::search::search_for, state::ApplicationConfig};

    #[test]
    fn test_query_videos() -> Result<(), Box<dyn Error + Sync + Send>> {
        let config_string = std::fs::read_to_string("config.toml")?;
        let config: ApplicationConfig = toml::from_str(&config_string)?;
        let handle = Runtime::new()?;
        let fut = search_for("Rick astley", 5, &config.youtube);
        let results = handle.block_on(fut)?;
        assert_eq!(results.len(), 5);
        Ok(())
    }
}
