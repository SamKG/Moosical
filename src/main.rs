use futures::stream::StreamExt;
use moosicyak::commands::{get_all_commands, handle_interaction};
use std::{env, error::Error, ops::Deref, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event, Intents,
};
use twilight_http::Client as HttpClient;
use twilight_model::id::{ApplicationId, GuildId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN")?;

    // This is also the default.
    let scheme = ShardScheme::Auto;

    // Specify intents requesting events about things like new and updated
    // messages in a guild and direct messages.
    let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES;

    let (cluster, mut events) = Cluster::builder(&token, intents)
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
    let http = Arc::new(HttpClient::new(token));
    let application_id: String = env::var("DISCORD_APP_ID")?;
    http.set_application_id(ApplicationId::from(
        std::num::NonZeroU64::new(application_id.parse()?).unwrap(),
    ));
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
        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http)));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::InteractionCreate(interaction) => {
            handle_interaction(http.deref(), interaction).await?;
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
            println!("Set guid commands for appid {:?}", http.application_id());
            http.set_guild_commands(
                GuildId::from(std::num::NonZeroU64::new(458035170608545802).unwrap()),
                &get_all_commands(),
            )?
            .exec()
            .await?;
        }
        _ => {}
    }

    Ok(())
}
