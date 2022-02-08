use futures::lock::Mutex;
use serde::Deserialize;
use std::{collections::HashMap, num::NonZeroU64, path::PathBuf, sync::Arc};
use tokio::{sync::mpsc, task::JoinHandle};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as HttpClient;
use twilight_model::{
    id::{ChannelId, GuildId},
    user::User,
};
use url::Url;

#[derive(Debug)]
pub struct ApplicationState {
    pub http: HttpClient,
    pub guild_states: Mutex<HashMap<GuildId, GuildState>>,
    pub config: ApplicationConfig,
    pub cache: InMemoryCache,
}

#[derive(Debug)]
pub struct EnqueuedVideo {
    pub(crate) url: Url,
    pub(crate) title: String,
    pub(crate) user: User,
    pub(crate) downloaded_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct GuildState {
    pub(crate) sender: mpsc::Sender<EnqueuedVideo>,
    thread_handle: JoinHandle<()>,
}

impl GuildState {
    pub(crate) fn new(appstate: &Arc<ApplicationState>) -> GuildState {
        let (tx, mut rx) = mpsc::channel(128);
        let appstate = appstate.clone();
        let handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(_) => {
                        println!("Working...");
                        appstate
                            .http
                            .channel(ChannelId::new(458035170608545806).unwrap())
                            .exec()
                            .await
                            .unwrap();
                    }
                    None => {
                        println!("Terminating.");
                        break;
                    }
                }
            }
        });
        GuildState {
            sender: tx,
            thread_handle: handle,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct DiscordConfig {
    pub token: String,
    pub app_id: NonZeroU64,
}

#[derive(Deserialize, Debug)]
pub struct YoutubeConfig {
    pub yt_dlp_path: String,
    pub audio_download_path: String,
}

#[derive(Deserialize, Debug)]
pub struct ApplicationConfig {
    pub discord: DiscordConfig,
    pub youtube: YoutubeConfig,
}
