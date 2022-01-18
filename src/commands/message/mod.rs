use async_trait::async_trait;
use std::error::Error;
use std::ops::Deref;

use twilight_model::application::interaction::Interaction;

use crate::state::ApplicationState;

pub(crate) mod search;

pub struct MessageComponent {
    pub name: String,
}

#[async_trait]
pub trait MessageComponentWrapper: Deref<Target = MessageComponent> + Sync + Send {
    async fn execute(
        &self,
        appstate: &ApplicationState,
        interaction: Interaction,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub fn get_message_components() -> Vec<Box<dyn MessageComponentWrapper>> {
    vec![Box::new(search::Search::new())]
}
