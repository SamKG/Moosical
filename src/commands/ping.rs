use std::pin::Pin;

use super::CommandHandler;
use futures::Future;
use twilight_http::{Client as HttpClient, Error};
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::Interaction;
use twilight_model::{application::callback::InteractionResponse, channel::message::MessageFlags};
use twilight_util::builder::command::CommandBuilder;
use twilight_util::builder::CallbackDataBuilder;

const COMMAND_NAME: &str = "ping";
const COMMAND_DESCRIPTION: &str = "Send a ping to MoosicYak servers";

fn execute(
    http: &HttpClient,
    interaction: Interaction,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + '_ + Send>> {
    if let Interaction::ApplicationCommand(interaction) = interaction {
        let callback = CallbackDataBuilder::new()
            .content("Pong!".into())
            .flags(MessageFlags::EPHEMERAL)
            .build();
        let response = InteractionResponse::ChannelMessageWithSource(callback);
        let fut = async move {
            http.interaction_callback(interaction.id, &interaction.token, &response)
                .exec()
                .await?;
            Ok::<(), Error>(())
        };
        Box::pin(fut)
    } else {
        panic!("Tried to use unhandled interaction type {:#?}", interaction)
    }
}

fn get_command() -> Command {
    CommandBuilder::new(
        COMMAND_NAME.into(),
        COMMAND_DESCRIPTION.into(),
        CommandType::ChatInput,
    )
    .build()
}
pub fn create_handler() -> CommandHandler {
    CommandHandler {
        execute,
        get_command,
        name: COMMAND_NAME.into(),
    }
}
