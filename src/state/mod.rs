use futures::lock::Mutex;
use std::{collections::HashMap, path::PathBuf};
use twilight_http::Client as HttpClient;
use twilight_model::{id::GuildId, user::User};
use url::Url;
#[derive(Debug)]
pub struct ApplicationState {
    pub http: HttpClient,
    pub guild_states: Mutex<HashMap<GuildId, GuildState>>,
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
