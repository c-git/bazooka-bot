//! This modules handles access to all the secret information

use std::str::FromStr;

use anyhow::{bail, Context as _};
use secrecy::{zeroize::DefaultIsZeroes, Secret, SecretString};
use shuttle_secrets::SecretStore;
use tracing::warn;

pub enum KeyName {
    DiscordToken,

    /// Used mostly for testing to register the commands directly for the guild
    RegistrationGuildId,

    /// The RoleId of the role that can run privileged commands
    AuthRoleId,

    /// Comma separated list of owner IDs
    Owners,

    // /// The channel to be used for unranked (Indented to be used to restrict messages for unranked to that channel)
    // ChannelUnrankedId,

    // /// The channel to send messages to the internal group like when ppl leave the server for example
    // ChannelAdminId,
    //
    /// For bot status messages like on connection
    ChannelBotStatus,
}

impl AsRef<str> for KeyName {
    fn as_ref(&self) -> &str {
        match self {
            KeyName::DiscordToken => "DISCORD_TOKEN",
            KeyName::RegistrationGuildId => "REGISTRATION_GUILD_ID",
            KeyName::AuthRoleId => "AUTH_ROLE_ID",
            KeyName::Owners => "OWNERS",
            // KeyName::ChannelUnrankedId => "CHANNEL_UNRANKED_ID",
            // KeyName::ChannelAdminId => "CHANNEL_ADMIN_ID",
            KeyName::ChannelBotStatus => "CHANNEL_BOT_STATUS_ID",
        }
    }
}

impl KeyName {
    pub fn get_secret_string(&self, secret_store: &SecretStore) -> anyhow::Result<SecretString> {
        Ok(SecretString::new(
            secret_store.access_secret_string(self.as_ref())?,
        ))
    }

    #[allow(dead_code)] // Left implemented to not need to figure it out if we need it later
    pub fn get_secret_parse<F: FromStr + DefaultIsZeroes>(
        &self,
        secret_store: &SecretStore,
    ) -> anyhow::Result<Secret<F>> {
        Ok(Secret::new(
            secret_store.access_secret_parse(self.as_ref())?,
        ))
    }

    pub fn get_non_secret_string(&self, secret_store: &SecretStore) -> anyhow::Result<String> {
        secret_store.access_secret_string(self.as_ref())
    }

    pub fn get_non_secret_parse<F: FromStr>(
        &self,
        secret_store: &SecretStore,
    ) -> anyhow::Result<F> {
        secret_store.access_secret_parse(self.as_ref())
    }

    pub fn get_non_secret_parse_opt<F: FromStr>(&self, secret_store: &SecretStore) -> Option<F> {
        match secret_store.access_secret_parse(self.as_ref()) {
            Ok(x) => Some(x),
            Err(e) => {
                warn!(
                    "failed to optionally load {}. Defaulting to use None instead. Error was: {}",
                    self.as_ref(),
                    e
                );
                None
            }
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
            Err(_) => bail!("failed to parse {key}. Value: '{value}'"),
        }
    }

    fn access_secret_string(&self, key: &str) -> anyhow::Result<String> {
        self.get(key)
            .with_context(|| format!("'{key}' was not found"))
    }
}

pub fn get_secret_discord_token(secret_store: &SecretStore) -> anyhow::Result<SecretString> {
    KeyName::DiscordToken.get_secret_string(secret_store)
}
