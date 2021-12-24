mod ping;
mod search;

use twilight_http::response::marker::EmptyBody;
use twilight_http::response::ResponseFuture;
use twilight_http::{Client as HttpClient, Error, Response};
use twilight_model::application::command::Command;
use twilight_model::application::interaction::Interaction;
use twilight_model::gateway::payload::incoming::InteractionCreate;

pub struct CommandHandler {
    execute: fn(http: &HttpClient, interaction: Interaction) -> ResponseFuture<EmptyBody>,
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
) -> Result<Response<EmptyBody>, Error> {
    let commands_list = get_all_handlers();
    let command = match interaction_create.0 {
        Interaction::ApplicationCommand(ref interaction) => commands_list
            .iter()
            .find(|x| x.name == interaction.data.name),
        _ => None,
    };
    match command {
        None => panic!(
            "Failed to handle interaction of unknown type {:?}",
            interaction_create
        ),
        Some(command) => (command.execute)(http, interaction_create.0).await,
    }
}
