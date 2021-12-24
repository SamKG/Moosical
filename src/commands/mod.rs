mod ping;
mod search;

use std::pin::Pin;

use futures::Future;
use twilight_http::{Client as HttpClient, Error};
use twilight_model::application::command::Command;
use twilight_model::application::interaction::Interaction;
use twilight_model::gateway::payload::incoming::InteractionCreate;

type CommandCallback = fn(
    http: &HttpClient,
    interaction: Interaction,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + '_ + Send>>;

pub struct CommandHandler {
    execute: CommandCallback,
    get_command: fn() -> Command,
    name: String,
}

pub fn get_all_handlers() -> Vec<CommandHandler> {
    vec![ping::create_handler(), search::create_handler()]
}

pub fn get_all_commands() -> Vec<Command> {
    get_all_handlers()
        .iter()
        .map(|x| (x.get_command)())
        .collect()
}

pub async fn handle_interaction(
    http: &HttpClient,
    interaction_create: Box<InteractionCreate>,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + '_ + Send>> {
    let commands_list = get_all_handlers();
    let command = match interaction_create.0 {
        Interaction::ApplicationCommand(ref interaction) => commands_list
            .iter()
            .find(|x| x.name == interaction.data.name),
        _ => None,
    };
    if let Some(command) = command {
        (command.execute)(http, interaction_create.0)
    } else {
        panic!(
            "Failed to handle interaction of unknown type {:?}",
            interaction_create
        )
    }
}
