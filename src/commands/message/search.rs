use super::{MessageComponent, MessageComponentWrapper};
use crate::state::{ApplicationState, EnqueuedVideo, GuildState};
use async_trait::async_trait;

use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use twilight_model::application::callback::InteractionResponse;
use twilight_model::application::component::Component;
use twilight_model::application::interaction::Interaction;
use url::Url;

const COMMAND_NAME: &str = "search";
pub(crate) struct Search(MessageComponent);

impl Search {
    pub(crate) fn new() -> Search {
        Search(MessageComponent {
            name: COMMAND_NAME.into(),
        })
    }
}

impl Deref for Search {
    type Target = MessageComponent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl MessageComponentWrapper for Search {
    async fn execute(
        &self,
        appstate: Arc<ApplicationState>,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Interaction::MessageComponent(interaction) = interaction {
            let video_id = &interaction.data.custom_id;
            let url_parsed = Url::parse(&format!("https://www.youtube.com/watch?v={video_id}"))?;
            let guild_id = interaction.guild_id.unwrap();

            let components = &interaction.message.components;
            let matched_button = components
                .iter()
                .filter_map(|c| match c {
                    Component::ActionRow(r) => Some(&r.components[0]),
                    _ => None,
                })
                .find_map(|c| match c {
                    Component::Button(b) => match &b.custom_id {
                        Some(id) => match *id == interaction.data.custom_id {
                            true => Some(b),
                            false => None,
                        },
                        None => None,
                    },
                    _ => None,
                })
                .unwrap_or_else(|| {
                    panic!("Did not find corresponding button for id {video_id}, {components:#?}")
                });

            let sender = {
                let mut guild_state_map = appstate.guild_states.lock().await;
                guild_state_map
                    .entry(guild_id)
                    .or_insert_with(|| GuildState::new(&appstate))
                    .sender
                    .clone()
            };
            sender
                .send(EnqueuedVideo {
                    url: url_parsed,
                    title: matched_button.label.as_ref().unwrap().to_string(),
                    user: interaction.member.and_then(|m| m.user).unwrap(),
                    downloaded_path: None,
                })
                .await?;

            let callback = twilight_util::builder::CallbackDataBuilder::new()
                .content("✅ Successfully added to queue!".to_string())
                .build();
            appstate
                .http
                .interaction_callback(
                    interaction.id,
                    &interaction.token,
                    &InteractionResponse::UpdateMessage(callback),
                )
                .exec()
                .await?;

            Ok(())
        } else {
            panic!("Expected application command interaction!");
        }
    }
}
