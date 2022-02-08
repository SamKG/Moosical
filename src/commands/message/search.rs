use super::{MessageComponent, MessageComponentWrapper};
use crate::state::{ApplicationState, EnqueuedVideo, GuildState};
use async_trait::async_trait;
use futures::TryStreamExt;
use twilight_model::id::UserId;

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
        appstate: &Arc<ApplicationState>,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Interaction::MessageComponent(interaction) = interaction {
            let feedback_msg: Result<String, String> = loop {
                let video_id = &interaction.data.custom_id;
                let url_parsed =
                    Url::parse(&format!("https://www.youtube.com/watch?v={video_id}"))?;
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
                    });
                if matched_button.is_none() {
                    break Err(format!(
                        "Did not find corresponding button for id {video_id}, {components:#?}"
                    ));
                }
                if let Some(matched_button) = matched_button {
                    let member = interaction.member.unwrap();
                    let user = member.user.unwrap();

                    let user_voice_state = appstate.cache.voice_state(user.id, guild_id);
                    let user_channel = match user_voice_state {
                        Some(s) => s.channel_id.unwrap(),
                        None => break Err("You must join a voice channel to do this!".to_string()),
                    };

                    let bot_voice_state = appstate.cache.voice_state(
                        UserId::new(u64::from(appstate.config.discord.app_id)).unwrap(),
                        guild_id,
                    );

                    if let Some(bot_channel) = bot_voice_state.and_then(|s| s.channel_id) {
                        if bot_channel != user_channel {
                            break Err(
                                "You must be in the same channel as the bot for this to work!"
                                    .to_string(),
                            );
                        }
                    }

                    let sender = {
                        let mut guild_state_map = appstate.guild_states.lock().await;
                        guild_state_map
                            .entry(guild_id)
                            .or_insert_with(|| GuildState::new(appstate))
                            .sender
                            .clone()
                    };

                    sender
                        .send(EnqueuedVideo {
                            url: url_parsed,
                            title: matched_button.label.as_ref().unwrap().to_string(),
                            user,
                            downloaded_path: None,
                        })
                        .await?;

                    break Ok("Added song to queue!".to_string());
                }
            };
            match feedback_msg {
                Ok(msg) => {
                    let callback = twilight_util::builder::CallbackDataBuilder::new()
                        .content(format!("✅ Success: {}", msg))
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
                }
                Err(err) => {
                    let callback = twilight_util::builder::CallbackDataBuilder::new()
                        .content(format!("❌ Failed: {}", err))
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
                    panic!("{}", err);
                }
            }
        } else {
            panic!("Expected application command interaction!");
        }
    }
}
