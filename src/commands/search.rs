use std::pin::Pin;

use super::CommandHandler;
use futures::Future;

use twilight_http::Client as HttpClient;
use twilight_http::Error;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::Interaction;
use twilight_model::{application::callback::InteractionResponse, channel::message::MessageFlags};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};
use twilight_util::builder::CallbackDataBuilder;

use youtube_dl::{SearchOptions, YoutubeDl};

const COMMAND_NAME: &str = "search";
const COMMAND_DESCRIPTION: &str = "Search for a YouTube video by name";

fn execute(
    http: &HttpClient,
    interaction: Interaction,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + '_ + Send>> {
    if let Interaction::ApplicationCommand(interaction) = interaction {
        Box::pin(async move {
            println!("async start");
            let callback = CallbackDataBuilder::new()
                .content("Searching..".into())
                .flags(MessageFlags::EPHEMERAL)
                .build();

            let response = InteractionResponse::ChannelMessageWithSource(callback);
            http.interaction_callback(interaction.id, &interaction.token, &response)
                .exec()
                .await?;
            let query = &interaction.data.options[0].value;
            if let CommandOptionValue::String(query) = query {
                let results = YoutubeDl::search_for(&SearchOptions::youtube(query).with_count(10))
                    .youtube_dl_path("/data/samyakg/anaconda3/bin/youtube-dl")
                    .run()
                    .unwrap();
                match results {
                    youtube_dl::YoutubeDlOutput::Playlist(results) => {
                        if let Some(r) = results.entries {
                            http.delete_interaction_original(&interaction.token)
                                .unwrap()
                                .exec()
                                .await?;
                            // http.interaction_callback(
                            //     interaction.id,
                            //     &interaction.token,
                            //     &response,
                            // )
                            // .exec()
                            // .await?;
                            for video in r {
                                println!("vid res {:#?}", video.title);
                            }
                        }
                    }
                    youtube_dl::YoutubeDlOutput::SingleVideo(_) => println!("recv singlevid"),
                }
            } else {
                panic!("Invalid query!");
            }
            Ok::<(), Error>(())
        })
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
    .option(
        StringBuilder::new("query".into(), "What to search for on YouTube".into()).required(true),
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
