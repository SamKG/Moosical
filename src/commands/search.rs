use crate::helpers::youtube;

use super::ApplicationCommandWrapper;
use async_trait::async_trait;
use std::error::Error;
use std::ops::Deref;
use twilight_http::Client as HttpClient;
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
        http: &HttpClient,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Interaction::ApplicationCommand(interaction) = interaction {
            let callback = CallbackDataBuilder::new()
                .content("Searching...".into())
                .flags(MessageFlags::EPHEMERAL)
                .build();

            let response = InteractionResponse::ChannelMessageWithSource(callback);
            let search_msg_fut = http
                .interaction_callback(interaction.id, &interaction.token, &response)
                .exec();
            let query = interaction.data.options[0].value.clone();
            if let CommandOptionValue::String(query) = query {
                let yt_fut = async move {
                    let results = youtube::search_for(&query, 5).await?;
                    let menu_options = results
                        .iter()
                        .map(|v| {
                            Component::Button(Button {
                                emoji: None,
                                label: Some(v.title.clone()),
                                custom_id: Some(v.url.as_ref().unwrap().clone()),
                                disabled: false,
                                style: ButtonStyle::Primary,
                                url: None,
                            })
                        })
                        .collect();
                    // let select_menu = Component::ActionRow(SelectMenu {
                    //     custom_id: "SomeString1".into(),
                    //     disabled: false,
                    //     max_values: None,
                    //     min_values: None,
                    //     options: menu_options,
                    //     placeholder: Some("Select an option to play!".into()),
                    // });
                    let action_row = Component::ActionRow(ActionRow {
                        components: menu_options,
                    });
                    let s = format!(
                        "Searched for `{}` and found {} results:",
                        query,
                        results.len()
                    );
                    http.update_interaction_original(&interaction.token)?
                        .content(Some(&s))?
                        .components(Some(&[action_row]))?
                        .exec()
                        .await?;
                    println!("handle play with id: {}", interaction.id);
                    for video in results {
                        println!("vid res {:#?}", video.title);
                    }
                    Ok::<(), Box<dyn Error + Send + Sync>>(())
                };
                search_msg_fut.await?;

                yt_fut.await?;

                Ok(())
            } else {
                panic!("Invalid query!");
            }
        } else {
            panic!("Expected application command interaction!");
        }
    }
}
