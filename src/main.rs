use futures::{lock::Mutex, stream::StreamExt};
use moosicyak::{
    commands::{application::get_application_commands, handle_interaction},
    state::{ApplicationConfig, ApplicationState, GuildState},
};
use tokio::sync::RwLock;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + Send + Sync>> {
    let config_string = std::fs::read_to_string("config.toml")?;
    let config: ApplicationConfig = toml::from_str(&config_string)?;

    // This is also the default.
    let scheme = ShardScheme::Auto;

    // Specify intents requesting events about things like new and updated
    // messages in a guild and direct messages.
    let intents = Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES
        | Intents::GUILDS
        | Intents::GUILD_VOICE_STATES;

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

    let user_id = http.current_user().exec().await?.model().await?.id;

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE | ResourceType::VOICE_STATE | ResourceType::GUILD)
        .build();

    let appstate = Arc::new(ApplicationState {
        http,
        guild_states: RwLock::new(HashMap::new()),
        config,
        cache,
    });

    // Startup an event loop to process each event in the event stream as they
    // come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache.
        appstate.cache.update(&event);

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
            handle_interaction(appstate, interaction).await?;
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
        Event::GuildCreate(guild) => {
            let mut guild_states = appstate.guild_states.write().await;
            guild_states.insert(guild.id, GuildState::new(&appstate));
        }
        _ => {}
    }

    Ok(())
}
