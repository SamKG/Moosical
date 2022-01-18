use super::ApplicationCommandWrapper;
use async_trait::async_trait;
use std::error::Error;
use std::ops::Deref;
use twilight_http::Client as HttpClient;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::{Interaction};
use twilight_model::{application::callback::InteractionResponse, channel::message::MessageFlags};
use twilight_util::builder::command::CommandBuilder;
use twilight_util::builder::CallbackDataBuilder;

const COMMAND_NAME: &str = "ping";
const COMMAND_DESCRIPTION: &str = "Send a ping to MoosicYak servers";
pub(crate) struct Ping(Command);

impl Ping {
    pub(crate) fn new() -> Ping {
        Ping(
            CommandBuilder::new(
                COMMAND_NAME.into(),
                COMMAND_DESCRIPTION.into(),
                CommandType::ChatInput,
            )
            .build(),
        )
    }
}

impl Deref for Ping {
    type Target = Command;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl ApplicationCommandWrapper for Ping {
    async fn execute(
        &self,
        http: &HttpClient,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Interaction::ApplicationCommand(interaction) = interaction {
            let callback = CallbackDataBuilder::new()
                .content("Pong!".into())
                .flags(MessageFlags::EPHEMERAL)
                .build();
            let response = InteractionResponse::ChannelMessageWithSource(callback);
            http.interaction_callback(interaction.id, &interaction.token, &response)
                .exec()
                .await?;
            Ok(())
        } else {
            panic!("Expected application command interaction!");
        }
    }
}