use super::{MessageComponent, MessageComponentWrapper};
use crate::state::{ApplicationState, EnqueuedVideo};
use async_trait::async_trait;
use std::error::Error;
use std::ops::Deref;
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
            // println!("{:#?}", interaction);
            let url_parsed = Url::parse(&interaction.data.custom_id)?;
            let guild_id = interaction.guild_id.unwrap();
            let domain = url_parsed.domain().unwrap();
            if domain != "www.youtube.com" {
                panic!("Invalid url with domain {domain}!");
            }

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

                guild_state.queue.push(EnqueuedVideo {
                    url: url_parsed.as_str().to_string(),
                    title: matched_button.label.as_ref().unwrap().to_string(),
                    user: interaction.member.and_then(|m| m.user).unwrap(),
                    downloaded_path: None,
                });
            } else {
                panic!("No corresponding components in message!");
            }

            Ok(())
        } else {
            panic!("Expected application command interaction!");
        }
    }
}
