use crate::helpers::youtube;
use crate::helpers::youtube::search::VideoInfo;

use super::ApplicationCommandWrapper;
use crate::state::ApplicationState;
use async_trait::async_trait;
use std::error::Error;
use std::ops::Deref;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::component::button::ButtonStyle;

use twilight_model::application::component::{ActionRow, Button, Component};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::Interaction;
use twilight_model::{application::callback::InteractionResponse, channel::message::MessageFlags};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};
use twilight_util::builder::CallbackDataBuilder;

const COMMAND_NAME: &str = "search";
const COMMAND_DESCRIPTION: &str = "Search for a song to play";
pub(crate) struct Search(Command);

impl Search {
    pub(crate) fn new() -> Search {
        Search(
            CommandBuilder::new(
                COMMAND_NAME.into(),
                COMMAND_DESCRIPTION.into(),
                CommandType::ChatInput,
            )
            .option(
                StringBuilder::new("query".to_string(), "song name to search for".to_string())
                    .required(true),
            )
            .build(),
        )
    }
}

impl Deref for Search {
    type Target = Command;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl ApplicationCommandWrapper for Search {
    async fn execute(
        &self,
        appstate: &ApplicationState,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Interaction::ApplicationCommand(interaction) = interaction {
            let callback = CallbackDataBuilder::new()
                .content("Searching...".into())
                .flags(MessageFlags::EPHEMERAL)
                .build();

            let response = InteractionResponse::ChannelMessageWithSource(callback);
            appstate
                .http
                .interaction_callback(interaction.id, &interaction.token, &response)
                .exec()
                .await?;

            let query = interaction.data.options[0].value.clone();
            if let CommandOptionValue::String(query) = query {
                let search_response = async move {
                    let results =
                        youtube::search::search_for(&query, 5, &appstate.config.youtube).await?;

                    let results: Vec<VideoInfo> = results
                        .iter()
                        .map(|v| {
                            let duration_str = format!("({}:{})", v.length / 60, v.length % 60);
                            VideoInfo {
                                title: format!(
                                    "{} {}",
                                    duration_str,
                                    v.title.replace(|c: char| !c.is_ascii(), ""),
                                )
                                .chars()
                                .into_iter()
                                .take(80)
                                .collect(),
                                video_id: v.video_id.clone(),
                                length: v.length,
                            }
                        })
                        .collect();

                    let menu_options: Vec<Component> = results
                        .iter()
                        .map(|v| {
                            Component::ActionRow(ActionRow {
                                components: vec![Component::Button(Button {
                                    emoji: None,
                                    label: Some(v.title.clone()),
                                    custom_id: Some(v.video_id.clone()),
                                    disabled: false,
                                    style: ButtonStyle::Primary,
                                    url: None,
                                })],
                            })
                        })
                        .collect();
                    let s = format!(
                        "Searched for `{}` and found {} results:",
                        query,
                        results.len()
                    );
                    println!("sending response to search..");
                    appstate
                        .http
                        .update_interaction_original(&interaction.token)?
                        .content(Some(&s))?
                        .components(Some(&menu_options))?
                        .exec()
                        .await?;
                    Ok::<(), Box<dyn Error + Send + Sync>>(())
                };

                search_response.await?;

                Ok(())
            } else {
                panic!("Invalid query!");
            }
        } else {
            panic!("Expected application command interaction!");
        }
    }
}
