use std::error::Error;

use youtube_dl::{SearchOptions, SingleVideo, YoutubeDl, YoutubeDlOutput};

pub(crate) async fn search_for(
    query: &str,
    num_results: usize,
) -> Result<Vec<SingleVideo>, Box<dyn Error + Send + Sync>> {
    let results = YoutubeDl::search_for(&SearchOptions::youtube(query).with_count(num_results))
        .youtube_dl_path("/data/samyakg/anaconda3/bin/youtube-dl")
        .flat_playlist(true)
        .run()?;
    match results {
        YoutubeDlOutput::Playlist(results) => Ok(results.entries.unwrap()),
        _ => {
            panic!("Unhandled result type!");
        }
    }
}
