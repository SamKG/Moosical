pub mod application;
pub mod message;
use crate::state::ApplicationState;

use self::message::get_message_components;
use application::get_application_commands;
use std::error::Error;
use std::sync::Arc;
use twilight_model::application::interaction::Interaction;
use twilight_model::gateway::payload::incoming::InteractionCreate;

pub async fn handle_interaction(
    appstate: Arc<ApplicationState>,
    interaction_create: Box<InteractionCreate>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match interaction_create.0 {
        Interaction::ApplicationCommand(ref interaction) => {
            let commands = get_application_commands();
            let handler = commands
                .iter()
                .find(|x| x.name == interaction.data.name)
                .unwrap();
            handler.execute(&appstate, interaction_create.0).await?;
            Ok(())
        }
        Interaction::MessageComponent(ref interaction) => {
            let message_components = get_message_components();
            let handler = message_components
                .iter()
                .find(|x| x.name == interaction.message.interaction.as_ref().unwrap().name)
                .unwrap();
            handler.execute(&appstate, interaction_create.0).await?;
            Ok(())
        }
        _ => panic!("Received invalid interaction!"),
    }
}
