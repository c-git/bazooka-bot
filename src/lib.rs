//! This is the library for the application. The majority of the logic can be found here
//! It is split into two main parts. The parts that receive commands from discord [`commands`] and
//! the part that handles the actual logic of what to do in the [`model`]

#![warn(unused_crate_dependencies)]

mod used_in_bin {
    use loadenv as _;
    use tracing_subscriber as _;
}

use secrecy::SecretString;

use clap::Parser;

use tracing::{info, instrument};

pub use self::{
    commands::commands_list,
    config::{SharedConfig, StartupConfig},
    model::Data,
};

mod commands;
mod config;
mod db;
pub mod heartbeat;
mod model;

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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ClapConfig {
    #[arg(long, env = "DISCORD_TOKEN")]
    pub discord_token: SecretString,

    /// Used mostly for testing to register the commands directly for the guild
    #[arg(long, env = "REGISTRATION_GUILD_ID")]
    pub registration_guild_id: String,

    /// The RoleId of the role that can run privileged commands
    #[arg(long, env = "AUTH_ROLE_ID")]
    pub auth_role_id: String,

    /// Comma separated list of owner IDs
    #[arg(long, env = "OWNERS")]
    pub owners: String,

    /// The channel to be used for unranked (Indented to be used to restrict messages for unranked to that channel)
    #[arg(long, env = "CHANNEL_UNRANKED_ID")]
    pub channel_unranked_id: String,

    /// For bot status messages like on connection
    #[arg(long, env = "CHANNEL_BOT_STATUS_ID")]
    pub channel_bot_status_id: String,
}
