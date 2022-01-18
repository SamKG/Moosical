use futures::{lock::Mutex, stream::StreamExt};
use moosicyak::{
    commands::{application::get_application_commands, handle_interaction},
    state::ApplicationState,
};
use serde::Deserialize;
use std::{collections::HashMap, error::Error, num::NonZeroU64, ops::Deref, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event, Intents,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::command::Command,
    id::{ApplicationId, GuildId},
};

#[derive(Deserialize)]
struct DiscordConfig {
    token: String,
    app_id: NonZeroU64,
}
#[derive(Deserialize)]
struct Config {
    discord: DiscordConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + Send + Sync>> {
    let config_string = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_string)?;

    // This is also the default.
    let scheme = ShardScheme::Auto;

    // Specify intents requesting events about things like new and updated
    // messages in a guild and direct messages.
    let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES;

    let (cluster, mut events) = Cluster::builder(config.discord.token.clone(), intents)
        .shard_scheme(scheme)
        .build()
        .await?;

    let cluster = Arc::new(cluster);

    // Start up the cluster
    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // The http client is seperate from the gateway, so startup a new
    // one, also use Arc such that it can be cloned to other threads.
    let http = HttpClient::new(config.discord.token.clone());
    http.set_application_id(ApplicationId::from(config.discord.app_id));

    let appstate = Arc::new(ApplicationState {
        http,
        guild_states: Mutex::new(HashMap::new()),
    });

    // Since we only care about messages, make the cache only process messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Startup an event loop to process each event in the event stream as they
    // come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache.
        cache.update(&event);

        // Spawn a new task to handle the event
        tokio::spawn(handle_event(shard_id, event, Arc::clone(&appstate)));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    appstate: Arc<ApplicationState>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::InteractionCreate(interaction) => {
            handle_interaction(appstate.deref(), interaction).await?;
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
            println!(
                "Set guid commands for appid {:?}",
                appstate.http.application_id()
            );
            appstate
                .http
                .set_guild_commands(
                    GuildId::from(NonZeroU64::new(458035170608545802).unwrap()),
                    &get_application_commands()
                        .iter()
                        .map(|x| x.deref().deref())
                        .cloned()
                        .collect::<Vec<Command>>(),
                )?
                .exec()
                .await?;
        }
        _ => {}
    }

    Ok(())
}
