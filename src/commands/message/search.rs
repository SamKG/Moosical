use super::{MessageComponent, MessageComponentWrapper};
use crate::state::{ApplicationState, EnqueuedVideo};
use async_trait::async_trait;

use std::error::Error;
use std::ops::Deref;
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
        appstate: &ApplicationState,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Interaction::MessageComponent(interaction) = interaction {
            let video_id = dbg!(&interaction.data.custom_id);
            let url_parsed = Url::parse(&format!("https://www.youtube.com/watch?v={video_id}"))?;
            let guild_id = interaction.guild_id.unwrap();

            if let Component::ActionRow(entry) = &interaction.message.components[0] {
                let components = &entry.components;
                let matched_button = components
                    .iter()
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
                    .expect("Did not find corresponding button!");
                let mut guild_state_map = appstate.guild_states.lock().await;
                let guild_state = guild_state_map.entry(guild_id).or_default();
                if guild_state.queue.iter().any(|x| x.url == url_parsed) {
                    let callback = twilight_util::builder::CallbackDataBuilder::new()
                        .content("❌ Song already in queue!".to_string())
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
                    return Ok(());
                }
                guild_state.queue.push(EnqueuedVideo {
                    url: url_parsed,
                    title: matched_button.label.as_ref().unwrap().to_string(),
                    user: interaction.member.and_then(|m| m.user).unwrap(),
                    downloaded_path: None,
                });

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
            } else {
                panic!("No corresponding components in message!");
            }

            Ok(())
        } else {
            panic!("Expected application command interaction!");
        }
    }
}
