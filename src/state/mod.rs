use futures::lock::Mutex;
use serde::Deserialize;
use std::{collections::HashMap, num::NonZeroU64, path::PathBuf};
use twilight_http::Client as HttpClient;
use twilight_model::{id::GuildId, user::User};
use url::Url;

#[derive(Debug)]
pub struct ApplicationState {
    pub http: HttpClient,
    pub guild_states: Mutex<HashMap<GuildId, GuildState>>,
    pub config: ApplicationConfig,
}

#[derive(Debug)]
pub struct EnqueuedVideo {
    pub(crate) url: Url,
    pub(crate) title: String,
    pub(crate) user: User,
    pub(crate) downloaded_path: Option<PathBuf>,
}

#[derive(std::default::Default, Debug)]
pub struct GuildState {
    pub(crate) queue: Vec<EnqueuedVideo>,
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
