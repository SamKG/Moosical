mod ping;
mod search;

use std::error::Error;
use std::ops::Deref;

use async_trait::async_trait;
use twilight_http::Client as HttpClient;
use twilight_model::application::command::Command;
use twilight_model::application::interaction::{Interaction};
use twilight_model::gateway::payload::incoming::InteractionCreate;

#[async_trait]
pub trait ApplicationCommandWrapper: Deref<Target = Command> + Sync + Send {
    async fn execute(
        &self,
        http: &HttpClient,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub fn get_all_commands() -> Vec<Box<dyn ApplicationCommandWrapper>> {
    vec![Box::new(ping::Ping::new()), Box::new(search::Search::new())]
}

pub async fn handle_interaction(
    http: &HttpClient,
    interaction_create: Box<InteractionCreate>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let commands_list = get_all_commands();
    match interaction_create.0 {
        Interaction::ApplicationCommand(ref interaction) => {
            let command = commands_list
                .iter()
                .find(|x| x.name == interaction.data.name)
                .unwrap();
            command.execute(http, interaction_create.0).await?;
            Ok(())
        }
        Interaction::MessageComponent(ref interaction) => {
            println!("Recv msg component {:?}", interaction);
            Ok(())
        }
        _ => panic!("Received invalid command!"),
    }
}
