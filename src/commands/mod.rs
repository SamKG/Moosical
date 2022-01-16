pub mod application;
use std::error::Error;

use application::get_application_commands;
use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::Interaction;
use twilight_model::gateway::payload::incoming::InteractionCreate;

pub async fn handle_interaction(
    http: &HttpClient,
    interaction_create: Box<InteractionCreate>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match interaction_create.0 {
        Interaction::ApplicationCommand(ref interaction) => {
            let commands_list = get_application_commands();
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
