//! This is the library for the application. The majority of the logic can be found here
//! It is split into two main parts. The parts that receive commands from discord [`commands`] and
//! the part that handles the actual logic of what to do in the [`model`]

mod commands;
mod model;

use std::str::FromStr;

use anyhow::{bail, Context as _};
use shuttle_secrets::SecretStore;

pub use self::{commands::commands_list, model::Data};

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

pub trait AccessSecrets {
    fn access_secret_parse<F: FromStr>(&self, key: &str) -> anyhow::Result<F>;
    fn access_secret_string(&self, key: &str) -> anyhow::Result<String>;
}
impl AccessSecrets for SecretStore {
    fn access_secret_parse<F: FromStr>(&self, key: &str) -> anyhow::Result<F> {
        let value = self.access_secret_string(key)?;
        match value.parse() {
            Ok(result) => Ok(result),
            Err(_) => bail!("failed to parse {key}. Value: {value:?}"),
        }
    }

    fn access_secret_string(&self, key: &str) -> anyhow::Result<String> {
        self.get(key)
            .with_context(|| format!("'{key}' was not found"))
    }
}
