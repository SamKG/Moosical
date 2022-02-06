use async_trait::async_trait;
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use twilight_model::application::command::Command;
use twilight_model::application::interaction::Interaction;

use crate::state::ApplicationState;

pub(crate) mod ping;
pub(crate) mod search;

#[async_trait]
pub trait ApplicationCommandWrapper: Deref<Target = Command> + Sync + Send {
    async fn execute(
        &self,
        appstate: Arc<ApplicationState>,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub fn get_application_commands() -> Vec<Box<dyn ApplicationCommandWrapper>> {
    vec![Box::new(ping::Ping::new()), Box::new(search::Search::new())]
}
