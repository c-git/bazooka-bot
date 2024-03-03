//! This is the library for the application. The majority of the logic can be found here
//! It is split into two main parts. The parts that receive commands from discord [`commands`] and
//! the part that handles the actual logic of what to do in the [`model`]

use tracing::{info, instrument};

pub use self::{
    commands::commands_list,
    config::{SharedConfig, StartupConfig},
    model::Data,
    secrets::{AccessSecrets, KeyName},
};

mod commands;
mod config;
mod migration;
mod model;
mod secrets;

/// Type used by poise framework as the context when commands are triggered
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

trait RemoveElement<T: PartialEq> {
    /// Returns true iff the element was found and removed
    fn remove_element(&mut self, element: &T) -> bool;
}

impl<T: PartialEq> RemoveElement<T> for Vec<T> {
    fn remove_element(&mut self, element: &T) -> bool {
        let index = self
            .iter()
            .enumerate()
            .find_map(|(i, x)| if x == element { Some(i) } else { None });
        if let Some(i) = index {
            self.remove(i);
            true
        } else {
            false
        }
    }
}

trait AuthorPreferredDisplay {
    async fn author_preferred_display(&self) -> String;
}

impl AuthorPreferredDisplay for Context<'_> {
    async fn author_preferred_display(&self) -> String {
        match self.author_member().await {
            Some(member) => member.display_name().to_string(),
            None => self.author().name.clone(),
        }
    }
}

trait Resettable: Default {
    fn reset(&mut self) {
        *self = Default::default();
    }
}

/// Removes identified problems with inputs
/// Not trying to remove all markdown just the parts that are
/// likely to cause issues. More will be added as needed
#[must_use]
#[instrument]
fn sanitize_markdown(s: String) -> String {
    const PATTERNS: [&str; 4] = ["**", "__", "```", "\n"];
    let mut result = s;
    for pattern in PATTERNS.iter() {
        result = result.replace(pattern, "");
    }
    info!(result);
    result
}
