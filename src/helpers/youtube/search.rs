use serde::Deserialize;
use tokio::time::timeout;

use std::{
    error::Error,
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct VideoInfo {
    pub(crate) title: String,
    pub(crate) video_id: String,
    pub(crate) length: u64,
}

pub(crate) async fn search_for(
    query: &str,
    num_results: usize,
) -> Result<Vec<VideoInfo>, Box<dyn Error + Send + Sync>> {
    let stdout = Stdio::piped();
    let child = Command::new("python3")
        .arg("./search_yt.py")
        .arg("--query")
        .arg(format!("\"{query}\""))
        .stdout(stdout)
        .spawn()?;

    let t = timeout(Duration::from_secs(5), async {
        let output = child.wait_with_output()?;
        match output.status.success() {
            true => Ok(serde_json::from_slice::<Vec<VideoInfo>>(&output.stdout)?
                .iter()
                .take(num_results)
                .map(|v| {
                    let duration_str = format!("({}:{})", v.length / 60, v.length % 60);
                    VideoInfo {
                        title: format!(
                            "{} {}",
                            duration_str,
                            v.title.replace(|c: char| !c.is_ascii(), ""),
                        )
                        .chars()
                        .into_iter()
                        .take(80)
                        .collect(),
                        video_id: v.video_id.clone(),
                        length: v.length,
                    }
                })
                .collect()),
            false => panic!(
                "failed to do query with err {:?}",
                String::from_utf8_lossy(&output.stderr)
            ),
        }
    });
    t.await?
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use tokio::runtime::Runtime;

    use crate::helpers::youtube::search::search_for;

    #[test]
    fn test_query_videos() -> Result<(), Box<dyn Error + Sync + Send>> {
        let handle = Runtime::new()?;
        let fut = search_for("Rick astley", 5);
        let results = handle.block_on(fut)?;
        assert_eq!(results.len(), 5);
        Ok(())
    }
}
